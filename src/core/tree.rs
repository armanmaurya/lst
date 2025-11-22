use std::path::Path;
use walkdir::{DirEntry, WalkDir};

use super::filters::should_show_entry;

/// Collect directory entries for the given path with specified depth and visibility options
pub fn collect_entries(path: &Path, max_depth: usize, show_hidden: bool) -> Vec<DirEntry> {
    WalkDir::new(path)
        .min_depth(1)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| should_show_entry(e, show_hidden))
        .filter_map(Result::ok)
        .collect()
}
