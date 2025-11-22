use std::io::IsTerminal;

/// Terminal character set for tree drawing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterSet {
    /// Unicode box-drawing characters (├ └ │ ─)
    Unicode,
    /// ASCII fallback characters (| + -)
    Ascii,
}

impl CharacterSet {
    /// Get the appropriate character set based on terminal capabilities
    pub fn detect() -> Self {
        if supports_unicode() {
            CharacterSet::Unicode
        } else {
            CharacterSet::Ascii
        }
    }

    /// Get the branch character for middle children
    pub fn branch_middle(&self) -> &'static str {
        match self {
            CharacterSet::Unicode => "├── ",
            CharacterSet::Ascii => "+-- ",
        }
    }

    /// Get the branch character for last child
    pub fn branch_last(&self) -> &'static str {
        match self {
            CharacterSet::Unicode => "└── ",
            CharacterSet::Ascii => "`-- ",
        }
    }

    /// Get the continuation character for vertical lines
    pub fn continuation(&self) -> &'static str {
        match self {
            CharacterSet::Unicode => "│   ",
            CharacterSet::Ascii => "|   ",
        }
    }

    /// Get the empty space (for last child continuation)
    pub fn empty(&self) -> &'static str {
        "    "
    }
}

/// Check if the terminal supports Unicode characters
fn supports_unicode() -> bool {
    // Check if stdout is a terminal
    if !std::io::stdout().is_terminal() {
        return false;
    }

    // On Windows, check console mode
    #[cfg(windows)]
    {
        use std::env;
        // Check if Windows Terminal, VS Code terminal, or other modern terminal
        if env::var("WT_SESSION").is_ok() 
            || env::var("TERM_PROGRAM").is_ok() 
            || env::var("VSCODE_GIT_IPC_HANDLE").is_ok() {
            return true;
        }
        
        // Check Windows version - Windows 10+ supports Unicode
        if let Ok(version) = env::var("OS") {
            if version.contains("Windows") {
                return true; // Modern Windows supports Unicode
            }
        }
        
        return false;
    }

    // On Unix-like systems, check TERM and locale
    #[cfg(not(windows))]
    {
        use std::env;
        
        // Check TERM environment variable
        if let Ok(term) = env::var("TERM") {
            let term_lower = term.to_lowercase();
            // Most modern terminals support Unicode
            if term_lower.contains("xterm")
                || term_lower.contains("screen")
                || term_lower.contains("tmux")
                || term_lower.contains("rxvt")
                || term_lower.contains("alacritty")
                || term_lower.contains("kitty")
                || term_lower == "linux"
            {
                return true;
            }
        }

        // Check locale for UTF-8 support
        if let Ok(lang) = env::var("LANG") {
            if lang.to_uppercase().contains("UTF-8") || lang.to_uppercase().contains("UTF8") {
                return true;
            }
        }

        if let Ok(lc_all) = env::var("LC_ALL") {
            if lc_all.to_uppercase().contains("UTF-8") || lc_all.to_uppercase().contains("UTF8") {
                return true;
            }
        }

        // Default to false if we can't determine
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_sets() {
        let unicode = CharacterSet::Unicode;
        assert_eq!(unicode.branch_middle(), "├── ");
        assert_eq!(unicode.branch_last(), "└── ");
        assert_eq!(unicode.continuation(), "│   ");

        let ascii = CharacterSet::Ascii;
        assert_eq!(ascii.branch_middle(), "+-- ");
        assert_eq!(ascii.branch_last(), "`-- ");
        assert_eq!(ascii.continuation(), "|   ");
    }

    #[test]
    fn test_detect_returns_valid_charset() {
        let charset = CharacterSet::detect();
        // Just ensure it returns one of the two variants
        assert!(charset == CharacterSet::Unicode || charset == CharacterSet::Ascii);
    }
}
