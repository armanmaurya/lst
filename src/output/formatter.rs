use colored::Colorize;
use humansize::{format_size, DECIMAL};
use walkdir::DirEntry;

use super::terminal::CharacterSet;

/// Format a file size in human-readable format
pub fn format_file_size(size: u64) -> String {
    format_size(size, DECIMAL)
}

/// Format a directory name with optional color
pub fn format_directory_name(name: &str, use_color: bool) -> String {
    if use_color {
        name.blue().to_string()
    } else {
        name.to_string()
    }
}

/// Format a file name with optional color
pub fn format_file_name(name: &str, use_color: bool) -> String {
    if use_color {
        name.green().to_string()
    } else {
        name.to_string()
    }
}

/// Format a file size with optional color
pub fn format_size_colored(size: &str, use_color: bool) -> String {
    if use_color {
        size.yellow().to_string()
    } else {
        size.to_string()
    }
}

/// Tree formatter with efficient single-pass rendering
pub struct TreeFormatter {
    charset: CharacterSet,
}

impl TreeFormatter {
    /// Create a new tree formatter with auto-detected character set
    pub fn new() -> Self {
        Self {
            charset: CharacterSet::detect(),
        }
    }

    /// Create a tree formatter with a specific character set
    pub fn with_charset(charset: CharacterSet) -> Self {
        Self { charset }
    }

    /// Generate indentation string for a tree entry
    /// 
    /// Uses ancestor state tracking for efficient single-pass rendering:
    /// - `depth`: Current depth in the tree
    /// - `is_last`: Vector tracking whether each ancestor is the last child
    pub fn generate_indent(&self, depth: usize, is_last: &[bool]) -> String {
        if depth == 0 {
            return String::new();
        }

        let mut indent = String::new();

        // Build the prefix based on ancestor states
        for i in 0..depth.saturating_sub(1) {
            if i < is_last.len() && is_last[i] {
                indent.push_str(self.charset.empty());
            } else {
                indent.push_str(self.charset.continuation());
            }
        }

        // Add the branch character for this entry
        if depth > 0 {
            let current_is_last = is_last.get(depth - 1).copied().unwrap_or(false);
            if current_is_last {
                indent.push_str(self.charset.branch_last());
            } else {
                indent.push_str(self.charset.branch_middle());
            }
        }

        indent
    }

    /// Compute which entries are last children at each depth level
    /// This enables proper tree drawing in a single pass
    pub fn compute_last_child_map(&self, entries: &[DirEntry]) -> Vec<Vec<bool>> {
        if entries.is_empty() {
            return Vec::new();
        }

        let mut result = vec![Vec::new(); entries.len()];

        // For each entry, determine if it's the last child at each depth
        for idx in 0..entries.len() {
            let current_depth = entries[idx].depth();
            let mut is_last = vec![false; current_depth];

            if current_depth > 0 {
                // Check if this is the last entry at its depth, or if next entry has lower/equal depth
                let is_last_at_depth = if idx + 1 < entries.len() {
                    entries[idx + 1].depth() < current_depth
                } else {
                    true // Last entry overall
                };

                is_last[current_depth - 1] = is_last_at_depth;

                // For ancestor depths, check if any following entry exists at that depth
                for ancestor_depth in 1..current_depth {
                    let has_following_at_depth = entries[idx + 1..]
                        .iter()
                        .any(|e| e.depth() == ancestor_depth);
                    is_last[ancestor_depth - 1] = !has_following_at_depth;
                }
            }

            result[idx] = is_last;
        }

        result
    }
}

impl Default for TreeFormatter {
    fn default() -> Self {
        Self::new()
    }
}
