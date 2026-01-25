//! Lexer tests

use crate::lexer::Lexer;
use crate::token::TokenKind;

#[test]
fn test_lex_keywords() {
    let source = "fn agent tool memory if else for while";
    let tokens: Vec<_> = Lexer::new(source).tokenize();
    
    let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
    assert_eq!(kinds, vec![
        TokenKind::Fn,
        TokenKind::Agent,
        TokenKind::Tool,
        TokenKind::Memory,
        TokenKind::If,
        TokenKind::Else,
        TokenKind::For,
        TokenKind::While,
        TokenKind::Eof,
    ]);
}

#[test]
fn test_lex_numbers() {
    let source = "42 3.14 1_000_000 1e10";
    let tokens: Vec<_> = Lexer::new(source).tokenize();
    
    assert_eq!(tokens[0].kind, TokenKind::Int);
    assert_eq!(tokens[0].text.as_str(), "42");
    
    assert_eq!(tokens[1].kind, TokenKind::Float);
    assert_eq!(tokens[1].text.as_str(), "3.14");
    
    assert_eq!(tokens[2].kind, TokenKind::Int);
    assert_eq!(tokens[2].text.as_str(), "1_000_000");
    
    assert_eq!(tokens[3].kind, TokenKind::Float);
}

#[test]
fn test_lex_strings() {
    let source = r#""hello" 'world'"#;
    let tokens: Vec<_> = Lexer::new(source).tokenize();
    
    assert_eq!(tokens[0].kind, TokenKind::String);
    assert_eq!(tokens[1].kind, TokenKind::String);
}

#[test]
fn test_lex_operators() {
    let source = "+ - * / == != <= >= -> =>";
    let tokens: Vec<_> = Lexer::new(source).tokenize();
    
    let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
    assert_eq!(kinds, vec![
        TokenKind::Plus,
        TokenKind::Minus,
        TokenKind::Star,
        TokenKind::Slash,
        TokenKind::Eq,
        TokenKind::Ne,
        TokenKind::Le,
        TokenKind::Ge,
        TokenKind::Arrow,
        TokenKind::FatArrow,
        TokenKind::Eof,
    ]);
}

#[test]
fn test_lex_agent_definition() {
    let source = r#"
agent MyAgent:
    model: gpt4
    task run():
        return 42
"#;
    let tokens: Vec<_> = Lexer::new(source).tokenize();
    
    // Should have agent keyword, identifier, colon, newline, indent, etc.
    let agent_idx = tokens.iter().position(|t| t.kind == TokenKind::Agent);
    assert!(agent_idx.is_some());
    
    let task_idx = tokens.iter().position(|t| t.kind == TokenKind::Task);
    assert!(task_idx.is_some());
}

#[test]
fn test_lex_indentation() {
    let source = "if x:\n    y\nz";
    let tokens: Vec<_> = Lexer::new(source).tokenize();
    
    let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
    
    // Should have: If, Ident(x), Colon, Newline, Indent, Ident(y), Newline, Dedent, Ident(z), Eof
    assert!(kinds.contains(&TokenKind::Indent));
    assert!(kinds.contains(&TokenKind::Dedent));
}
