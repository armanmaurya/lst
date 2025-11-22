use std::fmt;
use std::io;

/// Custom error type for the lst application
#[derive(Debug)]
pub enum LstError {
    /// I/O operation failed
    Io(io::Error),

    /// Invalid path provided
    InvalidPath(String),

    /// Syntax highlighting failed
    HighlightError(String),
}

impl fmt::Display for LstError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LstError::Io(e) => write!(f, "I/O error: {}", e),
            LstError::InvalidPath(path) => write!(f, "Invalid path: {}", path),
            LstError::HighlightError(e) => write!(f, "Syntax highlighting error: {}", e),
        }
    }
}

impl std::error::Error for LstError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LstError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for LstError {
    fn from(error: io::Error) -> Self {
        LstError::Io(error)
    }
}

impl From<Box<dyn std::error::Error>> for LstError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        LstError::HighlightError(error.to_string())
    }
}

/// Type alias for Result with LstError
pub type Result<T> = std::result::Result<T, LstError>;
