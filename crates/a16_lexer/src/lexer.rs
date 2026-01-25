//! A16 Lexer implementation
//!
//! Hand-written lexer for performance and error recovery.

use crate::indent::IndentTracker;
use crate::token::{Span, Token, TokenKind};

/// The A16 lexer
pub struct Lexer<'src> {
    source: &'src str,
    chars: std::iter::Peekable<std::str::CharIndices<'src>>,
    pos: u32,
    indent_tracker: IndentTracker,
    at_line_start: bool,
    pending_tokens: Vec<Token>,
}

impl<'src> Lexer<'src> {
    /// Create a new lexer for the given source
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
            pos: 0,
            indent_tracker: IndentTracker::new(),
            at_line_start: true,
            pending_tokens: Vec::new(),
        }
    }
    
    /// Get all tokens as a vector
    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }
    
    /// Get the next token
    pub fn next_token(&mut self) -> Token {
        // Return pending tokens first (dedents, etc.)
        if let Some(token) = self.pending_tokens.pop() {
            return token;
        }
        
        // Handle indentation at line start
        if self.at_line_start {
            self.at_line_start = false;
            if let Some(mut tokens) = self.handle_indentation() {
                if !tokens.is_empty() {
                    // Reverse so we can pop from the end in correct order
                    tokens.reverse();
                    // Take the first token to return
                    let first = tokens.pop().unwrap();
                    // Store the rest as pending
                    self.pending_tokens.extend(tokens);
                    return first;
                }
            }
        }
        
        // Skip whitespace (but not newlines)
        self.skip_whitespace();
        
        let start = self.pos;
        
        match self.peek() {
            None => Token::new(TokenKind::Eof, Span::new(start, start), ""),
            
            Some('\n') => {
                self.advance();
                self.at_line_start = true;
                Token::new(TokenKind::Newline, Span::new(start, self.pos), "\n")
            }
            
            Some('#') => self.lex_comment(start),
            
            Some('"') | Some('\'') => self.lex_string(start),
            
            Some(c) if c.is_ascii_digit() => self.lex_number(start),
            
            Some(c) if is_ident_start(c) => self.lex_identifier(start),
            
            Some(_) => self.lex_operator(start),
        }
    }
    
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }
    
    fn peek_next(&self) -> Option<char> {
        let mut iter = self.source[self.pos as usize..].chars();
        iter.next();
        iter.next()
    }
    
    fn advance(&mut self) -> Option<char> {
        self.chars.next().map(|(i, c)| {
            self.pos = (i + c.len_utf8()) as u32;
            c
        })
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn handle_indentation(&mut self) -> Option<Vec<Token>> {
        let start = self.pos;
        let mut indent = 0;
        
        while let Some(c) = self.peek() {
            match c {
                ' ' => {
                    indent += 1;
                    self.advance();
                }
                '\t' => {
                    indent += 4; // Tab = 4 spaces
                    self.advance();
                }
                '\n' => {
                    // Empty line, ignore indent
                    return None;
                }
                '#' => {
                    // Comment line, ignore indent
                    return None;
                }
                _ => break,
            }
        }
        
        self.indent_tracker.process_indent(indent, start)
    }
    
    fn lex_comment(&mut self, start: u32) -> Token {
        self.advance(); // consume #
        
        // Check for doc comment (###)
        let is_doc = self.peek() == Some('#') && self.peek_next() == Some('#');
        
        if is_doc {
            self.advance();
            self.advance();
            // Read until closing ###
            let mut depth = 0;
            loop {
                match self.peek() {
                    None => break,
                    Some('#') if depth >= 2 => {
                        self.advance();
                        break;
                    }
                    Some('#') => {
                        depth += 1;
                        self.advance();
                    }
                    Some('\n') => {
                        depth = 0;
                        self.advance();
                    }
                    Some(_) => {
                        depth = 0;
                        self.advance();
                    }
                }
            }
            let text = &self.source[start as usize..self.pos as usize];
            Token::new(TokenKind::DocComment, Span::new(start, self.pos), text)
        } else {
            // Single line comment
            while let Some(c) = self.peek() {
                if c == '\n' {
                    break;
                }
                self.advance();
            }
            let text = &self.source[start as usize..self.pos as usize];
            Token::new(TokenKind::Comment, Span::new(start, self.pos), text)
        }
    }
    
    fn lex_string(&mut self, start: u32) -> Token {
        let quote = self.advance().unwrap();
        let triple = self.peek() == Some(quote) && self.peek_next() == Some(quote);
        
        if triple {
            self.advance();
            self.advance();
            self.lex_triple_string(start, quote)
        } else {
            self.lex_single_string(start, quote)
        }
    }
    
    fn lex_single_string(&mut self, start: u32, quote: char) -> Token {
        loop {
            match self.peek() {
                None | Some('\n') => {
                    // Unterminated string
                    let text = &self.source[start as usize..self.pos as usize];
                    return Token::new(TokenKind::Error, Span::new(start, self.pos), text);
                }
                Some('\\') => {
                    self.advance();
                    self.advance(); // Skip escaped char
                }
                Some(c) if c == quote => {
                    self.advance();
                    let text = &self.source[start as usize..self.pos as usize];
                    return Token::new(TokenKind::String, Span::new(start, self.pos), text);
                }
                Some(_) => {
                    self.advance();
                }
            }
        }
    }
    
    fn lex_triple_string(&mut self, start: u32, quote: char) -> Token {
        loop {
            match self.peek() {
                None => {
                    let text = &self.source[start as usize..self.pos as usize];
                    return Token::new(TokenKind::Error, Span::new(start, self.pos), text);
                }
                Some(c) if c == quote => {
                    if self.peek_next() == Some(quote) {
                        self.advance();
                        if self.peek_next() == Some(quote) {
                            self.advance();
                            self.advance();
                            let text = &self.source[start as usize..self.pos as usize];
                            return Token::new(TokenKind::String, Span::new(start, self.pos), text);
                        }
                    } else {
                        self.advance();
                    }
                }
                Some(_) => {
                    self.advance();
                }
            }
        }
    }
    
    fn lex_number(&mut self, start: u32) -> Token {
        // Consume digits
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        
        // Check for float (has decimal point)
        let has_decimal = self.peek() == Some('.') && 
            self.peek_next().map(|c| c.is_ascii_digit()).unwrap_or(false);
        
        // Check for scientific notation (e.g., 1e10)
        let has_exponent = self.peek() == Some('e') || self.peek() == Some('E');
        
        let is_float = has_decimal || has_exponent;
        
        if has_decimal {
            self.advance(); // consume .
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() || c == '_' {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        
        // Handle exponent (for both 3.14e10 and 1e10)
        if self.peek() == Some('e') || self.peek() == Some('E') {
            self.advance();
            if self.peek() == Some('+') || self.peek() == Some('-') {
                self.advance();
            }
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        
        let text = &self.source[start as usize..self.pos as usize];
        if is_float {
            Token::new(TokenKind::Float, Span::new(start, self.pos), text)
        } else {
            Token::new(TokenKind::Int, Span::new(start, self.pos), text)
        }
    }
    
    fn lex_identifier(&mut self, start: u32) -> Token {
        while let Some(c) = self.peek() {
            if is_ident_continue(c) {
                self.advance();
            } else {
                break;
            }
        }
        
        let text = &self.source[start as usize..self.pos as usize];
        
        // Check for keywords
        let kind = TokenKind::from_keyword(text).unwrap_or(TokenKind::Ident);
        
        Token::new(kind, Span::new(start, self.pos), text)
    }
    
    fn lex_operator(&mut self, start: u32) -> Token {
        let c = self.advance().unwrap();
        let next = self.peek();
        
        let (kind, consumed) = match (c, next) {
            // Two-character operators
            ('+', Some('=')) => (TokenKind::PlusEq, true),
            ('-', Some('=')) => (TokenKind::MinusEq, true),
            ('-', Some('>')) => (TokenKind::Arrow, true),
            ('*', Some('*')) => {
                self.advance();
                if self.peek() == Some('=') {
                    (TokenKind::DoubleStarEq, true)
                } else {
                    (TokenKind::DoubleStar, false)
                }
            }
            ('*', Some('=')) => (TokenKind::StarEq, true),
            ('/', Some('/')) => {
                self.advance();
                if self.peek() == Some('=') {
                    (TokenKind::DoubleSlashEq, true)
                } else {
                    (TokenKind::DoubleSlash, false)
                }
            }
            ('/', Some('=')) => (TokenKind::SlashEq, true),
            ('%', Some('=')) => (TokenKind::PercentEq, true),
            ('=', Some('=')) => (TokenKind::Eq, true),
            ('=', Some('>')) => (TokenKind::FatArrow, true),
            ('!', Some('=')) => (TokenKind::Ne, true),
            ('<', Some('=')) => (TokenKind::Le, true),
            ('<', Some('<')) => {
                self.advance();
                if self.peek() == Some('=') {
                    (TokenKind::LShiftEq, true)
                } else {
                    (TokenKind::LShift, false)
                }
            }
            ('>', Some('=')) => (TokenKind::Ge, true),
            ('>', Some('>')) => {
                self.advance();
                if self.peek() == Some('=') {
                    (TokenKind::RShiftEq, true)
                } else {
                    (TokenKind::RShift, false)
                }
            }
            ('&', Some('=')) => (TokenKind::AmpEq, true),
            ('|', Some('=')) => (TokenKind::PipeEq, true),
            ('^', Some('=')) => (TokenKind::CaretEq, true),
            ('.', Some('.')) => {
                self.advance();
                if self.peek() == Some('.') {
                    (TokenKind::Ellipsis, true)
                } else {
                    // Just two dots, error
                    (TokenKind::Error, false)
                }
            }
            
            // Single-character operators
            ('+', _) => (TokenKind::Plus, false),
            ('-', _) => (TokenKind::Minus, false),
            ('*', _) => (TokenKind::Star, false),
            ('/', _) => (TokenKind::Slash, false),
            ('%', _) => (TokenKind::Percent, false),
            ('@', _) => (TokenKind::At, false),
            ('&', _) => (TokenKind::Ampersand, false),
            ('|', _) => (TokenKind::Pipe, false),
            ('^', _) => (TokenKind::Caret, false),
            ('~', _) => (TokenKind::Tilde, false),
            ('<', _) => (TokenKind::Lt, false),
            ('>', _) => (TokenKind::Gt, false),
            ('=', _) => (TokenKind::Assign, false),
            
            // Delimiters
            ('(', _) => (TokenKind::LParen, false),
            (')', _) => (TokenKind::RParen, false),
            ('[', _) => (TokenKind::LBracket, false),
            (']', _) => (TokenKind::RBracket, false),
            ('{', _) => (TokenKind::LBrace, false),
            ('}', _) => (TokenKind::RBrace, false),
            
            // Punctuation
            (',', _) => (TokenKind::Comma, false),
            (':', _) => (TokenKind::Colon, false),
            (';', _) => (TokenKind::Semicolon, false),
            ('.', _) => (TokenKind::Dot, false),
            
            _ => (TokenKind::Error, false),
        };
        
        if consumed {
            self.advance();
        }
        
        let text = &self.source[start as usize..self.pos as usize];
        Token::new(kind, Span::new(start, self.pos), text)
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl<'src> Iterator for Lexer<'src> {
    type Item = Token;
    
    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if token.kind == TokenKind::Eof {
            None
        } else {
            Some(token)
        }
    }
}
