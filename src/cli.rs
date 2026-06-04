use clap::{Parser, Subcommand};
use clap_complete::engine::ArgValueCompleter;

#[derive(Debug, Parser)]
#[command(name = "cmuxinator")]
#[command(about = "Start cmux projects from tmuxinator-style YAML files")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a new cmux window and open all or selected project workspaces.
    Start {
        /// Project name from ~/.config/cmuxinator/<project>.yml.
        #[arg(add = ArgValueCompleter::new(crate::completion::project_completer))]
        project: String,
        /// Create a cmux workspace group for the project workspaces.
        #[arg(long, conflicts_with = "no_group")]
        group: bool,
        /// Deprecated no-op: grouping is disabled by default.
        #[arg(long, hide = true)]
        no_group: bool,
        /// Optional workspace names to start from the project.
        #[arg(add = ArgValueCompleter::new(crate::completion::workspace_completer))]
        workspaces: Vec<String>,
    },

    /// Print generated cmux calls and layout JSON without changing cmux.
    DryRun {
        /// Project name from ~/.config/cmuxinator/<project>.yml.
        #[arg(add = ArgValueCompleter::new(crate::completion::project_completer))]
        project: String,
        /// Include workspace-group commands in the dry-run output.
        #[arg(long, conflicts_with = "no_group")]
        group: bool,
        /// Deprecated no-op: grouping is disabled by default.
        #[arg(long, hide = true)]
        no_group: bool,
        /// Optional workspace names to dry-run from the project.
        #[arg(add = ArgValueCompleter::new(crate::completion::workspace_completer))]
        workspaces: Vec<String>,
    },

    /// List available cmuxinator projects.
    List,

    /// Scaffold a project YAML file.
    New {
        /// Project name to create under ~/.config/cmuxinator/.
        project: String,
        /// Overwrite an existing project file.
        #[arg(long, short)]
        force: bool,
    },

    /// Validate a project file without changing cmux.
    Validate {
        /// Project name from ~/.config/cmuxinator/<project>.yml.
        #[arg(add = ArgValueCompleter::new(crate::completion::project_completer))]
        project: String,
    },
}
