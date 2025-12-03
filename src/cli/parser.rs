use std::path::Path;

use crate::commands;
use crate::error::Result;

use super::args::{effective_depth, Cli, Commands};

/// Entry point for CLI execution: parse args and dispatch to subcommands.
pub fn run_cli() -> Result<()> {
    let cli = Cli::parse_cli();
    let path_str = cli.path.as_deref().unwrap_or(".");
    let path = Path::new(path_str);

    match cli.command {
        Some(Commands::Search { pattern }) => {
            let max_depth = effective_depth(cli.depth);
            commands::search::run(&pattern, path, cli.all, max_depth, cli.output.as_deref(), cli.json)
        }
        None => {
            // Default behavior: list current directory with global flags
            let max_depth = effective_depth(cli.depth);
            commands::list::run(path, cli.all, max_depth, cli.output.as_deref(), cli.json)
        }
    }
}
