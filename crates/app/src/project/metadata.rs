use std::path::{Path, PathBuf};
use std::{env, io};

use anyhow::anyhow;
use todo_lib::id::HashedId;
use todo_tracker_fs::config::{DeserializedId, FsProjectConfig};
use todo_tracker_fs::Placement;

use crate::config::SourceConfig;
use crate::target::Location;

pub enum ProjectData<ID: HashedId = String> {
    Fs(FsProjectMetadata<ID>),
}

#[derive(Debug)]
pub struct FsProjectMetadata<ID: HashedId> {
    id: Option<ID>,
    name: String,
    root_dir: PathBuf,
    is_current_dir_parent: bool,
    config_placement: Option<Placement<PathBuf>>,
    config: Option<FsProjectConfig<ID>>,
}

impl<ID: HashedId> Default for FsProjectMetadata<ID> {
    fn default() -> Self {
        Self {
            id: None,
            name: Default::default(),
            root_dir: Default::default(),
            is_current_dir_parent: false,
            config_placement: None,
            config: None,
        }
    }
}

impl<ID: HashedId> FsProjectMetadata<ID> {
    pub fn with_id(mut self, id: ID) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn with_name_from_path(mut self, path: impl AsRef<Path>) -> Option<Self> {
        path.as_ref().file_name().map(|name| {
            self.name = name.to_string_lossy().into();
            self
        })
    }

    pub fn with_root_dir(mut self, root_dir: impl Into<PathBuf>) -> Self {
        self.root_dir = root_dir.into();
        self
    }

    pub fn with_root_dir_from_current_dir_and_name(mut self) -> io::Result<Self> {
        self.root_dir = env::current_dir()?.join(&self.name);
        self.is_current_dir_parent = true;
        Ok(self)
    }

    pub fn with_root_dir_from_path(mut self, path: impl Into<PathBuf> + AsRef<Path>) -> io::Result<Self> {
        self.is_current_dir_parent = path.as_ref().iter().count() == 1;
        self.root_dir = if path.as_ref().is_relative() {
            env::current_dir()?.join(path)
        } else {
            path.into()
        };
        Ok(self)
    }

    pub fn with_config_placement(mut self, placement: Placement<PathBuf>) -> Self {
        self.config_placement = Some(placement);
        self
    }

    pub fn with_config_placement_maybe(mut self, placement: Option<Placement<PathBuf>>) -> Self {
        self.config_placement = placement;
        self
    }

    pub fn with_config(mut self, config: FsProjectConfig<ID>) -> Self {
        self.config = Some(config);
        self
    }

    pub fn id(&self) -> Option<&ID> {
        self.config.as_ref().map(|config| &config.id).or(self.id.as_ref())
    }

    pub fn name(&self) -> &str {
        self.config
            .as_ref()
            .and_then(|config| config.name.as_deref())
            .unwrap_or(&self.name)
    }

    pub fn root_dir(&self) -> &Path {
        self.config
            .as_ref()
            .and_then(|config| config.root_dir.as_deref())
            .unwrap_or(&self.root_dir)
    }

    pub fn is_current_dir_parent(&self) -> bool {
        self.is_current_dir_parent
    }

    pub fn set_config_placement_if_exist(&mut self, config: &SourceConfig) -> bool {
        self.config_placement = config.find_project_config_placement(&self.root_dir, Some(&self.name));
        self.config_placement.is_some()
    }

    #[allow(clippy::type_complexity)]
    pub fn destructure(
        self,
    ) -> (
        Option<ID>,
        String,
        PathBuf,
        bool,
        Option<Placement<PathBuf>>,
        Option<FsProjectConfig<ID>>,
    ) {
        let Self {
            id,
            name,
            root_dir,
            is_current_dir_parent,
            config_placement,
            config,
        } = self;
        (id, name, root_dir, is_current_dir_parent, config_placement, config)
    }
}

impl<ID: HashedId + Default> FsProjectMetadata<ID> {
    pub fn into_config(self) -> (FsProjectConfig<ID>, Option<Placement<PathBuf>>) {
        let Self {
            id,
            name,
            root_dir,
            is_current_dir_parent: _,
            config_placement,
            config,
        } = self;
        let mut config = config.unwrap_or_else(|| FsProjectConfig::new(id.unwrap_or_default()));

        if config.name.is_none() {
            config.name = Some(name)
        }

        if config.root_dir.is_none() {
            config.root_dir = Some(root_dir)
        }

        (config, config_placement)
    }
}

impl<ID: HashedId + ToString> FsProjectMetadata<ID> {
    pub fn from_location(location: Location<ID>) -> anyhow::Result<Self> {
        Ok(match location {
            Location::Path(path) => Self::default()
                .with_name_from_path(&path)
                .ok_or_else(|| anyhow!("path `{}` is not containing a directory name", path.display()))?
                .with_root_dir_from_path(path)?,
            Location::Id(id) => {
                let name = id.to_string();

                Self::default()
                    .with_id(id)
                    .with_root_dir_from_path(&name)?
                    .with_name(name)
            },
            Location::Name(name) => Self::default().with_root_dir_from_path(&name)?.with_name(name),
            Location::IdOrName(id_or_name) => Self::default()
                .with_root_dir_from_path(&id_or_name)?
                .with_name(id_or_name),
        })
    }

    pub fn new(location: Location<ID>, config: &SourceConfig, use_manifest: bool) -> anyhow::Result<Self> {
        let mut data = Self::from_location(location)?;

        if use_manifest {
            let file_name = config.project_manifest_file_name(&data.name);

            data.config_placement
                .replace(Placement::CodeBlockInFile(data.root_dir.join(file_name)));
        } else {
            data.config_placement
                .replace(Placement::WholeFile(data.root_dir.join(&config.project_config_file)));
        }

        Ok(data)
    }
}

impl<ID: DeserializedId + Clone> FsProjectMetadata<ID> {
    pub fn update_from_config(&mut self) -> anyhow::Result<()> {
        if let Some(source) = &self.config_placement {
            let project_config = FsProjectConfig::<ID>::load(source)?;

            if let Some(name) = &project_config.name {
                self.name = name.clone()
            }
            if let Some(path) = &project_config.root_dir {
                self.root_dir = path.clone()
            }
            self.id = Some(project_config.id.clone());
            self.config = Some(project_config);
        }

        Ok(())
    }
}
