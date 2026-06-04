mod cli;
mod cmux;
mod commands;
mod completion;
mod config;
mod formatting;
mod layout;
mod paths;
mod templates;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use cli::{Cli, Command};

fn main() -> Result<()> {
    if completion::try_complete_project_or_workspace()? {
        return Ok(());
    }
    clap_complete::CompleteEnv::with_factory(Cli::command).complete();

    let cli = Cli::parse();

    match cli.command {
        Command::Start {
            project,
            group,
            workspaces,
            ..
        } => commands::start::run(&project, &workspaces, group),
        Command::DryRun {
            project,
            group,
            workspaces,
            ..
        } => commands::start::dry_run(&project, &workspaces, group),
        Command::List => commands::list::run(),
        Command::New { project, force } => commands::new::run(&project, force),
        Command::Validate { project } => commands::validate::run(&project),
    }
}
