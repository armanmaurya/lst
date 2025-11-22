use clap::Parser;

/// Command-line arguments for lst
#[derive(Parser, Debug)]
#[command(name = "lst", about = "A fast, colorful CLI tool for listing directories")]
pub struct Cli {
    /// Path to inspect (file or directory)
    pub path: Option<String>,

    /// Show hidden files and directories
    #[arg(short, long, default_value_t = false)]
    pub all: bool,

    /// Max depth of traversal (0 for unlimited)
    #[arg(short, long, default_value_t = 1)]
    pub depth: usize,

    /// Search for files/directories by name (case-insensitive)
    #[arg(short = 'f', long)]
    pub find: Option<String>,

    /// Export tree to a file (plain text, no color)
    #[arg(short, long)]
    pub output: Option<String>,
}

impl Cli {
    /// Parse CLI arguments from environment
    pub fn parse_cli() -> Self {
        Self::parse()
    }

    /// Get the effective max depth (handles special cases)
    pub fn effective_depth(&self) -> usize {
        if self.depth == 0 || (self.depth == 1 && self.find.is_some()) {
            usize::MAX
        } else {
            self.depth
        }
    }
}
