use std::{fs, io};

use todo_lib::id::HashedId;
use todo_lib::issue::Issue;
use todo_tracker_fs::generator::IntIdGenerator;
use todo_tracker_fs::issue::SaveIssue;
use todo_tracker_fs::plan::extract_md_todo_block;
use todo_tracker_fs::plan::parse::parse as parse_plan;
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
    let project_root_dir = project_config.root_dir.unwrap_or_default();
    let project_name = project_config.name;
    let destination = config
        .find_issues_placement(&project_root_dir, project_name.as_deref())
        .unwrap_or_else(|| {
            Placement::WholeFile(config.make_issues_file_path(project_root_dir, project_name.as_deref()))
        });

    match order {
        Order::First => issue.add_first(&destination),
        Order::Last => issue.add_last(&destination),
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
    new_name: &str,
) -> io::Result<()> {
    let (project_config, _) = project_metadata.into_config();
    let project_root_dir = project_config.root_dir.unwrap_or_default();
    let project_name = project_config.name;
    let start_id = project_config.start_id.unwrap_or(1);
    let destination = config
        .find_issues_placement(&project_root_dir, project_name.as_deref())
        .unwrap_or_else(|| {
            Placement::WholeFile(config.make_issues_file_path(project_root_dir, project_name.as_deref()))
        });

    let path = destination.as_ref().as_path();
    let snapshot = patch::FileMetaSnapshot::capture(path)?;
    let content = fs::read_to_string(path)?;
    let (block_start, plan_src): (usize, &str) = match &destination {
        Placement::WholeFile(_) => (0, content.as_str()),
        Placement::CodeBlockInFile(_) => extract_md_todo_block(&content)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no `md todo` block"))?,
    };

    let id_gen = IntIdGenerator::new(start_id);
    let parsed = parse_plan::<u64, _>(plan_src, &id_gen)?;

    let target = target_name.as_ref();
    let candidates: Vec<_> = parsed
        .plan
        .issues()
        .values()
        .filter(|issue| issue.name == target)
        .collect();
    let id = match candidates.as_slice() {
        [] => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("issue `{target}` not found"),
            ));
        },
        [issue] => issue.id,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("issue `{target}` is ambiguous; specify a unique name"),
            ));
        },
    };

    let name_span = parsed
        .name_spans
        .get(&id)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "issue has no name span"))?;
    let file_span = (block_start + name_span.start)..(block_start + name_span.end);
    let patched = patch::replace_range(&content, file_span, new_name);
    patch::write_if_unchanged(path, &snapshot, &patched)?;
    Ok(())
}
