use std::io;
use std::path::Path;

use anyhow::anyhow;
use indexmap::IndexMap;
use thiserror::Error;
use todo_tracker_fs::config::{find_project, find_projects, DeserializedId, FsProjectConfig, LoadConfigError};
use todo_tracker_fs::FsTracker;

use crate::config::{Config, WorkingMode};
use crate::target::Location;

pub mod config;
pub mod issue;
pub mod project;
pub mod target;

#[derive(Debug, Error)]
pub enum OpenTrackerError {
    #[error("Fail to load config: {0}")]
    LoadConfig(#[from] LoadConfigError),

    #[error("Fail to create tracker: {0}")]
    CreateTracker(#[from] io::Error),
}

pub fn locate_project_config<PID>(
    location: Location<PID>,
    local_search_roots: impl IntoIterator<Item = impl AsRef<Path>> + Clone,
    config: &Config,
) -> anyhow::Result<Option<FsProjectConfig<PID>>>
where
    PID: DeserializedId + Clone + Ord + ToString + TryFrom<String>,
    <PID as TryFrom<String>>::Error: Into<anyhow::Error>,
{
    match location {
        Location::Path(path) => Ok(find_project::<PID>(path, |path| {
            config.source.find_project_config_placement(path, None)
        })),
        Location::Id(id) => locate_project_config_by_id::<PID>(id.to_string(), local_search_roots, config),
        Location::Name(name) => locate_project_config_by_name::<PID>(name, local_search_roots, config),
        Location::IdOrName(id_or_name) => {
            let mut project_config =
                locate_project_config_by_id::<PID>(&id_or_name, local_search_roots.clone(), config)?;
            if project_config.is_none() {
                project_config = locate_project_config_by_name::<PID>(id_or_name, local_search_roots, config)?;
            }

            Ok(project_config)
        },
    }
}

pub fn locate_project_configs<PID>(
    location: Option<Location<PID>>,
    local_search_roots: impl IntoIterator<Item = impl AsRef<Path>> + Clone,
    config: &Config,
) -> anyhow::Result<IndexMap<PID, FsProjectConfig<PID>>>
where
    PID: DeserializedId + Clone + Ord + ToString + TryFrom<String>,
    <PID as TryFrom<String>>::Error: Into<anyhow::Error>,
{
    let mut projects = IndexMap::new();

    match location {
        Some(location) => {
            if let Some(project_config) = locate_project_config(location, local_search_roots, config)? {
                projects.insert(project_config.id.clone(), project_config);
            }
        },
        None => {
            projects.extend(projects_list_from_search_roots(local_search_roots, config));

            if config.working_mode.is_global() && config.list.projects.enabled {
                for (id, project_config) in &config.project {
                    let project_id = PID::try_from(id.clone()).map_err(Into::into)?;
                    let loaded_project_config = project_config
                        .load_fs_project_config(project_id, &config.source)?
                        .ok_or_else(|| {
                            anyhow!(
                                "wrong project path or configuration: project id: {id}, path: {:?}",
                                project_config.path
                            )
                        })?;

                    projects.insert(loaded_project_config.id.clone(), loaded_project_config);
                }
            }
        },
    }

    Ok(projects)
}

fn projects_list_from_search_roots<PID>(
    local_search_roots: impl IntoIterator<Item = impl AsRef<Path>>,
    config: &Config,
) -> IndexMap<PID, FsProjectConfig<PID>>
where
    PID: DeserializedId + Ord + Clone,
{
    match config.working_mode {
        WorkingMode::Local => find_projects::<PID>(local_search_roots, |path| {
            config.source.find_project_config_placement(path, None)
        }),
        WorkingMode::Global if config.search.projects.enabled => {
            find_projects::<PID>(&config.search.projects.dirs, |path| {
                config.source.find_project_config_placement(path, None)
            })
        },
        _ => Default::default(),
    }
}

pub fn locate_project_config_by_id<PID>(
    serialized_id: impl AsRef<str> + Into<String>,
    local_search_roots: impl IntoIterator<Item = impl AsRef<Path>>,
    config: &Config,
) -> anyhow::Result<Option<FsProjectConfig<PID>>>
where
    PID: DeserializedId + Clone + Ord + TryFrom<String>,
    <PID as TryFrom<String>>::Error: Into<anyhow::Error>,
{
    if config.working_mode.is_global() && config.list.projects.enabled {
        for (id, project_config) in &config.project {
            if id.as_str() == serialized_id.as_ref() {
                let project_id = PID::try_from(serialized_id.into()).map_err(Into::into)?;
                let loaded_project_config = project_config
                    .load_fs_project_config(project_id, &config.source)?
                    .ok_or_else(|| {
                        anyhow!(
                            "wrong project path or configuration; project id: {id}, path: {:?}",
                            project_config.path
                        )
                    })?;

                return Ok(Some(loaded_project_config));
            }
        }
    }

    let projects_list = projects_list_from_search_roots(local_search_roots, config);
    if !projects_list.is_empty() {
        let project_id = PID::try_from(serialized_id.into()).map_err(Into::into)?;
        for (id, project_config) in projects_list {
            if id == project_id {
                return Ok(Some(project_config));
            }
        }
    }

    Ok(None)
}

pub fn locate_project_config_by_name<PID>(
    name: impl AsRef<str>,
    local_search_roots: impl IntoIterator<Item = impl AsRef<Path>>,
    config: &Config,
) -> anyhow::Result<Option<FsProjectConfig<PID>>>
where
    PID: DeserializedId + Clone + Ord + TryFrom<String>,
    <PID as TryFrom<String>>::Error: Into<anyhow::Error>,
{
    let name = name.as_ref();

    if config.working_mode.is_global() && config.list.projects.enabled {
        for (id, project_config) in &config.project {
            let project_id = PID::try_from(id.clone()).map_err(Into::into)?;
            let loaded_project_config = project_config
                .load_fs_project_config(project_id, &config.source)?
                .ok_or_else(|| {
                    anyhow!(
                        "wrong project path or configuration; project id: {id}, path: {:?}",
                        project_config.path
                    )
                })?;

            let name = loaded_project_config.name.as_ref();
            if name == loaded_project_config.name.as_ref() {
                return Ok(Some(loaded_project_config));
            }
        }
    }

    let projects_list = projects_list_from_search_roots(local_search_roots, config);
    for (_, project_config) in projects_list {
        if project_config.name.as_deref() == Some(name) {
            return Ok(Some(project_config));
        }
    }

    Ok(None)
}

pub fn open_tracker<ID>(
    location: Option<Location<ID>>,
    local_search_roots: impl IntoIterator<Item = impl AsRef<Path>> + Clone,
    config: &Config,
) -> anyhow::Result<FsTracker<ID>>
where
    ID: DeserializedId + Clone + Ord + ToString + TryFrom<String>,
    <ID as TryFrom<String>>::Error: Into<anyhow::Error>,
{
    let projects = locate_project_configs(location, local_search_roots, config)?;
    Ok(FsTracker::new(
        projects,
        &config.source.manifest_filename_regex,
        &config.source.issues_filename_regex,
    )?)
}
