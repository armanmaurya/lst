use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};
use ignore::{WalkBuilder, DirEntry as IgnoreDirEntry};
use serde_json::json;

use super::formatter::{
    format_directory_name, format_file_name, format_file_size, 
    format_size_colored, TreeFormatter,
};
use super::terminal::CharacterSet;
use crate::core::search::{build_search_filter, should_print_entry};
use crate::core::tree::collect_entries;
use crate::core::filters::should_show_entry;
use crate::error::Result;

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    /// Plain text with tree characters
    Text,
    /// JSON structured output
    Json,
}

/// Configuration for tree printing
pub struct TreeConfig<'a> {
    pub path: &'a Path,
    pub max_depth: usize,
    pub show_all: bool,
    pub search_pattern: Option<&'a str>,
    pub spinner_stop: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    pub json_output: bool,
}

impl<'a> TreeConfig<'a> {
    /// Get the output format
    pub fn format(&self) -> OutputFormat {
        if self.json_output {
            OutputFormat::Json
        } else {
            OutputFormat::Text
        }
    }
}

/// Holds the collected tree data
struct TreeData {
    entries: Vec<DirEntry>,
    show_dirs: HashSet<PathBuf>,
}

impl TreeData {
    fn collect(config: &TreeConfig) -> Self {
        let entries = collect_entries(config.path, config.max_depth, config.show_all);
        let show_dirs = if let Some(pattern) = config.search_pattern {
            build_search_filter(&entries, pattern, config.show_all)
        } else {
            HashSet::new()
        };
        
        Self { entries, show_dirs }
    }
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
        let tree_data = TreeData::collect(config);
        print_tree(writer, &tree_data.entries, config.search_pattern, &tree_data.show_dirs, self.use_color)?;
        Ok(())
    }

    /// Write tree to a file with a header
    pub fn write_to_file(&self, output_path: &str, config: &TreeConfig) -> Result<()> {
        let mut file = std::fs::File::create(output_path)?;
        
        match config.format() {
            OutputFormat::Json => self.write_json(&mut file, config)?,
            OutputFormat::Text => {
                writeln!(file, ".")?;
                self.write(&mut file, config)?;
            }
        }
        
        println!("Tree exported to {}", output_path);
        Ok(())
    }

    /// Write tree to terminal (stdout)
    pub fn write_to_terminal(&self, config: &TreeConfig) -> Result<()> {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        
        match config.format() {
            OutputFormat::Json => self.write_json(&mut handle, config),
            OutputFormat::Text => self.write_streaming(&mut handle, config),
        }
    }

    /// Write directory tree as JSON
    fn write_json<W: Write>(&self, writer: &mut W, config: &TreeConfig) -> Result<()> {
        let tree_data = TreeData::collect(config);
        let json_tree = JsonTreeBuilder::build(&tree_data, config);
        
        let json_str = serde_json::to_string_pretty(&json_tree)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        writeln!(writer, "{}", json_str)?;
        Ok(())
    }
}

/// Helper struct for building JSON tree representation
struct JsonTreeBuilder;

impl JsonTreeBuilder {
    fn build(tree_data: &TreeData, config: &TreeConfig) -> serde_json::Value {
        json!({
            "name": config.path.file_name().unwrap_or(config.path.as_os_str()).to_string_lossy(),
            "type": "directory",
            "path": config.path.to_string_lossy(),
            "children": Self::build_children(
                &tree_data.entries,
                config.path,
                config.search_pattern,
                &tree_data.show_dirs,
                config.show_all
            )
        })
    }

    fn build_children(
        entries: &[DirEntry],
        parent_path: &Path,
        search_pattern: Option<&str>,
        show_dirs: &HashSet<PathBuf>,
        show_all: bool,
    ) -> serde_json::Value {
        let mut children = Vec::new();

        for entry in entries {
            if entry.path().parent() != Some(parent_path) {
                continue;
            }

            if let Some(pattern) = search_pattern {
                if !should_print_entry(entry, Some(pattern), show_dirs, show_all) {
                    continue;
                }
            }

            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.file_type().is_dir();
            let size = if !is_dir {
                entry.metadata().map(|m| m.len()).ok()
            } else {
                None
            };

            let mut node = json!({
                "name": name,
                "type": if is_dir { "directory" } else { "file" },
                "path": entry.path().to_string_lossy().to_string(),
            });

            if let Some(s) = size {
                node["size"] = json!(s);
            }

            if is_dir {
                let subtree = Self::build_children(entries, entry.path(), search_pattern, show_dirs, show_all);
                if !subtree.as_array().unwrap().is_empty() || search_pattern.is_none() {
                    node["children"] = subtree;
                }
            }

            children.push(node);
        }

        json!(children)
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

/// Print a single directory entry line for ignore::DirEntry
fn print_entry_line_ignore<W: Write>(
    writer: &mut W,
    entry: &IgnoreDirEntry,
    indent: &str,
    use_color: bool,
) -> std::io::Result<()> {
    let file_name = entry.file_name().to_string_lossy();

    // ignore::DirEntry may not always have metadata/file_type pre-fetched; be defensive
    if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
        let formatted_name = format_directory_name(&file_name, use_color);
        writeln!(writer, "{}{}/", indent, formatted_name)
    } else {
        // Compute size lazily; skip on error for speed
        let size = std::fs::metadata(entry.path()).map(|m| m.len()).unwrap_or(0);
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

    // Use Unicode for better visual output
    let charset = if use_color {
        CharacterSet::detect()
    } else {
        CharacterSet::Unicode  // Use Unicode for file output too
    };

    let formatter = TreeFormatter::with_charset(charset);
    
    // Filter entries based on search pattern first
    let filtered_entries: Vec<&DirEntry> = entries
        .iter()
        .filter(|entry| should_print_entry(entry, search_pattern, show_dirs, true))
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

/// Stream the directory tree while scanning, printing entries incrementally
impl TreeWriter {
    fn write_streaming<W: Write>(&self, writer: &mut W, config: &TreeConfig) -> Result<()> {
        // Use Unicode for better visual output
        let charset = if self.use_color { CharacterSet::detect() } else { CharacterSet::Unicode };
        let formatter = TreeFormatter::with_charset(charset);

        // Choose walker: for search, use ignore's fast walker; otherwise use walkdir
        let searching = config.search_pattern.is_some();
        let use_ignore = searching;
        let mut iter_ig_opt = None;
        let mut iter_wd_opt = None;
        if use_ignore {
            let it = WalkBuilder::new(config.path)
                .max_depth(if config.max_depth == usize::MAX { None } else { Some(config.max_depth) })
                .hidden(!config.show_all)
                .git_ignore(true)
                .git_global(true)
                .git_exclude(true)
                    .filter_entry(|e| !crate::core::filters::is_common_skip_os(e.file_name()))
                    .build()
                .peekable();
            iter_ig_opt = Some(it);
        } else {
            let it = WalkDir::new(config.path)
                .min_depth(1)
                .max_depth(config.max_depth)
                .into_iter()
                .filter_entry(|e| should_show_entry(e, config.show_all))
                .peekable();
            iter_wd_opt = Some(it);
        }

        // Track ancestor continuation states per depth
        let mut ancestor_has_more: Vec<bool> = Vec::new();

        // Precompute search visibility helper
        // We need show_dirs for search to print parents; compute lazily when needed
        let show_dirs = if let Some(pattern) = config.search_pattern {
            let entries = collect_entries(config.path, config.max_depth, config.show_all);
            build_search_filter(&entries, pattern, config.show_all)
        } else {
            std::collections::HashSet::new()
        };

        let mut first_print_done = false;
        if use_ignore {
            let iter_ig = iter_ig_opt.expect("iterator init");
            let mut iter_ig = iter_ig;
            while let Some(res) = iter_ig.next() {
                let entry = match res {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                // ignore walker already handles hidden when configured; apply search filter
                let name = entry.file_name().to_string_lossy();
                let matches = match config.search_pattern {
                    Some(p) => {
                        let name_lc = name.to_ascii_lowercase();
                        name_lc.contains(&p.to_ascii_lowercase()) || show_dirs.contains(entry.path())
                    }
                    None => true,
                };
                if !matches { continue; }

                let depth = entry.depth();
                let next_depth = iter_ig.peek().and_then(|r| r.as_ref().ok()).map(|e| e.depth());

            // Adjust ancestor stack when depth decreases
            let current_stack_len = ancestor_has_more.len();
            if depth < current_stack_len {
                ancestor_has_more.truncate(depth);
            } else if depth > current_stack_len {
                // Extend stack for deeper levels; assume ancestors have more until proven otherwise
                ancestor_has_more.resize(depth, true);
            }

            // Compute is_last flags slice for formatter
            let mut is_last: Vec<bool> = vec![false; depth];
            // Ancestors: if ancestor_has_more[i] is false, it's last at that level
            for i in 0..depth.saturating_sub(1) {
                is_last[i] = !ancestor_has_more.get(i).copied().unwrap_or(false);
            }

            // Current level last-child: if next entry is at shallower depth, this is last
            let current_is_last = match next_depth {
                Some(nd) => nd < depth,
                None => true,
            };
            if depth > 0 {
                is_last[depth - 1] = current_is_last;
            }

            // Update ancestor_has_more for current level based on peek
            if depth > 0 {
                let idx = depth - 1;
                ancestor_has_more.resize(depth, true);
                ancestor_has_more[idx] = !current_is_last;
            }

            // Stop spinner on first printable entry to avoid overlap
            if !first_print_done {
                if let Some(stop) = &config.spinner_stop {
                    stop.store(true, std::sync::atomic::Ordering::Relaxed);
                    // Clear spinner line on stderr
                    eprint!("\r                \r");
                    let _ = std::io::stderr().flush();
                }
                first_print_done = true;
            }

            let indent = formatter.generate_indent(depth, &is_last);
            print_entry_line_ignore(writer, &entry, &indent, self.use_color)?;
            }
        } else {
            let iter_wd = iter_wd_opt.expect("iterator init");
            let mut iter_wd = iter_wd;
            while let Some(res) = iter_wd.next() {
                let entry = match res {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                if !should_print_entry(&entry, config.search_pattern, &show_dirs, config.show_all) {
                    continue;
                }

                let depth = entry.depth();
                let next_depth = iter_wd.peek().and_then(|r| r.as_ref().ok()).map(|e| e.depth());

                let current_stack_len = ancestor_has_more.len();
                if depth < current_stack_len {
                    ancestor_has_more.truncate(depth);
                } else if depth > current_stack_len {
                    ancestor_has_more.resize(depth, true);
                }

                let mut is_last: Vec<bool> = vec![false; depth];
                for i in 0..depth.saturating_sub(1) {
                    is_last[i] = !ancestor_has_more.get(i).copied().unwrap_or(false);
                }

                let current_is_last = match next_depth {
                    Some(nd) => nd < depth,
                    None => true,
                };
                if depth > 0 {
                    is_last[depth - 1] = current_is_last;
                }

                if depth > 0 {
                    let idx = depth - 1;
                    ancestor_has_more.resize(depth, true);
                    ancestor_has_more[idx] = !current_is_last;
                }

                if !first_print_done {
                    if let Some(stop) = &config.spinner_stop {
                        stop.store(true, std::sync::atomic::Ordering::Relaxed);
                        eprint!("\r                \r");
                        let _ = std::io::stderr().flush();
                    }
                    first_print_done = true;
                }

                let indent = formatter.generate_indent(depth, &is_last);
                print_entry_line(writer, &entry, &indent, self.use_color)?;
            }
        }

        Ok(())
    }
}
