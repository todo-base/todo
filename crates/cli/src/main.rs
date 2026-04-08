use clap::Parser;
use todo_app::config::{Config, SourceConfig};

use crate::opts::{AddIssue, CliOpts, Command, InitProject, List, NewProject, Tree};

mod command;
mod display;
mod opts;

fn main() -> anyhow::Result<()> {
    let CliOpts {
        config_file,
        working_mode,
        command,
    } = CliOpts::parse();

    let mut config = Config::load(config_file)?;
    config.update_working_mode(working_mode.local, working_mode.global);

    match command {
        Command::New(NewProject {
            with_manifest,
            with_project_config,
            location,
        }) => {
            let use_manifest = use_manifest(with_manifest, with_project_config, &config.source);
            command::new_project(use_manifest, location, &config)?;
        },
        Command::Init(InitProject {
            with_manifest,
            with_project_config,
            location,
        }) => {
            let use_manifest = use_manifest(with_manifest, with_project_config, &config.source);
            command::init_project(use_manifest, location, &config)?;
        },
        Command::Add(AddIssue { location, order, issue }) => {
            command::add_issue(location, order, issue, &config)?;
        },
        Command::List(List {
            max_steps,
            display,
            location,
            project_location,
        }) => {
            config.update_display_project(display.compact, display.pretty, max_steps);
            command::list(location, project_location, &config)?;
        },
        Command::Tree(Tree {
            max_steps,
            display,
            location,
            project_location,
        }) => {
            config.update_display_project(display.compact, display.pretty, max_steps);
            command::tree(location, project_location, &config)?;
        },
    }

    Ok(())
}

fn use_manifest(with_manifest: bool, with_project_config: bool, config: &SourceConfig) -> bool {
    with_manifest || (!with_project_config && config.use_manifest_file_by_default)
}
