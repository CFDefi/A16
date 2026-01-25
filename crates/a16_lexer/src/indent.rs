//! Indentation tracking for A16's Python-like blocks

use crate::token::{Span, Token, TokenKind};

/// Tracks indentation levels for INDENT/DEDENT tokens
pub struct IndentTracker {
    /// Stack of indentation levels
    stack: Vec<usize>,
}

impl IndentTracker {
    pub fn new() -> Self {
        Self {
            stack: vec![0], // Start with indent level 0
        }
    }
    
    /// Process current line indentation, returning any INDENT/DEDENT tokens
    pub fn process_indent(&mut self, indent: usize, pos: u32) -> Option<Vec<Token>> {
        let current = *self.stack.last().unwrap();
        
        if indent > current {
            // Increased indentation - emit INDENT
            self.stack.push(indent);
            Some(vec![Token::new(
                TokenKind::Indent,
                Span::new(pos, pos),
                "",
            )])
        } else if indent < current {
            // Decreased indentation - emit DEDENTs
            let mut tokens = Vec::new();
            
            while let Some(&level) = self.stack.last() {
                if level <= indent {
                    break;
                }
                self.stack.pop();
                tokens.push(Token::new(
                    TokenKind::Dedent,
                    Span::new(pos, pos),
                    "",
                ));
            }
            
            // Validate that we landed on a valid indent level
            if *self.stack.last().unwrap() != indent {
                // Invalid dedent - emit error
                tokens.push(Token::new(
                    TokenKind::Error,
                    Span::new(pos, pos),
                    "invalid dedent",
                ));
            }
            
            if tokens.is_empty() {
                None
            } else {
                Some(tokens)
            }
        } else {
            // Same indentation - no tokens
            None
        }
    }
    
    /// Get remaining DEDENT tokens at end of file
    #[allow(dead_code)]
    pub fn finish(&mut self, pos: u32) -> Vec<Token> {
        let mut tokens = Vec::new();
        
        while self.stack.len() > 1 {
            self.stack.pop();
            tokens.push(Token::new(
                TokenKind::Dedent,
                Span::new(pos, pos),
                "",
            ));
        }
        
        tokens
    }
    
    /// Current indentation level
    #[allow(dead_code)]
    pub fn current_level(&self) -> usize {
        *self.stack.last().unwrap()
    }
}

impl Default for IndentTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_indent_increase() {
        let mut tracker = IndentTracker::new();
        
        // Increase to 4 spaces
        let tokens = tracker.process_indent(4, 0);
        assert!(tokens.is_some());
        assert_eq!(tokens.unwrap()[0].kind, TokenKind::Indent);
    }
    
    #[test]
    fn test_dedent_single() {
        let mut tracker = IndentTracker::new();
        
        tracker.process_indent(4, 0); // Indent
        let tokens = tracker.process_indent(0, 10); // Dedent
        
        assert!(tokens.is_some());
        let tokens = tokens.unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Dedent);
    }
    
    #[test]
    fn test_multiple_dedents() {
        let mut tracker = IndentTracker::new();
        
        tracker.process_indent(4, 0);  // Indent to 4
        tracker.process_indent(8, 10); // Indent to 8
        let tokens = tracker.process_indent(0, 20); // Dedent to 0
        
        assert!(tokens.is_some());
        let tokens = tokens.unwrap();
        assert_eq!(tokens.len(), 2); // Two dedents
    }
}
