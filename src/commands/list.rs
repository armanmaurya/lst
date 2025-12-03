use std::path::Path;

use crate::error::{LstError, Result};
use crate::output::highlight::print_file_with_highlighting;
use crate::output::printer::{TreeConfig, TreeWriter};

pub fn run(path: &Path, show_all: bool, max_depth: usize, output: Option<&str>, json: bool) -> Result<()> {
    // If it's a file, print with syntax highlighting
    if path.is_file() {
        return print_file_with_highlighting(path);
    }

    if path.is_dir() {
        let config = TreeConfig {
            path,
            max_depth,
            show_all,
            search_pattern: None,
            spinner_stop: None,
            json_output: json,
        };

        if let Some(output_path) = output {
            TreeWriter::for_file().write_to_file(output_path, &config)
        } else {
            TreeWriter::for_terminal().write_to_terminal(&config)
        }
    } else {
        Err(LstError::InvalidPath(format!(
            "'{}' is not a valid file or directory",
            path.display()
        )))
    }
}
