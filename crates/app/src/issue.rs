use std::io;

use todo_lib::id::HashedId;
use todo_lib::issue::Issue;
use todo_tracker_fs::issue::SaveIssue;
use todo_tracker_fs::{tracker, Placement};

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
