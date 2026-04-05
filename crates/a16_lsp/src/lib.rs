//! A16 Language Server
//!
//! Provides IDE integration features: diagnostics, completion, hover, goto definition.
//! This crate implements the LSP logic; transport (stdin/stdout, tower-lsp) is in the CLI.

pub mod diagnostics;
pub mod completion;
pub mod hover;

use indexmap::IndexMap;

/// Document state tracked by the server
#[derive(Debug, Clone)]
pub struct DocumentState {
    /// URI (path) of the document
    pub uri: String,
    /// Full source text
    pub source: String,
    /// Version counter
    pub version: i64,
}

/// The core language server backend
pub struct LanguageBackend {
    /// Open documents
    pub documents: IndexMap<String, DocumentState>,
}

impl LanguageBackend {
    /// Create a new backend
    pub fn new() -> Self {
        Self {
            documents: IndexMap::new(),
        }
    }

    /// Open or update a document
    pub fn update_document(&mut self, uri: &str, source: String, version: i64) {
        self.documents.insert(uri.to_string(), DocumentState {
            uri: uri.to_string(),
            source,
            version,
        });
    }

    /// Close a document
    pub fn close_document(&mut self, uri: &str) {
        self.documents.swap_remove(uri);
    }

    /// Get diagnostics for a document
    pub fn get_diagnostics(&self, uri: &str) -> Vec<diagnostics::Diagnostic> {
        if let Some(doc) = self.documents.get(uri) {
            diagnostics::compute_diagnostics(&doc.source)
        } else {
            vec![]
        }
    }

    /// Get completions at a position
    pub fn get_completions(&self, uri: &str, _line: u32, _character: u32) -> Vec<completion::CompletionItem> {
        if let Some(doc) = self.documents.get(uri) {
            completion::compute_completions(&doc.source)
        } else {
            completion::compute_completions("")
        }
    }

    /// Get hover info at a position
    pub fn get_hover(&self, _uri: &str, _line: u32, word: &str) -> Option<hover::HoverInfo> {
        hover::compute_hover(word)
    }
}

impl Default for LanguageBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
