use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context};
use todo_tracker_fs::config::{FsProjectConfig, SerializedId};
use todo_tracker_fs::file::find_by_name_part;

pub use self::metadata::{FsProjectMetadata, ProjectData};
use crate::config::SourceConfig;

pub mod metadata;

pub fn create<ID: SerializedId>(ProjectData::Fs(project_metadata): ProjectData<ID>) -> anyhow::Result<()> {
    let (id, name, root_dir, _, config_placement, config) = project_metadata.destructure();
    if root_dir.exists() {
        return Err(anyhow!("destination `{}` already exists", root_dir.display()));
    }

    fs::create_dir(&root_dir).with_context(|| format!("Failed to create project directory {}", root_dir.display()))?;

    if let Some(destination) = config_placement {
        if let Some(project_config) = config {
            project_config.save(destination)?;
        } else if let Some(id) = id {
            let project_config = FsProjectConfig::new(id).with_name(name);
            project_config.save(destination)?;
        } else {
            let project_config = FsProjectConfig::new(name.clone()).with_name(name);
            project_config.save(destination)?;
        }
    }

    Ok(())
}

pub fn init<ID: SerializedId>(
    ProjectData::Fs(project_metadata): ProjectData<ID>,
    source_config: &SourceConfig,
) -> anyhow::Result<()> {
    let (id, name, root_dir, _, config_placement, config) = project_metadata.destructure();
    if !root_dir.exists() {
        return Err(anyhow!("destination `{}` does not exists", root_dir.display()));
    }

    if let Some(destination) = config_placement {
        if destination.as_ref().exists() {
            return Err(anyhow!(
                "destination `{}` already exists",
                destination.as_ref().display()
            ));
        }

        let project_manifest_file_name = source_config.project_manifest_file_name(&name);
        let project_manifest_file = root_dir.join(project_manifest_file_name);
        if project_manifest_file.exists() {
            return Err(anyhow!(
                "destination `{}` already exists",
                project_manifest_file.display()
            ));
        }

        let project_config_file = root_dir.join(&source_config.project_config_file);
        if project_config_file.exists() {
            return Err(anyhow!(
                "destination `{}` already exists",
                project_config_file.display()
            ));
        }

        if let Some(project_config) = config {
            project_config.save(destination)?;
        } else if let Some(id) = id {
            let project_config = FsProjectConfig::new(id).with_name(name);
            project_config.save(destination)?;
        } else {
            let project_config = FsProjectConfig::new(name.clone()).with_name(name);
            project_config.save(destination)?;
        }
    }

    Ok(())
}

pub fn default_path(root: impl Into<PathBuf>, config: &SourceConfig) -> anyhow::Result<PathBuf> {
    let mut current_dir = root.into();
    let current_dir_string: String = current_dir.to_string_lossy().into();
    let example_project_name = config.manifest_example_project_name();
    let filename_part = config.manifest_filename_example.replace(example_project_name, "");

    loop {
        let project_config_file = current_dir.join(&config.project_config_file);
        if project_config_file.exists() {
            return Ok(current_dir);
        } else {
            if let Some(_manifest_file) = find_by_name_part(&current_dir, &filename_part) {
                return Ok(current_dir);
            }

            let Some(parent) = current_dir.parent() else {
                break;
            };
            current_dir = parent.to_path_buf();
        }
    }

    Err(anyhow!(
        "could not find `{}` or `{}` in `{current_dir_string}` or any parent directory",
        config.project_config_file.display(),
        config.manifest_filename_example.replace(example_project_name, "*")
    ))
}
