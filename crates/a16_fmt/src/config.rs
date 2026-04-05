//! Formatter configuration

/// Configuration for the A16 formatter
#[derive(Debug, Clone)]
pub struct FormatConfig {
    /// Number of spaces per indent level
    pub indent_size: usize,
    /// Maximum line width before wrapping
    pub max_line_width: usize,
    /// Whether to add a trailing newline
    pub trailing_newline: bool,
    /// Whether to normalize string quotes to double quotes
    pub normalize_quotes: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent_size: 4,
            max_line_width: 100,
            trailing_newline: true,
            normalize_quotes: false,
        }
    }
}
