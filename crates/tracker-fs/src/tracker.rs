use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use indexmap::{IndexMap, IndexSet};
use regex::Regex;
use todo_lib::id::HashedId;
use todo_lib::plan::Plan;
use todo_lib::project::Project;

use crate::config::FsProjectConfig;
use crate::file::find_by_regex;
use crate::generator::IntIdGenerator;
use crate::plan::LoadProjectPlan;
use crate::project::LoadProject;
use crate::Placement;

pub struct FsTracker<PID = String, ID = u64> {
    projects: IndexMap<PID, Project<PID>>,
    project_root_dirs: HashMap<PID, PathBuf>,
    parents: IndexMap<PID, PID>,
    planes: HashMap<PID, Plan<ID>>,
}

impl<PID: HashedId + Clone> FsTracker<PID> {
    pub fn new(
        project_configs: IndexMap<PID, FsProjectConfig<PID>>,
        manifest_filename_regex: &Regex,
        todo_filename_regex: &Regex,
    ) -> io::Result<Self> {
        let mut projects = IndexMap::new();
        let mut project_root_dirs = HashMap::new();
        let mut parents = IndexMap::new();
        let mut planes = HashMap::new();

        for (parent_id, config) in &project_configs {
            parents.extend(config.subprojects.iter().cloned().map(|id| (id, parent_id.clone())));
        }

        for (id, config) in project_configs {
            if let Some(project_root) = config.root_dir.clone() {
                if let Some(plan) = load_project_plan(&config, manifest_filename_regex, todo_filename_regex)? {
                    planes.insert(id.clone(), plan);
                }
                project_root_dirs.insert(id.clone(), project_root);
            }
            let project = Project::load(id.clone(), parents.get(&id).cloned(), config);
            projects.insert(id, project);
        }

        Ok(Self {
            projects,
            project_root_dirs,
            parents,
            planes,
        })
    }

    pub fn projects(&self) -> &IndexMap<PID, Project<PID>> {
        &self.projects
    }

    pub fn project_parents(&self) -> &IndexMap<PID, PID> {
        &self.parents
    }

    pub fn subprojects(&self) -> IndexMap<PID, IndexSet<PID>> {
        let mut subprojects: IndexMap<_, IndexSet<_>> = IndexMap::new();
        for (child, parent) in &self.parents {
            let child = child.clone();
            if let Some(children) = subprojects.get_mut(parent) {
                children.insert(child);
            } else {
                subprojects.insert(parent.clone(), IndexSet::from([child.clone()]));
            }
        }
        subprojects
    }

    pub fn project_plan(&self, id: &PID) -> Option<&Plan<u64>> {
        self.planes.get(id)
    }

    pub fn project_root_dir(&self, id: &PID) -> Option<&Path> {
        self.project_root_dirs.get(id).map(|path| path.as_path())
    }
}

pub fn load_project_plan<PID>(
    project_config: &FsProjectConfig<PID>,
    manifest_filename_regex: &Regex,
    issues_filename_regex: &Regex,
) -> io::Result<Option<Plan<u64>>>
where
    PID: HashedId,
{
    let Some(project_root) = project_config.root_dir.clone() else {
        return Ok(None);
    };
    let id_generator = IntIdGenerator::new(project_config.start_id.unwrap_or(1));
    let mut plan = Plan::new();
    let mut plan_exists = false;

    if let Some(manifest_source) = find_by_regex(&project_root, manifest_filename_regex).map(Placement::CodeBlockInFile)
    {
        plan = match Plan::load(&manifest_source, &id_generator)? {
            Some(plan) => plan,
            None => return Ok(None),
        };
        plan_exists = true;
    }

    if let Some(issues_source) = find_by_regex(&project_root, issues_filename_regex).map(Placement::WholeFile) {
        plan = plan.merge(match Plan::load(&issues_source, &id_generator)? {
            Some(plan) => plan,
            None => return Ok(None),
        });
        plan_exists = true;
    }

    if plan_exists {
        Ok(Some(plan))
    } else {
        Ok(None)
    }
}
