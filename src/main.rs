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
            workspaces,
        } => commands::start::run(&project, &workspaces),
        Command::DryRun {
            project,
            workspaces,
        } => commands::start::dry_run(&project, &workspaces),
        Command::List => commands::list::run(),
        Command::New { project, force } => commands::new::run(&project, force),
        Command::Validate { project } => commands::validate::run(&project),
    }
}
