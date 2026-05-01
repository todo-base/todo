use std::borrow::Cow;
use std::path::{Path, PathBuf};

pub use config::ConfigError;
use config::builder::DefaultState;
use config::{ConfigBuilder, Environment};
use config_load::{ConfigLoader, FileLocation, Load};
use indexmap::{IndexMap, IndexSet};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::DeserializeFromStr;
use strum::EnumString;
use todo_tracker_fs::Placement;
use todo_tracker_fs::config::{DeserializedId, FsProjectConfig};
use todo_tracker_fs::file::find_by_name_part;

use crate::issue::Order;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct SearchConfig {
    pub projects: SearchProjectsConfig,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct SearchProjectsConfig {
    #[serde(default = "SearchProjectsConfig::default_enabled")]
    pub enabled: bool,

    pub dirs: Vec<PathBuf>,
}

impl Default for SearchProjectsConfig {
    fn default() -> Self {
        Self {
            enabled: Self::default_enabled(),
            dirs: Default::default(),
        }
    }
}

impl SearchProjectsConfig {
    pub const fn default_enabled() -> bool {
        true
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ListConfig {
    pub projects: ListProjectsConfig,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ListProjectsConfig {
    #[serde(default = "ListProjectsConfig::default_enabled")]
    pub enabled: bool,
}

impl Default for ListProjectsConfig {
    fn default() -> Self {
        Self {
            enabled: Self::default_enabled(),
        }
    }
}

impl ListProjectsConfig {
    pub const fn default_enabled() -> bool {
        true
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct IssueConfig {
    #[serde(default)]
    pub add_order: IssueAddOrder,
}

#[derive(Copy, Clone, Default, Debug, Deserialize, Serialize)]
pub enum IssueAddOrder {
    First,
    #[default]
    Last,
}

impl IssueAddOrder {
    pub fn into_order(self) -> Order {
        match self {
            Self::First => Order::First,
            Self::Last => Order::Last,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ProjectConfig {
    pub path: Option<PathBuf>,

    pub subprojects: IndexSet<String>,
}

impl ProjectConfig {
    pub fn load_fs_project_config<ID>(
        &self,
        project_id: ID,
        config: &SourceConfig,
    ) -> anyhow::Result<Option<FsProjectConfig<ID>>>
    where
        ID: DeserializedId + TryFrom<String>,
        <ID as TryFrom<String>>::Error: Into<anyhow::Error>,
    {
        let Some(mut project_path) = self.path.clone() else {
            return Ok(None);
        };

        if let Some(projects_root_dir) = &config.projects_root_dir
            && project_path.is_relative()
        {
            project_path = projects_root_dir.join(project_path)
        }

        let Some(config_placement) = config.find_project_config_placement(&project_path, None) else {
            return Ok(None);
        };

        let mut config = match FsProjectConfig::load(&config_placement) {
            Ok(config) => config,
            Err(err) => {
                if project_path.is_dir() {
                    FsProjectConfig::new(project_id)
                } else {
                    return Err(err.into());
                }
            },
        };

        config.root_dir = Some(project_path);
        for subproject_id in &self.subprojects {
            let id = ID::try_from(subproject_id.clone()).map_err(Into::into)?;
            config.subprojects.insert(id);
        }

        Ok(Some(config))
    }
}

#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TitleConsist {
    #[default]
    Id,
    Name,
    IdAndName,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct DisplayProjectTitleConfig {
    pub consist: TitleConsist,

    #[serde(default = "DisplayProjectTitleConfig::default_id_before")]
    pub id_before: Option<Cow<'static, str>>,

    #[serde(default = "DisplayProjectTitleConfig::default_id_after")]
    pub id_after: Option<Cow<'static, str>>,

    pub name_before: Option<Cow<'static, str>>,

    pub name_after: Option<Cow<'static, str>>,

    pub id_and_name_before: Option<Cow<'static, str>>,

    #[serde(default = "DisplayProjectTitleConfig::default_id_and_name_separator")]
    pub id_and_name_separator: Option<Cow<'static, str>>,

    pub id_and_name_after: Option<Cow<'static, str>>,

    #[serde(default = "DisplayProjectTitleConfig::default_show_steps_count")]
    pub show_steps_count: bool,
}

impl Default for DisplayProjectTitleConfig {
    fn default() -> Self {
        Self {
            consist: Default::default(),
            id_before: Self::default_id_before(),
            id_after: Self::default_id_after(),
            name_before: None,
            name_after: None,
            id_and_name_before: None,
            id_and_name_separator: Self::default_id_and_name_separator(),
            id_and_name_after: None,
            show_steps_count: Self::default_show_steps_count(),
        }
    }
}

impl DisplayProjectTitleConfig {
    pub const fn default_id_before() -> Option<Cow<'static, str>> {
        Some(Cow::Borrowed("["))
    }

    pub const fn default_id_after() -> Option<Cow<'static, str>> {
        Some(Cow::Borrowed("]"))
    }

    pub const fn default_id_and_name_separator() -> Option<Cow<'static, str>> {
        Some(Cow::Borrowed(" "))
    }

    pub const fn default_show_steps_count() -> bool {
        true
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct DisplayProjectConfig {
    pub title: DisplayProjectTitleConfig,
    pub max_steps: Option<usize>,
    pub show_substeps: bool,
    pub compact: bool,
    pub separate_projects: bool,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct DisplayConfig {
    pub project: DisplayProjectConfig,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct SourceConfig {
    #[serde(default = "SourceConfig::default_use_manifest_file_by_default")]
    pub use_manifest_file_by_default: bool,

    #[serde(default = "SourceConfig::default_manifest_filename_regex", with = "serde_regex")]
    pub manifest_filename_regex: Regex,

    #[serde(default = "SourceConfig::default_manifest_filename_example")]
    pub manifest_filename_example: String,

    #[serde(default = "SourceConfig::default_issues_filename_regex", with = "serde_regex")]
    pub issues_filename_regex: Regex,

    #[serde(default = "SourceConfig::default_issues_filename_example")]
    pub issues_filename_example: String,

    #[serde(default = "SourceConfig::default_project_config_file")]
    pub project_config_file: PathBuf,

    pub projects_root_dir: Option<PathBuf>,
}

impl Default for SourceConfig {
    fn default() -> Self {
        Self {
            use_manifest_file_by_default: Self::default_use_manifest_file_by_default(),
            manifest_filename_regex: Self::default_manifest_filename_regex(),
            manifest_filename_example: Self::default_manifest_filename_example(),
            issues_filename_regex: Self::default_issues_filename_regex(),
            issues_filename_example: Self::default_issues_filename_example(),
            project_config_file: Self::default_project_config_file(),
            projects_root_dir: None,
        }
    }
}

impl SourceConfig {
    pub const fn default_use_manifest_file_by_default() -> bool {
        false
    }

    pub fn default_manifest_filename_regex() -> Regex {
        Regex::new("(.*)\\.manifest\\.md$").expect("regex mus be correct")
    }

    pub fn default_manifest_filename_example() -> String {
        "example.manifest.md".into()
    }

    pub fn default_issues_filename_regex() -> Regex {
        Regex::new("^TODO\\.md$").expect("regex mus be correct")
    }

    pub fn default_issues_filename_example() -> String {
        "TODO.md".into()
    }

    pub fn default_project_config_file() -> PathBuf {
        "Project.toml".into()
    }

    pub fn manifest_example_project_name(&self) -> &str {
        self.manifest_filename_regex
            .captures(&self.manifest_filename_example)
            .and_then(|captures| captures.get(1))
            .map(|name| name.as_str())
            .unwrap_or_default()
    }

    pub fn project_manifest_file_name(&self, project_name: impl AsRef<str>) -> String {
        let example_project_name = self.manifest_example_project_name();
        if example_project_name.is_empty() {
            self.manifest_filename_example.clone()
        } else {
            self.manifest_filename_example
                .replace(example_project_name, project_name.as_ref())
        }
    }

    pub fn make_manifest_file_path(&self, root_dir: impl AsRef<Path>, project_name: Option<&str>) -> PathBuf {
        let root_dir = root_dir.as_ref();
        if let Some(name) = project_name {
            root_dir.join(self.project_manifest_file_name(name))
        } else {
            let manifest_file_name_part = self.project_manifest_file_name("");
            find_by_name_part(root_dir, &manifest_file_name_part)
                .unwrap_or_else(|| root_dir.join(manifest_file_name_part))
        }
    }

    pub fn issues_example_project_name(&self) -> &str {
        self.issues_filename_regex
            .captures(&self.issues_filename_example)
            .and_then(|captures| captures.get(1))
            .map(|name| name.as_str())
            .unwrap_or_default()
    }

    pub fn project_issues_file_name(&self, project_name: impl AsRef<str>) -> String {
        let example_project_name = self.issues_example_project_name();
        if example_project_name.is_empty() {
            self.issues_filename_example.clone()
        } else {
            self.issues_filename_example
                .replace(example_project_name, project_name.as_ref())
        }
    }

    pub fn make_issues_file_path(&self, root_dir: impl AsRef<Path>, project_name: Option<&str>) -> PathBuf {
        root_dir
            .as_ref()
            .join(self.project_issues_file_name(project_name.unwrap_or_default()))
    }

    pub fn find_project_config_placement(
        &self,
        root_dir: impl AsRef<Path>,
        project_name: Option<&str>,
    ) -> Option<Placement<PathBuf>> {
        let project_config_file = root_dir.as_ref().join(&self.project_config_file);
        if project_config_file.exists() {
            Some(Placement::WholeFile(project_config_file))
        } else {
            let project_manifest_file = self.make_manifest_file_path(root_dir, project_name);
            project_manifest_file
                .exists()
                .then_some(Placement::CodeBlockInFile(project_manifest_file))
        }
    }

    pub fn find_issues_placement(
        &self,
        root_dir: impl AsRef<Path>,
        project_name: Option<&str>,
    ) -> Option<Placement<PathBuf>> {
        let issues_file = self.make_issues_file_path(root_dir.as_ref(), project_name);
        if issues_file.exists() {
            Some(Placement::WholeFile(issues_file))
        } else {
            let manifest_file = self.make_manifest_file_path(root_dir, project_name);
            manifest_file
                .exists()
                .then_some(Placement::CodeBlockInFile(manifest_file))
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, EnumString, DeserializeFromStr)]
#[strum(ascii_case_insensitive)]
#[serde(rename_all = "lowercase")]
pub enum WorkingMode {
    #[default]
    Local,
    Global,
}

impl WorkingMode {
    pub fn is_local(&self) -> bool {
        matches!(self, WorkingMode::Local)
    }

    pub fn is_global(&self) -> bool {
        matches!(self, WorkingMode::Global)
    }
}

pub const DEFAULT_CONFIG_FILE_NAME: &str = "todo.toml";
pub const ROOT_CONFIG_ENV_KEY: &str = "TODO_ROOT_CONFIG";

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub working_mode: WorkingMode,

    pub source: SourceConfig,

    pub display: DisplayConfig,

    pub search: SearchConfig,

    pub list: ListConfig,

    pub issue: IssueConfig,

    pub project: IndexMap<String, ProjectConfig>,
}

impl Config {
    pub fn update_working_mode(&mut self, force_local: bool, force_global: bool) {
        if force_local != force_global {
            self.working_mode = if force_local {
                WorkingMode::Local
            } else {
                WorkingMode::Global
            };
        }
    }

    pub fn update_display_project(&mut self, force_compact: bool, force_pretty: bool, force_max_steps: Option<usize>) {
        if let Some(max_steps) = force_max_steps {
            self.display.project.max_steps = Some(max_steps);
        }

        if force_compact != force_pretty {
            self.display.project.compact = force_compact && !force_pretty;
        }
    }

    pub fn load(config_file: Option<PathBuf>) -> config_load::Result<Self> {
        ConfigLoader::default()
            .add(
                FileLocation::first_some_path()
                    .from_env(ROOT_CONFIG_ENV_KEY)
                    .from_home_exists(Path::new(".todo").join(DEFAULT_CONFIG_FILE_NAME)),
            )
            .exclude_not_exists()
            .add(
                FileLocation::first_some_path()
                    .from_file(config_file)
                    .from_cwd_and_parents_exists(DEFAULT_CONFIG_FILE_NAME),
            )
            .load()
    }
}

impl Load for Config {
    fn load(config_builder: ConfigBuilder<DefaultState>) -> config_load::Result<Self> {
        // Add in settings from the environment (with a prefix of TODO)
        // Eg. `TODO_CONFIG_HTTP_PORT=8090 ` would set the `http.port` param
        let config = config_builder
            .add_source(Environment::with_prefix("TODO_CONFIG").separator("_"))
            .build()?;
        config.try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct Wrap {
        working_mode: WorkingMode,
    }

    #[test]
    fn working_mode_deserialize() {
        for raw in ["local", "Local", "LOCAL", "LoCaL"] {
            let toml_src = format!("working_mode = \"{raw}\"");
            let config: Wrap = toml::from_str(&toml_src).unwrap();
            assert_eq!(config.working_mode, WorkingMode::Local, "input: {raw:?}");
        }

        for raw in ["global", "Global", "GLOBAL", "GloBal"] {
            let toml_src = format!("working_mode = \"{raw}\"");
            let config: Wrap = toml::from_str(&toml_src).unwrap();
            assert_eq!(config.working_mode, WorkingMode::Global, "input: {raw:?}");
        }
    }

    #[test]
    fn working_mode_deserialize_unknown_variant() {
        let err = toml::from_str::<Wrap>("working_mode = \"remote\"").unwrap_err();
        assert!(
            err.to_string().to_lowercase().contains("remote") || err.to_string().contains("working_mode"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn working_mode_serialize() {
        let local = toml::to_string(&Wrap {
            working_mode: WorkingMode::Local,
        })
        .unwrap();
        assert_eq!(local.trim(), "working_mode = \"local\"");

        let global = toml::to_string(&Wrap {
            working_mode: WorkingMode::Global,
        })
        .unwrap();
        assert_eq!(global.trim(), "working_mode = \"global\"");
    }
}
