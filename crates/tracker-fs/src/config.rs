use std::io;
use std::path::{Path, PathBuf};

use fs_err as fs;
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use todo_lib::id::HashedId;
use walkdir::WalkDir;

use crate::Placement;

#[derive(Debug, Error)]
pub enum LoadConfigError {
    #[error("{0}")]
    FailToRead(#[from] io::Error),

    #[error("fail to deserialize: {0}")]
    FailToDeserialize(#[from] toml::de::Error),
}

#[derive(Debug, Error)]
pub enum SaveConfigError {
    #[error("{0}")]
    FailToWrite(#[from] io::Error),

    #[error("fail to serialize: {0}")]
    FailToSerialize(#[from] toml::ser::Error),
}

pub trait SerializedId: Serialize + HashedId {}
impl<T> SerializedId for T where T: Serialize + HashedId {}

pub trait DeserializedId: for<'a> Deserialize<'a> + HashedId {}
impl<T> DeserializedId for T where T: for<'a> Deserialize<'a> + HashedId {}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FsProjectConfig<ID: HashedId = String> {
    pub id: ID,

    pub name: Option<String>,

    pub desc: Option<String>,

    pub root_dir: Option<PathBuf>,

    #[serde(default = "Default::default")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    pub start_id: Option<u64>,

    #[serde(default = "Default::default")]
    #[serde(skip_serializing_if = "IndexSet::is_empty")]
    pub subprojects: IndexSet<ID>,
}

impl<ID: HashedId> FsProjectConfig<ID> {
    const MD_BLOCK_START: &'static str = "```toml project";
    const MD_BLOCK_END: &'static str = "```";

    pub fn new(id: ID) -> Self {
        Self {
            id,
            name: None,
            desc: None,
            root_dir: None,
            tags: Default::default(),
            start_id: None,
            subprojects: Default::default(),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_root_dir(mut self, root_dir: PathBuf) -> Self {
        self.root_dir = Some(root_dir);
        self
    }
}

impl<ID: DeserializedId> FsProjectConfig<ID> {
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    pub fn load(source: &Placement<impl AsRef<Path>>) -> Result<Self, LoadConfigError> {
        match source {
            Placement::WholeFile(path) => {
                let content = fs::read_to_string(path.as_ref())?;
                Ok(Self::from_toml(&content)?)
            },
            Placement::CodeBlockInFile(path) => {
                let mut content = String::new();
                let file_content = fs::read_to_string(path.as_ref())?;

                let mut in_block = false;
                for line in file_content.lines() {
                    if line.trim().starts_with('`') && line.trim().to_lowercase() == Self::MD_BLOCK_START {
                        in_block = true;
                    } else if line.trim().starts_with('`') && line.trim().to_lowercase() == Self::MD_BLOCK_END {
                        in_block = false;
                    } else if in_block {
                        content.push_str(line);
                        content.push('\n');
                    }
                }

                Self::from_toml(content.trim_end()).map_err(Into::into)
            },
        }
    }
}

impl<ID: SerializedId> FsProjectConfig<ID> {
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(self)
    }

    pub fn save(&self, destination: Placement<impl AsRef<Path>>) -> Result<(), SaveConfigError> {
        match destination {
            Placement::WholeFile(path) => {
                let content = self.to_toml()?;
                fs::write(path, content).map_err(Into::into)
            },
            Placement::CodeBlockInFile(path) => {
                let content = format!(
                    "{prefix}\n{config}\n{suffix}\n",
                    prefix = Self::MD_BLOCK_START,
                    config = self.to_toml()?.trim_end(),
                    suffix = Self::MD_BLOCK_END
                );

                if path.as_ref().exists() {
                    let file_content = fs::read_to_string(path.as_ref())?;

                    let mut new_content = String::new();
                    let mut inserted = false;
                    let mut in_block = false;
                    for line in file_content.lines() {
                        if line.trim().starts_with('`') && line.trim().to_lowercase() == Self::MD_BLOCK_START {
                            in_block = true;
                            new_content.push_str(&content);
                            inserted = true;
                        }
                        if !in_block {
                            new_content.push_str(line);
                            new_content.push('\n');
                        }
                        if line.trim().starts_with('`') && line.trim().to_lowercase() == Self::MD_BLOCK_END {
                            in_block = false;
                        }
                    }
                    if !inserted {
                        new_content.push_str(&content);
                    }

                    fs::write(path, new_content).map_err(Into::into)
                } else {
                    let project_name = self.name.as_deref().unwrap_or("");
                    fs::write(path, format!("# {project_name}\n\n{content}")).map_err(Into::into)
                }
            },
        }
    }
}

pub fn find_project<ID>(
    project_root: impl AsRef<Path>,
    get_project_config_placement: impl Fn(&Path) -> Option<Placement<PathBuf>>,
) -> Option<FsProjectConfig<ID>>
where
    ID: DeserializedId + Ord + Clone,
{
    if let Some(project_config_placement) = get_project_config_placement(project_root.as_ref()) {
        let root_dir = project_config_placement.as_ref().parent();
        if let Ok(mut project_config) = FsProjectConfig::<ID>::load(&project_config_placement) {
            if project_config.root_dir.is_none() {
                project_config.root_dir = root_dir.map(ToOwned::to_owned);
            }

            return Some(project_config);
        }
    }

    None
}

#[derive(Debug)]
struct Parent<ID> {
    id: ID,
    root_dir: PathBuf,
}

pub fn find_projects<ID>(
    search_roots: impl IntoIterator<Item = impl AsRef<Path>>,
    get_project_config_placement: impl Fn(&Path) -> Option<Placement<PathBuf>>,
) -> IndexMap<ID, FsProjectConfig<ID>>
where
    ID: DeserializedId + Ord + Clone,
{
    let mut projects: IndexMap<ID, FsProjectConfig<ID>> = IndexMap::new();
    let mut parents: Vec<Parent<ID>> = Vec::new();

    for root in search_roots {
        for entry in WalkDir::new(root)
            .follow_links(true)
            .into_iter()
            .filter_map(|entry| entry.ok().and_then(|entry| entry.file_type().is_dir().then_some(entry)))
        {
            let path = entry.path();

            let mut parent = loop {
                if let Some(parent) = parents.pop() {
                    if path.starts_with(&parent.root_dir) {
                        break Some(parent);
                    }
                } else {
                    break None;
                }
            };

            if let Some(project_config) = find_project::<ID>(path, &get_project_config_placement) {
                if let Some(parent) = parent {
                    if let Some(parent_project_config) = projects.get_mut(&parent.id) {
                        parent_project_config.subprojects.insert(project_config.id.clone());
                    }
                    parents.push(parent);
                }

                parent = Some(Parent {
                    id: project_config.id.clone(),
                    root_dir: project_config.root_dir.clone().unwrap_or_default(),
                });
                projects.insert(project_config.id.clone(), project_config);
            }

            if let Some(parent) = parent {
                parents.push(parent);
            }
        }
    }

    projects
}
