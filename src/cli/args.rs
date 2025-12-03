use clap::{Parser, Subcommand};

/// Command-line arguments for lst
#[derive(Parser, Debug)]
#[command(name = "lst", about = "A fast, colorful CLI tool for listing directories")]
pub struct Cli {
    /// Path to inspect (file or directory)
    #[arg(global = true)]
    pub path: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Show hidden files and directories
    #[arg(short, long, global = true, default_value_t = false)]
    pub all: bool,

    /// Max depth of traversal (0 for unlimited)
    #[arg(short, long, global = true, default_value_t = 0)]
    pub depth: usize,

    /// Export tree to a file (plain text, no color)
    #[arg(short, long, global = true)]
    pub output: Option<String>,

    /// Output format as JSON
    #[arg(short, long, global = true, default_value_t = false)]
    pub json: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Search for files/directories by name
    Search {
        /// Pattern to search for (case-insensitive)
        pattern: String,
    },
}

impl Cli {
    /// Parse CLI arguments from environment
    pub fn parse_cli() -> Self {
        Self::parse()
    }
}

/// Helper to compute effective depth
pub fn effective_depth(depth: usize) -> usize {
    if depth == 0 {
        usize::MAX
    } else {
        depth
    }
}
