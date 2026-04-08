use std::path::PathBuf;

use clap::{Parser, Subcommand};
use todo_app::issue;
use todo_app::target::Location;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliOpts {
    #[arg(short, long)]
    pub config_file: Option<PathBuf>,

    #[command(flatten)]
    pub working_mode: WorkingMode,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Parser, Clone, Copy, Debug)]
pub struct WorkingMode {
    /// Work in local mode
    #[arg(short, long, conflicts_with = "global")]
    pub local: bool,

    /// Work in global mode
    #[arg(short, long, conflicts_with = "local")]
    pub global: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create new project
    New(NewProject),

    /// Initialize project
    Init(InitProject),

    /// Add issue
    Add(AddIssue),

    /// List issues
    List(List),

    /// Display projects tree
    Tree(Tree),
}

#[derive(Parser, Clone)]
pub struct NewProject {
    /// Create new project with manifest file
    #[arg(short = 'm', long, conflicts_with = "with_project_config")]
    pub with_manifest: bool,

    /// Create new project with project config file
    #[arg(long, conflicts_with = "with_manifest")]
    pub with_project_config: bool,

    /// New project location (new project path by example)
    pub location: String,
}

#[derive(Parser, Clone)]
pub struct InitProject {
    /// Initialize project with manifest file
    #[arg(short = 'm', long, conflicts_with = "with_project_config")]
    pub with_manifest: bool,

    /// Initialize project with project config file
    #[arg(long, conflicts_with = "with_manifest")]
    pub with_project_config: bool,

    /// Initialize project location (exists directory path by example, current directory by default)
    pub location: Option<String>,
}

#[derive(Parser, Clone)]
pub struct AddIssue {
    /// The location of the project to add issue (current directory project by default)
    #[command(flatten)]
    pub location: ProjectLocation,

    /// The issue adding order
    #[command(flatten)]
    pub order: Order,

    /// The name of the issue
    pub issue: String,
}

#[derive(Parser, Clone)]
pub struct List {
    /// Maximum number of steps in the list (project issues and milestones)
    #[arg(short = 's', long)]
    pub max_steps: Option<usize>,

    #[command(flatten)]
    pub display: DisplayMode,

    /// Listing root location (exists directory path by example, current directory by default)
    pub location: Option<String>,

    /// The location of the project to show its list of steps
    #[command(flatten)]
    pub project_location: ProjectLocation,
}

#[derive(Parser, Clone)]
pub struct Tree {
    /// Maximum number of steps in the tree (project issues and milestones)
    #[arg(short = 's', long)]
    pub max_steps: Option<usize>,

    #[command(flatten)]
    pub display: DisplayMode,

    /// Tree from root location (exists directory path by example, current directory by default)
    pub location: Option<String>,

    /// The location of the project to show its tree
    #[command(flatten)]
    pub project_location: ProjectLocation,
}

#[derive(Parser, Clone, Copy, Debug)]
pub struct Order {
    /// Issue will be added to the top of the list
    #[arg(short, long, conflicts_with = "last")]
    pub first: bool,

    /// Issue will be added to the bottom of the list
    #[arg(short, long, conflicts_with = "first")]
    pub last: bool,
}

impl Order {
    pub fn into_order(self) -> Option<issue::Order> {
        if self.first {
            Some(issue::Order::First)
        } else if self.last {
            Some(issue::Order::Last)
        } else {
            None
        }
    }
}

#[derive(Parser, Clone, Copy, Debug)]
pub struct DisplayMode {
    /// Show in compact mode
    #[arg(long, conflicts_with = "pretty")]
    pub compact: bool,

    /// Show in pretty mode
    #[arg(long, conflicts_with = "compact")]
    pub pretty: bool,
}

#[derive(Parser, Clone, Debug)]
pub struct ProjectLocation {
    /// Project location (path, id or name)
    #[arg(short, long, conflicts_with_all = &["project_path", "project_id", "project_name"])]
    pub project: Option<String>,

    /// Project path, using if project location is not specified
    #[arg(long, conflicts_with_all = &["project", "project_id", "project_name"])]
    pub project_path: Option<PathBuf>,

    /// Project id, using if `--project` and `--project-path` is not specified
    #[arg(long, conflicts_with_all = &["project", "project_path", "project_name"])]
    pub project_id: Option<String>,

    /// Project name, using if `--project`, `--project-path` and `--project-id` is not specified
    #[arg(long, conflicts_with_all = &["project", "project_path", "project_id"])]
    pub project_name: Option<String>,
}

impl ProjectLocation {
    pub fn into_location(self) -> Option<Location> {
        let Self {
            project,
            project_path,
            project_id,
            project_name,
        } = self;

        project
            .map(Location::from_unknown)
            .or(project_path.map(Location::Path))
            .or(project_id.map(Location::Id))
            .or(project_name.map(Location::Name))
    }
}
