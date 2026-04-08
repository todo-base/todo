use todo_lib::id::HashedId;
use todo_lib::project::Project;

use crate::config::FsProjectConfig;

pub trait LoadProject<ID: HashedId> {
    fn load(id: ID, parent: Option<ID>, config: FsProjectConfig<ID>) -> Project<ID>;
}

impl<ID: HashedId> LoadProject<ID> for Project<ID> {
    fn load(id: ID, parent: Option<ID>, config: FsProjectConfig<ID>) -> Project<ID> {
        let mut project = Project::new(id, config.name.unwrap_or_default()).with_subprojects(config.subprojects);
        if let Some(parent_id) = parent {
            project.set_parent(parent_id);
        }
        project
    }
}
