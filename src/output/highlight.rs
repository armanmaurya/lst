use std::path::Path;
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

use crate::error::{LstError, Result};

/// Global syntax set, loaded once
static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();

/// Global theme set, loaded once
static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();

/// Get or initialize the syntax set
fn get_syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(|| SyntaxSet::load_defaults_newlines())
}

/// Get or initialize the theme set
fn get_theme_set() -> &'static ThemeSet {
    THEME_SET.get_or_init(|| ThemeSet::load_defaults())
}

/// Print a file's content with syntax highlighting if the extension is supported
pub fn print_file_with_highlighting(path: &Path) -> Result<()> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let content = std::fs::read_to_string(path)?;
    
    let ps = get_syntax_set();
    
    if ps.find_syntax_by_extension(ext).is_some() {
        match highlight_content(&content, ext) {
            Ok(()) => Ok(()),
            Err(e) => {
                // Fallback to plain text on error
                println!("{}", content);
                Err(LstError::HighlightError(format!(
                    "Syntax highlighting failed: {}. Displayed plain text instead.",
                    e
                )))
            }
        }
    } else {
        // No syntax support, print plain
        println!("{}", content);
        Ok(())
    }
}

/// Highlight content using syntect with the default theme
fn highlight_content(content: &str, ext: &str) -> Result<()> {
    let ps = get_syntax_set();
    let ts = get_theme_set();
    
    let syntax = ps
        .find_syntax_by_extension(ext)
        .unwrap_or_else(|| ps.find_syntax_plain_text());
    
    let mut highlighter = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    
    for line in LinesWithEndings::from(content) {
        let ranges: Vec<(Style, &str)> = highlighter
            .highlight_line(line, ps)
            .map_err(|e| LstError::HighlightError(e.to_string()))?;
        print!("{}", as_24_bit_terminal_escaped(&ranges[..], false));
    }
    
    Ok(())
}
