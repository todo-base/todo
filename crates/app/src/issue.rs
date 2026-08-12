use std::io;
use std::path::PathBuf;

use todo_lib::id::HashedId;
use todo_lib::issue::Issue;
use todo_tracker_fs::config::FsProjectConfig;
use todo_tracker_fs::generator::IntIdGenerator;
use todo_tracker_fs::issue::SaveIssue;
use todo_tracker_fs::plan::PlanSource;
use todo_tracker_fs::plan::parse::{id_prefix, parse as parse_plan};
use todo_tracker_fs::{Placement, patch, tracker};

use crate::config::SourceConfig;
use crate::project::ProjectData;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Order {
    First,
    Last,
    // Before(u64),
    // After(u64),
}

pub fn add<ID: HashedId + Default>(
    ProjectData::Fs(project_metadata): ProjectData<ID>,
    config: &SourceConfig,
    order: Order,
    name: impl AsRef<str> + Into<String>,
    content: impl Into<String>,
) -> io::Result<()> {
    let (project_config, _) = project_metadata.into_config();

    if let Some(plan) = tracker::load_project_plan(
        &project_config,
        &config.manifest_filename_regex,
        &config.issues_filename_regex,
    )? {
        let name_ref = name.as_ref();
        if plan.find_issue(name_ref).is_some() {
            let error = if let Some(root_dir) = &project_config.root_dir {
                format!("issue `{name_ref}` in `{}` already exists", root_dir.display())
            } else {
                format!("issue `{name_ref}` already exists")
            };
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, error));
        }
    }

    let issue = Issue::new(0, name).with_content(content);
    let destination = issues_placement(&project_config, config);

    match order {
        Order::First => issue.add_first(&destination),
        Order::Last => issue.add_last(&destination),
    }
}

/// Where the project keeps its issues: the placement that already exists, or the
/// default issues file.
fn issues_placement<ID: HashedId>(project_config: &FsProjectConfig<ID>, config: &SourceConfig) -> Placement<PathBuf> {
    let root_dir = project_config.root_dir.clone().unwrap_or_default();
    let project_name = project_config.name.as_deref();
    config
        .find_issues_placement(&root_dir, project_name)
        .unwrap_or_else(|| Placement::WholeFile(config.make_issues_file_path(root_dir, project_name)))
}

/// An issue name is written back as the first line of its list item, so it has to
/// survive a round trip through the reader.
fn check_issue_name(name: &str) -> io::Result<()> {
    let invalid = |reason| io::Error::new(io::ErrorKind::InvalidInput, format!("invalid issue name: {reason}"));
    if name.trim().is_empty() {
        Err(invalid("must not be empty".into()))
    } else if name.contains(['\n', '\r']) {
        Err(invalid("must be a single line".into()))
    } else if let Some(prefix) = id_prefix(name) {
        Err(invalid(format!(
            "must not start with `{prefix}` — it would be read back as an issue id"
        )))
    } else {
        Ok(())
    }
}

/// Rename an existing issue in place (identified by exact name).
///
/// Patches only the issue's name byte span; the rest of the file is preserved
/// byte-identical.
pub fn rename<ID: HashedId + Default>(
    ProjectData::Fs(project_metadata): ProjectData<ID>,
    config: &SourceConfig,
    target_name: impl AsRef<str>,
    new_name: impl AsRef<str>,
) -> io::Result<()> {
    let new_name = new_name.as_ref();
    check_issue_name(new_name)?;

    let (project_config, _) = project_metadata.into_config();
    let start_id = project_config.start_id.unwrap_or(1);
    let destination = issues_placement(&project_config, config);

    let path = destination.as_ref().as_path();
    let snapshot = patch::FileMetaSnapshot::capture(path)?;
    let Some(plan_source) = PlanSource::read(&destination)? else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("`{}` does not exist", path.display()),
        ));
    };
    let parsed = parse_plan::<u64, _>(plan_source.text(), &IntIdGenerator::new(start_id))?;

    let target = target_name.as_ref();
    let mut matching = parsed.plan.issues().values().filter(|issue| issue.name == target);
    let id = match (matching.next(), matching.next()) {
        (Some(issue), None) => issue.id,
        (Some(_), Some(_)) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("issue `{target}` is ambiguous; specify a unique name"),
            ));
        },
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("issue `{target}` not found"),
            ));
        },
    };
    if parsed.plan.issues().values().any(|issue| issue.name == new_name) {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("issue `{new_name}` already exists"),
        ));
    }

    let name_span = parsed
        .name_spans
        .get(&id)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, format!("issue `{target}` has no name span")))?;
    let patched = patch::replace_range(plan_source.content(), plan_source.file_range(name_span), new_name);
    patch::write_if_unchanged(path, &snapshot, &patched)
}
