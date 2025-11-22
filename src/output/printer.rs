use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::DirEntry;

use super::formatter::{
    format_directory_name, format_file_name, format_file_size, 
    format_size_colored, TreeFormatter,
};
use super::terminal::CharacterSet;
use crate::core::search::{build_search_filter, should_print_entry};
use crate::core::tree::collect_entries;
use crate::error::Result;

/// Configuration for tree printing
pub struct TreeConfig<'a> {
    pub path: &'a Path,
    pub max_depth: usize,
    pub show_all: bool,
    pub search_pattern: Option<&'a str>,
}

/// Tree writer that handles directory tree output
pub struct TreeWriter {
    use_color: bool,
}

impl TreeWriter {
    /// Create a new TreeWriter for terminal output (with color)
    pub fn for_terminal() -> Self {
        Self { use_color: true }
    }

    /// Create a new TreeWriter for file output (no color)
    pub fn for_file() -> Self {
        Self { use_color: false }
    }

    /// Write the tree to the provided writer
    pub fn write<W: Write>(&self, writer: &mut W, config: &TreeConfig) -> Result<()> {
        let entries = collect_entries(config.path, config.max_depth, config.show_all);
        let show_dirs = if let Some(pattern) = config.search_pattern {
            build_search_filter(&entries, pattern)
        } else {
            HashSet::new()
        };

        print_tree(writer, &entries, config.search_pattern, &show_dirs, self.use_color)?;
        Ok(())
    }

    /// Write tree to a file with a header
    pub fn write_to_file(&self, output_path: &str, config: &TreeConfig) -> Result<()> {
        let mut file = std::fs::File::create(output_path)?;
        writeln!(file, ".")?;
        self.write(&mut file, config)?;
        println!("Tree exported to {}", output_path);
        Ok(())
    }

    /// Write tree to terminal (stdout)
    pub fn write_to_terminal(&self, config: &TreeConfig) -> Result<()> {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        self.write(&mut handle, config)
    }
}

/// Print a single directory entry line with proper tree formatting
fn print_entry_line<W: Write>(
    writer: &mut W,
    entry: &DirEntry,
    indent: &str,
    use_color: bool,
) -> std::io::Result<()> {
    let file_name = entry.file_name().to_string_lossy();

    if entry.file_type().is_dir() {
        let formatted_name = format_directory_name(&file_name, use_color);
        writeln!(writer, "{}{}/", indent, formatted_name)
    } else {
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        let human_size = format_file_size(size);
        let formatted_name = format_file_name(&file_name, use_color);
        let formatted_size = format_size_colored(&human_size, use_color);
        writeln!(writer, "{}{} ({})", indent, formatted_name, formatted_size)
    }
}

/// Print the complete directory tree with proper branching
pub fn print_tree<W: Write>(
    writer: &mut W,
    entries: &[DirEntry],
    search_pattern: Option<&str>,
    show_dirs: &HashSet<PathBuf>,
    use_color: bool,
) -> std::io::Result<()> {
    if entries.is_empty() {
        return Ok(());
    }

    // Use ASCII for file output (no color), Unicode for terminal if supported
    let charset = if use_color {
        CharacterSet::detect()
    } else {
        CharacterSet::Ascii
    };

    let formatter = TreeFormatter::with_charset(charset);
    
    // Filter entries based on search pattern first
    let filtered_entries: Vec<&DirEntry> = entries
        .iter()
        .filter(|entry| should_print_entry(entry, search_pattern, show_dirs))
        .collect();

    if filtered_entries.is_empty() {
        return Ok(());
    }

    // Compute last-child states for all entries
    let entries_vec: Vec<DirEntry> = filtered_entries.iter().map(|&e| e.clone()).collect();
    let last_child_map = formatter.compute_last_child_map(&entries_vec);

    // Print each entry with proper indentation
    for (idx, entry) in entries_vec.iter().enumerate() {
        let depth = entry.depth();
        let is_last = last_child_map.get(idx).map(|v| v.as_slice()).unwrap_or(&[]);
        let indent = formatter.generate_indent(depth, is_last);
        
        print_entry_line(writer, entry, &indent, use_color)?;
    }
    
    Ok(())
}
