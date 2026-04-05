//! Token definitions for A16

use smol_str::SmolStr;

/// A span in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
    
    pub fn len(&self) -> u32 {
        self.end - self.start
    }
    
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// A token in the A16 source code
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: SmolStr,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, text: impl Into<SmolStr>) -> Self {
        Self {
            kind,
            span,
            text: text.into(),
        }
    }
}

/// Token kinds for A16
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // === Keywords ===
    // Core
    Fn,
    Class,
    Agent,
    Tool,
    Memory,
    Prompt,
    Task,
    
    // Control flow
    If,
    Elif,
    Else,
    Match,
    Case,
    For,
    While,
    Loop,
    Break,
    Continue,
    Return,
    
    // Error handling
    Try,
    Except,
    Finally,
    Raise,
    Assert,
    
    // Declarations
    Let,
    Const,
    Type,
    Enum,
    Struct,
    
    // Modules
    Import,
    From,
    As,
    Export,
    
    // Async
    Async,
    Await,
    Spawn,
    Yield,
    
    // FFI
    Extern,
    
    // Operators (keyword)
    And,
    Or,
    Not,
    In,
    Is,
    
    // Other keywords
    With,
    Pass,
    Del,
    
    // Literals
    None,
    True,
    False,
    
    // AI Keywords
    Budget,
    Model,
    Context,
    Retrieve,
    Store,
    Compress,
    Stream,
    Invoke,
    Sandbox,
    Audit,
    Permit,
    Deny,
    Policy,
    Trace,
    
    // Team keywords
    Team,
    On,
    
    // === Literals ===
    Int,
    Float,
    String,
    FString,
    PromptString,
    RawString,
    Bytes,
    
    // === Identifiers ===
    Ident,
    
    // === Operators ===
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    DoubleSlash,// //
    Percent,    // %
    DoubleStar, // **
    At,         // @
    
    // Bitwise
    Ampersand,  // &
    Pipe,       // |
    Caret,      // ^
    Tilde,      // ~
    LShift,     // <<
    RShift,     // >>
    
    // Comparison
    Eq,         // ==
    Ne,         // !=
    Lt,         // <
    Le,         // <=
    Gt,         // >
    Ge,         // >=
    
    // Assignment
    Assign,     // =
    PlusEq,     // +=
    MinusEq,    // -=
    StarEq,     // *=
    SlashEq,    // /=
    PercentEq,  // %=
    AmpEq,      // &=
    PipeEq,     // |=
    CaretEq,    // ^=
    LShiftEq,   // <<=
    RShiftEq,   // >>=
    DoubleStarEq, // **=
    DoubleSlashEq, // //=
    
    // Delimiters
    LParen,     // (
    RParen,     // )
    LBracket,   // [
    RBracket,   // ]
    LBrace,     // {
    RBrace,     // }
    
    // Punctuation
    Comma,      // ,
    Colon,      // :
    Semicolon,  // ;
    Dot,        // .
    Arrow,      // ->
    FatArrow,   // =>
    Ellipsis,   // ...
    
    // === Structural ===
    Newline,
    Indent,
    Dedent,
    
    // === Special ===
    Comment,
    DocComment,
    Whitespace,
    Error,
    Eof,
}

impl TokenKind {
    /// Returns true if this token is a keyword
    pub fn is_keyword(&self) -> bool {
        use TokenKind::*;
        matches!(
            self,
            Fn | Class | Agent | Tool | Memory | Prompt | Task |
            If | Elif | Else | Match | Case | For | While | Loop |
            Break | Continue | Return | Try | Except | Finally |
            Raise | Assert | Let | Const | Type | Enum | Struct |
            Import | From | As | Export | Async | Await | Spawn |
            Yield | And | Or | Not | In | Is | With | Pass | Del |
            None | True | False | Budget | Model | Context | Retrieve |
            Store | Compress | Stream | Invoke | Sandbox | Audit |
            Permit | Deny | Policy | Trace | Team | On | Extern
        )
    }
    
    /// Returns true if this token is trivia (whitespace, comments)
    pub fn is_trivia(&self) -> bool {
        matches!(self, TokenKind::Whitespace | TokenKind::Comment)
    }
    
    /// Keyword lookup from string
    pub fn from_keyword(s: &str) -> Option<TokenKind> {
        use TokenKind::*;
        match s {
            "fn" => Some(Fn),
            "class" => Some(Class),
            "agent" => Some(Agent),
            "tool" => Some(Tool),
            "memory" => Some(Memory),
            "prompt" => Some(Prompt),
            "task" => Some(Task),
            "if" => Some(If),
            "elif" => Some(Elif),
            "else" => Some(Else),
            "match" => Some(Match),
            "case" => Some(Case),
            "for" => Some(For),
            "while" => Some(While),
            "loop" => Some(Loop),
            "break" => Some(Break),
            "continue" => Some(Continue),
            "return" => Some(Return),
            "try" => Some(Try),
            "except" => Some(Except),
            "finally" => Some(Finally),
            "raise" => Some(Raise),
            "assert" => Some(Assert),
            "let" => Some(Let),
            "const" => Some(Const),
            "type" => Some(Type),
            "enum" => Some(Enum),
            "struct" => Some(Struct),
            "import" => Some(Import),
            "from" => Some(From),
            "as" => Some(As),
            "export" => Some(Export),
            "async" => Some(Async),
            "await" => Some(Await),
            "spawn" => Some(Spawn),
            "yield" => Some(Yield),
            "extern" => Some(Extern),
            "and" => Some(And),
            "or" => Some(Or),
            "not" => Some(Not),
            "in" => Some(In),
            "is" => Some(Is),
            "with" => Some(With),
            "pass" => Some(Pass),
            "del" => Some(Del),
            "None" => Some(None),
            "True" => Some(True),
            "False" => Some(False),
            "budget" => Some(Budget),
            "model" => Some(Model),
            "context" => Some(Context),
            "retrieve" => Some(Retrieve),
            "store" => Some(Store),
            "compress" => Some(Compress),
            "stream" => Some(Stream),
            "invoke" => Some(Invoke),
            "sandbox" => Some(Sandbox),
            "audit" => Some(Audit),
            "permit" => Some(Permit),
            "deny" => Some(Deny),
            "policy" => Some(Policy),
            "trace" => Some(Trace),
            "team" => Some(Team),
            "on" => Some(On),
            _ => Option::None,
        }
    }
}
