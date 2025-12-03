use std::collections::HashSet;
use std::path::PathBuf;
use walkdir::DirEntry;
use super::filters::is_hidden;
use rayon::prelude::*;
use dashmap::DashSet;
use aho_corasick::AhoCorasick;

/// Build a set of directories that should be shown based on search pattern
/// 
/// When searching, we need to show:
/// 1. Files/dirs that match the pattern
/// 2. All parent directories leading to matches
pub fn build_search_filter(
    entries: &[DirEntry],
    pattern: &str,
    show_hidden: bool,
) -> HashSet<PathBuf> {
    // Lowercase pattern once using ASCII for speed; build fast matcher
    let pattern_lower = pattern.to_ascii_lowercase();
    let matcher = AhoCorasick::new([pattern_lower.clone()]).expect("failed to build matcher");

    // Concurrent set to collect parent directories without intermediate Vecs
    let show_dirs = DashSet::new();

    entries.par_iter().for_each(|entry| {
        // Skip hidden directories entirely when searching, unless explicitly showing hidden
        if entry.file_type().is_dir() && !show_hidden && is_hidden(entry) {
            return;
        }
        let name = entry.file_name().to_string_lossy();
        let name_lc = name.to_ascii_lowercase();
        if matcher.is_match(&name_lc) {
            // Insert parent chain directly into concurrent set
            let mut path = entry.path();
            while let Some(parent) = path.parent() {
                show_dirs.insert(parent.to_path_buf());
                path = parent;
            }
        }
    });

    // Convert to HashSet for downstream usage
    show_dirs.into_iter().collect()
}

/// Check if an entry should be printed based on search criteria
pub fn should_print_entry(
    entry: &DirEntry,
    search_pattern: Option<&str>,
    show_dirs: &HashSet<PathBuf>,
    show_hidden: bool,
) -> bool {
    match search_pattern {
        Some(pattern) => {
            // Do not print hidden directories while searching unless overridden
            if entry.file_type().is_dir() && !show_hidden && is_hidden(entry) {
                return false;
            }
            let name = entry.file_name().to_string_lossy();
            let name_lc = name.to_ascii_lowercase();
            let pattern_lower = pattern.to_ascii_lowercase();
            name_lc.contains(&pattern_lower) || show_dirs.contains(entry.path())
        }
        None => true,
    }
}
