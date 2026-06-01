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
    clap_complete::CompleteEnv::with_factory(Cli::command).complete();

    let cli = Cli::parse();

    match cli.command {
        Command::Start {
            project,
            no_group,
            workspaces,
        } => commands::start::run(&project, &workspaces, !no_group),
        Command::DryRun {
            project,
            no_group,
            workspaces,
        } => commands::start::dry_run(&project, &workspaces, !no_group),
        Command::List => commands::list::run(),
        Command::New { project, force } => commands::new::run(&project, force),
        Command::Validate { project } => commands::validate::run(&project),
    }
}
