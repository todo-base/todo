use std::path::PathBuf;

use anyhow::Context;
use clap::{Args, ValueEnum};
use todo_app::config::{self, Config};

#[derive(Args, Debug, Default)]
pub struct ConfigOpts {
    #[clap(flatten)]
    pub working_mode: WorkingMode,
    #[clap(flatten)]
    pub source: SourceOpts,
    #[clap(flatten)]
    pub display: DisplayOpts,
    #[clap(flatten)]
    pub search: SearchOpts,
    #[clap(flatten)]
    pub list: ListOpts,
    #[clap(flatten)]
    pub issue: IssueOpts,
}

impl ConfigOpts {
    pub fn override_config(self, config: Config) -> anyhow::Result<Config> {
        let ConfigOpts {
            working_mode,
            source,
            display,
            search,
            list,
            issue,
        } = self;
        let mayby_working_mode = working_mode.into_config_option();

        Ok(Config {
            working_mode: mayby_working_mode.unwrap_or(config.working_mode),
            source: source.override_config(config.source)?,
            display: display.override_config(config.display)?,
            search: search.override_config(config.search)?,
            list: list.override_config(config.list)?,
            issue: issue.override_config(config.issue)?,
            project: config.project,
        })
    }
}

#[derive(Args, Clone, Copy, Debug, Default)]
#[group(multiple = false)]
pub struct WorkingMode {
    /// Work in local mode
    #[arg(short, long)]
    pub local: bool,
    /// Work in global mode
    #[arg(short, long)]
    pub global: bool,
}

impl WorkingMode {
    pub fn into_config_option(self) -> Option<config::WorkingMode> {
        if self.local == self.global {
            return None;
        }

        Some(if self.local {
            config::WorkingMode::Local
        } else {
            config::WorkingMode::Global
        })
    }
}

#[derive(Args, Debug, Default)]
pub struct SourceOpts {
    #[arg(long)]
    pub use_manifest_file_by_default: Option<bool>,
    #[arg(long)]
    pub manifest_filename_regex: Option<String>,
    #[arg(long)]
    pub manifest_filename_example: Option<String>,
    #[arg(long)]
    pub issues_filename_regex: Option<String>,
    #[arg(long)]
    pub issues_filename_example: Option<String>,
    #[arg(long)]
    pub project_config_file: Option<PathBuf>,
    #[arg(long)]
    pub projects_root_dir: Option<PathBuf>,
}

impl SourceOpts {
    pub fn override_config(self, config: config::SourceConfig) -> anyhow::Result<config::SourceConfig> {
        let Self {
            use_manifest_file_by_default,
            manifest_filename_regex,
            manifest_filename_example,
            issues_filename_regex,
            issues_filename_example,
            project_config_file,
            projects_root_dir,
        } = self;

        Ok(config::SourceConfig {
            use_manifest_file_by_default: use_manifest_file_by_default.unwrap_or(config.use_manifest_file_by_default),
            manifest_filename_regex: manifest_filename_regex
                .map(TryInto::try_into)
                .transpose()
                .context("parse manifest filename regex")?
                .unwrap_or(config.manifest_filename_regex),
            manifest_filename_example: manifest_filename_example.unwrap_or(config.manifest_filename_example),
            issues_filename_regex: issues_filename_regex
                .map(TryInto::try_into)
                .transpose()
                .context("parse issues filename regex")?
                .unwrap_or(config.issues_filename_regex),
            issues_filename_example: issues_filename_example.unwrap_or(config.issues_filename_example),
            project_config_file: project_config_file.unwrap_or(config.project_config_file),
            projects_root_dir: projects_root_dir.or(config.projects_root_dir),
        })
    }
}

#[derive(Args, Debug, Default)]
pub struct DisplayOpts {
    #[clap(flatten)]
    pub project: DisplayProjectOpts,
}

impl DisplayOpts {
    pub fn override_config(self, config: config::DisplayConfig) -> anyhow::Result<config::DisplayConfig> {
        let Self { project } = self;

        Ok(config::DisplayConfig {
            project: project.override_config(config.project)?,
        })
    }
}

#[derive(Args, Debug, Default)]
pub struct DisplayProjectOpts {
    #[clap(flatten)]
    pub project_title: DisplayProjectTitleOpts,
    #[arg(long)]
    pub project_max_steps: Option<usize>,
    #[arg(long)]
    pub project_show_substeps: Option<bool>,
    #[arg(long)]
    pub project_compact: Option<bool>,
    #[arg(long)]
    pub separate_projects: Option<bool>,
}

impl DisplayProjectOpts {
    pub fn override_config(self, config: config::DisplayProjectConfig) -> anyhow::Result<config::DisplayProjectConfig> {
        let Self {
            project_title,
            project_max_steps,
            project_show_substeps,
            project_compact,
            separate_projects,
        } = self;

        Ok(config::DisplayProjectConfig {
            title: project_title.override_config(config.title)?,
            max_steps: project_max_steps.or(config.max_steps),
            show_substeps: project_show_substeps.unwrap_or(config.show_substeps),
            compact: project_compact.unwrap_or(config.compact),
            separate_projects: separate_projects.unwrap_or(config.separate_projects),
        })
    }
}

#[derive(Args, Debug, Default)]
pub struct DisplayProjectTitleOpts {
    #[arg(long)]
    pub project_title_consist: Option<TitleConsist>,
    #[arg(long)]
    pub project_title_id_before: Option<String>,
    #[arg(long)]
    pub project_title_id_after: Option<String>,
    #[arg(long)]
    pub project_title_name_before: Option<String>,
    #[arg(long)]
    pub project_title_name_after: Option<String>,
    #[arg(long)]
    pub project_title_id_and_name_before: Option<String>,
    #[arg(long)]
    pub project_title_id_and_name_separator: Option<String>,
    #[arg(long)]
    pub project_title_id_and_name_after: Option<String>,
    #[arg(long)]
    pub project_title_show_steps_count: Option<bool>,
}

impl DisplayProjectTitleOpts {
    pub fn override_config(
        self,
        config: config::DisplayProjectTitleConfig,
    ) -> anyhow::Result<config::DisplayProjectTitleConfig> {
        let Self {
            project_title_consist,
            project_title_id_before,
            project_title_id_after,
            project_title_name_before,
            project_title_name_after,
            project_title_id_and_name_before,
            project_title_id_and_name_separator,
            project_title_id_and_name_after,
            project_title_show_steps_count,
        } = self;

        Ok(config::DisplayProjectTitleConfig {
            consist: project_title_consist.map(Into::into).unwrap_or(config.consist),
            id_before: project_title_id_before.map(Into::into).or(config.id_before),
            id_after: project_title_id_after.map(Into::into).or(config.id_after),
            name_before: project_title_name_before.map(Into::into).or(config.name_before),
            name_after: project_title_name_after.map(Into::into).or(config.name_after),
            id_and_name_before: project_title_id_and_name_before
                .map(Into::into)
                .or(config.id_and_name_before),
            id_and_name_separator: project_title_id_and_name_separator
                .map(Into::into)
                .or(config.id_and_name_separator),
            id_and_name_after: project_title_id_and_name_after
                .map(Into::into)
                .or(config.id_and_name_after),
            show_steps_count: project_title_show_steps_count.unwrap_or(config.show_steps_count),
        })
    }
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum TitleConsist {
    Id,
    Name,
    IdAndName,
}

impl From<TitleConsist> for config::TitleConsist {
    fn from(value: TitleConsist) -> Self {
        match value {
            TitleConsist::Id => config::TitleConsist::Id,
            TitleConsist::Name => config::TitleConsist::Name,
            TitleConsist::IdAndName => config::TitleConsist::IdAndName,
        }
    }
}

#[derive(Args, Debug, Default)]
pub struct SearchOpts {
    #[clap(flatten)]
    pub projects: SearchProjectsOpts,
}

impl SearchOpts {
    fn override_config(self, config: config::SearchConfig) -> anyhow::Result<config::SearchConfig> {
        let Self { projects } = self;

        Ok(config::SearchConfig {
            projects: projects.override_config(config.projects)?,
        })
    }
}

#[derive(Args, Debug, Default)]
pub struct SearchProjectsOpts {
    #[arg(long)]
    pub search_projects_enabled: Option<bool>,
    #[arg(long)]
    pub search_projects_dir: Vec<PathBuf>,
}

impl SearchProjectsOpts {
    fn override_config(self, config: config::SearchProjectsConfig) -> anyhow::Result<config::SearchProjectsConfig> {
        let Self {
            search_projects_enabled,
            search_projects_dir,
        } = self;

        let mut dirs = config.dirs;
        dirs.extend(search_projects_dir);

        Ok(config::SearchProjectsConfig {
            enabled: search_projects_enabled.unwrap_or(config.enabled),
            dirs,
        })
    }
}

#[derive(Args, Debug, Default)]
pub struct ListOpts {
    #[clap(flatten)]
    pub projects: ListProjectsOpts,
}

impl ListOpts {
    fn override_config(self, config: config::ListConfig) -> anyhow::Result<config::ListConfig> {
        let Self { projects } = self;

        Ok(config::ListConfig {
            projects: projects.override_config(config.projects)?,
        })
    }
}

#[derive(Args, Debug, Default)]
pub struct ListProjectsOpts {
    #[arg(long)]
    pub list_projects_enabled: Option<bool>,
}

impl ListProjectsOpts {
    fn override_config(self, config: config::ListProjectsConfig) -> anyhow::Result<config::ListProjectsConfig> {
        let Self { list_projects_enabled } = self;

        Ok(config::ListProjectsConfig {
            enabled: list_projects_enabled.unwrap_or(config.enabled),
        })
    }
}

#[derive(Args, Debug, Default)]
pub struct IssueOpts {
    #[arg(long)]
    pub issue_add_order: Option<IssueAddOrder>,
}

impl IssueOpts {
    fn override_config(self, config: config::IssueConfig) -> anyhow::Result<config::IssueConfig> {
        let Self { issue_add_order } = self;

        Ok(config::IssueConfig {
            add_order: issue_add_order.map(Into::into).unwrap_or(config.add_order),
        })
    }
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum IssueAddOrder {
    First,
    Last,
}

impl From<IssueAddOrder> for config::IssueAddOrder {
    fn from(value: IssueAddOrder) -> Self {
        match value {
            IssueAddOrder::First => config::IssueAddOrder::First,
            IssueAddOrder::Last => config::IssueAddOrder::Last,
        }
    }
}
