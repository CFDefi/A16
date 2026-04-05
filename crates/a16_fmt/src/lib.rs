//! A16 Source Code Formatter
//!
//! AST-based formatter that produces consistently styled A16 source code.

mod formatter;
mod config;

pub use config::FormatConfig;
pub use formatter::Formatter;

/// Format A16 source code with default configuration
pub fn format_source(source: &str) -> Result<String, FormatError> {
    format_source_with_config(source, &FormatConfig::default())
}

/// Format A16 source code with custom configuration
pub fn format_source_with_config(source: &str, config: &FormatConfig) -> Result<String, FormatError> {
    let module = a16_parser::parse(source)
        .map_err(|e| FormatError::ParseError(format!("{:?}", e)))?;
    
    let mut formatter = Formatter::new(config.clone());
    formatter.format_module(&module);
    Ok(formatter.output())
}

/// Formatter error type
#[derive(Debug, Clone)]
pub enum FormatError {
    ParseError(String),
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for FormatError {}

#[cfg(test)]
mod tests;
