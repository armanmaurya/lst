mod args;
mod core;
mod error;
mod output;

use std::path::Path;

use args::Cli;
use error::{LstError, Result};
use output::highlight::print_file_with_highlighting;
use output::printer::{TreeConfig, TreeWriter};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse_cli();
    let path_str = cli.path.as_deref().unwrap_or(".");
    let path = Path::new(path_str);

    // If it's a file, print with syntax highlighting
    if path.is_file() {
        return print_file_with_highlighting(path);
    }

    // If it's a directory (or default), print the tree
    if path.is_dir() {
        let config = TreeConfig {
            path,
            max_depth: cli.effective_depth(),
            show_all: cli.all,
            search_pattern: cli.find.as_deref(),
        };

        if let Some(output_path) = &cli.output {
            TreeWriter::for_file().write_to_file(output_path, &config)
        } else {
            TreeWriter::for_terminal().write_to_terminal(&config)
        }
    } else {
        Err(LstError::InvalidPath(format!(
            "'{}' is not a valid file or directory",
            path_str
        )))
    }
}
