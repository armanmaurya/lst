use walkdir::DirEntry;

/// Check if a directory entry is hidden (starts with '.' but not '.' or '..')
pub fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.') && s != "." && s != "..")
        .unwrap_or(false)
}

/// Filter predicate for walkdir that respects the show_hidden flag
pub fn should_show_entry(entry: &DirEntry, show_hidden: bool) -> bool {
    show_hidden || !is_hidden(entry)
}
