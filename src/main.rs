mod cli;
mod cmux;
mod commands;
mod config;
mod formatting;
mod layout;
mod paths;
mod templates;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
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
