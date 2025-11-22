use std::collections::HashSet;
use std::path::PathBuf;
use walkdir::DirEntry;

/// Build a set of directories that should be shown based on search pattern
/// 
/// When searching, we need to show:
/// 1. Files/dirs that match the pattern
/// 2. All parent directories leading to matches
pub fn build_search_filter(
    entries: &[DirEntry],
    pattern: &str,
) -> HashSet<PathBuf> {
    let mut show_dirs = HashSet::new();
    let pattern_lower = pattern.to_lowercase();

    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        
        if name.contains(&pattern_lower) {
            // Add all parent directories of this match
            let mut path = entry.path();
            while let Some(parent) = path.parent() {
                show_dirs.insert(parent.to_path_buf());
                path = parent;
            }
        }
    }

    show_dirs
}

/// Check if an entry should be printed based on search criteria
pub fn should_print_entry(
    entry: &DirEntry,
    search_pattern: Option<&str>,
    show_dirs: &HashSet<PathBuf>,
) -> bool {
    match search_pattern {
        Some(pattern) => {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            let pattern_lower = pattern.to_lowercase();
            name.contains(&pattern_lower) || show_dirs.contains(entry.path())
        }
        None => true,
    }
}
