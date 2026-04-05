//! Core parser implementation

use a16_ast::*;
use a16_lexer::{Lexer, Token, TokenKind};
use crate::error::{ParseError, ParseResult};

/// A16 Parser
pub struct Parser<'src> {
    #[allow(dead_code)]
    pub(crate) source: &'src str,
    pub(crate) tokens: Vec<Token>,
    pub(crate) pos: usize,
}

impl<'src> Parser<'src> {
    /// Create a new parser
    pub fn new(source: &'src str) -> Self {
        let tokens = Lexer::new(source).tokenize();
        Self {
            source,
            tokens,
            pos: 0,
        }
    }
    
    // =========================================================================
    // TOKEN HELPERS
    // =========================================================================
    
    pub(crate) fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&self.tokens[self.tokens.len() - 1])
    }
    
    pub(crate) fn peek(&self) -> TokenKind {
        self.current().kind
    }
    
    pub(crate) fn peek_ahead(&self, n: usize) -> TokenKind {
        self.tokens.get(self.pos + n).map(|t| t.kind).unwrap_or(TokenKind::Eof)
    }
    
    pub(crate) fn at(&self, kind: TokenKind) -> bool {
        self.peek() == kind
    }
    
    #[allow(dead_code)]
    pub(crate) fn at_any(&self, kinds: &[TokenKind]) -> bool {
        kinds.contains(&self.peek())
    }
    
    pub(crate) fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        token
    }
    
    pub(crate) fn expect(&mut self, kind: TokenKind) -> ParseResult<Token> {
        if self.at(kind) {
            Ok(self.advance())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", kind),
                found: format!("{:?}", self.peek()),
                span: self.span(),
            })
        }
    }
    
    pub(crate) fn consume_if(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    pub(crate) fn span(&self) -> Span {
        let t = self.current();
        Span::new(t.span.start, t.span.end)
    }
    
    pub(crate) fn skip_newlines(&mut self) {
        while self.at(TokenKind::Newline) {
            self.advance();
        }
    }
    
    // =========================================================================
    // TOP-LEVEL PARSING
    // =========================================================================
    
    /// Parse a complete module
    pub fn parse_module(&mut self) -> ParseResult<Module> {
        let start = self.span();
        let mut items = Vec::new();
        
        self.skip_newlines();
        
        while !self.at(TokenKind::Eof) {
            items.push(self.parse_item()?);
            self.skip_newlines();
        }
        
        let end = self.span();
        Ok(Module {
            items,
            span: start.merge(end),
        })
    }
    
    /// Parse a top-level item
    pub fn parse_item(&mut self) -> ParseResult<Item> {
        // Skip decorators for now
        let decorators = self.parse_decorators()?;
        
        match self.peek() {
            TokenKind::Fn => {
                let mut func = self.parse_function()?;
                func.decorators = decorators;
                Ok(Item::Function(func))
            }
            TokenKind::Async if self.peek_ahead(1) == TokenKind::Fn => {
                let mut func = self.parse_function()?;
                func.decorators = decorators;
                Ok(Item::Function(func))
            }
            TokenKind::Class => {
                let mut class = self.parse_class()?;
                class.decorators = decorators;
                Ok(Item::Class(class))
            }
            TokenKind::Agent => {
                let mut agent = self.parse_agent()?;
                agent.decorators = decorators;
                Ok(Item::Agent(agent))
            }
            TokenKind::Tool => {
                let mut tool = self.parse_tool()?;
                tool.decorators = decorators;
                Ok(Item::Tool(tool))
            }
            TokenKind::Struct => Ok(Item::Struct(self.parse_struct()?)),
            TokenKind::Enum => Ok(Item::Enum(self.parse_enum()?)),
            TokenKind::Import | TokenKind::From => Ok(Item::Import(self.parse_import()?)),
            TokenKind::Const => Ok(Item::Const(self.parse_const()?)),
            TokenKind::Extern => Ok(Item::Extern(self.parse_extern()?)),
            _ => {
                let stmt = self.parse_statement()?;
                Ok(Item::Stmt(stmt))
            }
        }
    }
    
    // =========================================================================
    // DECORATORS
    // =========================================================================
    
    fn parse_decorators(&mut self) -> ParseResult<Vec<Decorator>> {
        let mut decorators = Vec::new();
        
        while self.at(TokenKind::At) {
            let start = self.span();
            self.advance(); // @
            
            let name = self.parse_dotted_name()?;
            
            let args = if self.at(TokenKind::LParen) {
                self.advance();
                let args = self.parse_arguments()?;
                self.expect(TokenKind::RParen)?;
                args
            } else {
                Vec::new()
            };
            
            self.expect(TokenKind::Newline)?;
            
            decorators.push(Decorator {
                name,
                args,
                span: start.merge(self.span()),
            });
        }
        
        Ok(decorators)
    }
    
    // =========================================================================
    // IDENTIFIERS
    // =========================================================================
    
    pub fn parse_ident(&mut self) -> ParseResult<Ident> {
        if self.at(TokenKind::Ident) {
            let token = self.advance();
            Ok(Ident::new(token.text.clone(), self.token_span(&token)))
        } else {
            Err(ParseError::ExpectedIdentifier { span: self.span() })
        }
    }
    
    /// Parse an identifier, allowing keywords to be used as identifiers
    /// (useful for dotted paths like a16.ai.agent where agent is a keyword)
    pub fn parse_ident_or_keyword(&mut self) -> ParseResult<Ident> {
        let token = self.current();
        if token.kind == TokenKind::Ident || token.kind.is_keyword() {
            let token = self.advance();
            Ok(Ident::new(token.text.clone(), self.token_span(&token)))
        } else {
            Err(ParseError::ExpectedIdentifier { span: self.span() })
        }
    }
    
    pub(crate) fn token_span(&self, token: &Token) -> Span {
        Span::new(token.span.start, token.span.end)
    }
    
    pub fn parse_dotted_name(&mut self) -> ParseResult<DottedName> {
        let start = self.span();
        let mut parts = vec![self.parse_ident_or_keyword()?];
        
        while self.consume_if(TokenKind::Dot) {
            parts.push(self.parse_ident_or_keyword()?);
        }
        
        Ok(DottedName {
            parts,
            span: start.merge(self.span()),
        })
    }
    
    // =========================================================================
    // BLOCKS
    // =========================================================================
    
    pub fn parse_block(&mut self) -> ParseResult<Block> {
        let start = self.span();
        self.expect(TokenKind::Colon)?;
        self.expect(TokenKind::Newline)?;
        self.expect(TokenKind::Indent)?;
        
        let mut stmts = Vec::new();
        
        while !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
            self.skip_newlines();
            if self.at(TokenKind::Dedent) || self.at(TokenKind::Eof) {
                break;
            }
            stmts.push(self.parse_statement()?);
        }
        
        if self.at(TokenKind::Dedent) {
            self.advance();
        }
        
        Ok(Block {
            stmts,
            span: start.merge(self.span()),
        })
    }
    
    // =========================================================================
    // FUNCTIONS
    // =========================================================================
    
    pub fn parse_function(&mut self) -> ParseResult<FunctionDef> {
        let start = self.span();
        
        let is_async = self.consume_if(TokenKind::Async);
        self.expect(TokenKind::Fn)?;
        
        let name = self.parse_ident()?;
        
        self.expect(TokenKind::LParen)?;
        let params = self.parse_parameters()?;
        self.expect(TokenKind::RParen)?;
        
        let return_type = if self.consume_if(TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        let body = self.parse_block()?;
        
        Ok(FunctionDef {
            name,
            params,
            return_type,
            body,
            decorators: Vec::new(),
            is_async,
            span: start.merge(self.span()),
        })
    }
    
    pub(crate) fn parse_parameters(&mut self) -> ParseResult<Vec<Param>> {
        let mut params = Vec::new();
        
        while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
            let start = self.span();
            let name = self.parse_ident()?;
            
            let ty = if self.consume_if(TokenKind::Colon) {
                Some(self.parse_type()?)
            } else {
                None
            };
            
            let default = if self.consume_if(TokenKind::Assign) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            
            params.push(Param {
                name,
                ty,
                default,
                span: start.merge(self.span()),
            });
            
            if !self.consume_if(TokenKind::Comma) {
                break;
            }
        }
        
        Ok(params)
    }
    
    // =========================================================================
    // TYPES
    // =========================================================================
    
    pub fn parse_type(&mut self) -> ParseResult<TypeExpr> {
        let ty = self.parse_type_primary()?;
        
        // Handle union types: A | B
        if self.at(TokenKind::Pipe) {
            let mut types = vec![ty];
            while self.consume_if(TokenKind::Pipe) {
                types.push(self.parse_type_primary()?);
            }
            let span = types.first().unwrap().span().merge(types.last().unwrap().span());
            return Ok(TypeExpr::Union(types, span));
        }
        
        Ok(ty)
    }
    
    fn parse_type_primary(&mut self) -> ParseResult<TypeExpr> {
        let start = self.span();
        let ident = self.parse_ident()?;
        
        // Generic type: List[T]
        if self.at(TokenKind::LBracket) {
            self.advance();
            let mut args = vec![self.parse_type()?];
            while self.consume_if(TokenKind::Comma) {
                args.push(self.parse_type()?);
            }
            self.expect(TokenKind::RBracket)?;
            
            return Ok(TypeExpr::Generic(GenericType {
                name: ident,
                args,
                span: start.merge(self.span()),
            }));
        }
        
        Ok(TypeExpr::Name(ident))
    }
    
    // =========================================================================
    // ARGUMENTS (for function calls)
    // =========================================================================
    
    fn parse_arguments(&mut self) -> ParseResult<Vec<Expr>> {
        let mut args = Vec::new();
        
        while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
            args.push(self.parse_expression()?);
            if !self.consume_if(TokenKind::Comma) {
                break;
            }
        }
        
        Ok(args)
    }
    
    // =========================================================================
    // IMPORTS
    // =========================================================================
    
    fn parse_import(&mut self) -> ParseResult<ImportStmt> {
        let start = self.span();
        
        let kind = if self.at(TokenKind::Import) {
            self.advance();
            let path = self.parse_dotted_name()?;
            let alias = if self.consume_if(TokenKind::As) {
                Some(self.parse_ident()?)
            } else {
                None
            };
            ImportKind::Module { path, alias }
        } else {
            self.expect(TokenKind::From)?;
            let path = self.parse_dotted_name()?;
            self.expect(TokenKind::Import)?;
            
            let mut items = Vec::new();
            loop {
                let name = self.parse_ident()?;
                let alias = if self.consume_if(TokenKind::As) {
                    Some(self.parse_ident()?)
                } else {
                    None
                };
                items.push(ImportItem { name, alias });
                
                if !self.consume_if(TokenKind::Comma) {
                    break;
                }
            }
            ImportKind::From { path, items }
        };
        
        self.consume_if(TokenKind::Newline);
        
        Ok(ImportStmt {
            kind,
            span: start.merge(self.span()),
        })
    }
    
    pub(crate) fn parse_const(&mut self) -> ParseResult<ConstDef> {
        let start = self.span();
        self.expect(TokenKind::Const)?;
        let name = self.parse_ident()?;
        
        let ty = if self.consume_if(TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        self.expect(TokenKind::Assign)?;
        let value = self.parse_expression()?;
        
        Ok(ConstDef {
            name,
            ty,
            value,
            span: start.merge(self.span()),
        })
    }
    
    // =========================================================================
    // STRUCTS & ENUMS
    // =========================================================================
    
    fn parse_struct(&mut self) -> ParseResult<StructDef> {
        let start = self.span();
        self.expect(TokenKind::Struct)?;
        let name = self.parse_ident()?;
        self.expect(TokenKind::Colon)?;
        self.expect(TokenKind::Newline)?;
        self.expect(TokenKind::Indent)?;
        
        let mut fields = Vec::new();
        while !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
            self.skip_newlines();
            if self.at(TokenKind::Dedent) {
                break;
            }
            fields.push(self.parse_field()?);
            self.consume_if(TokenKind::Newline);
        }
        
        if self.at(TokenKind::Dedent) {
            self.advance();
        }
        
        Ok(StructDef {
            name,
            fields,
            span: start.merge(self.span()),
        })
    }
    
    fn parse_field(&mut self) -> ParseResult<FieldDef> {
        let start = self.span();
        let name = self.parse_ident()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type()?;
        
        let default = if self.consume_if(TokenKind::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        Ok(FieldDef {
            name,
            ty,
            default,
            span: start.merge(self.span()),
        })
    }
    
    fn parse_enum(&mut self) -> ParseResult<EnumDef> {
        let start = self.span();
        self.expect(TokenKind::Enum)?;
        let name = self.parse_ident()?;
        self.expect(TokenKind::Colon)?;
        self.expect(TokenKind::Newline)?;
        self.expect(TokenKind::Indent)?;
        
        let mut variants = Vec::new();
        while !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
            self.skip_newlines();
            if self.at(TokenKind::Dedent) {
                break;
            }
            let vstart = self.span();
            let vname = self.parse_ident()?;
            variants.push(EnumVariant {
                name: vname,
                fields: None,
                span: vstart.merge(self.span()),
            });
            self.consume_if(TokenKind::Newline);
        }
        
        if self.at(TokenKind::Dedent) {
            self.advance();
        }
        
        Ok(EnumDef {
            name,
            variants,
            span: start.merge(self.span()),
        })
    }
    
    // =========================================================================
    // CLASSES
    // =========================================================================
    
    pub fn parse_class(&mut self) -> ParseResult<ClassDef> {
        let start = self.span();
        self.expect(TokenKind::Class)?;
        let name = self.parse_ident()?;
        
        let bases = if self.at(TokenKind::LParen) {
            self.advance();
            let bases = self.parse_arguments()?;
            self.expect(TokenKind::RParen)?;
            bases
        } else {
            Vec::new()
        };
        
        self.expect(TokenKind::Colon)?;
        self.expect(TokenKind::Newline)?;
        self.expect(TokenKind::Indent)?;
        
        let mut body = Vec::new();
        while !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
            self.skip_newlines();
            if self.at(TokenKind::Dedent) {
                break;
            }
            body.push(self.parse_class_member()?);
        }
        
        if self.at(TokenKind::Dedent) {
            self.advance();
        }
        
        Ok(ClassDef {
            name,
            bases,
            body,
            decorators: Vec::new(),
            span: start.merge(self.span()),
        })
    }
    
    fn parse_class_member(&mut self) -> ParseResult<ClassMember> {
        match self.peek() {
            TokenKind::Fn | TokenKind::Async => {
                Ok(ClassMember::Method(self.parse_function()?))
            }
            TokenKind::Class => {
                Ok(ClassMember::Class(self.parse_class()?))
            }
            _ => {
                Ok(ClassMember::Field(self.parse_field()?))
            }
        }
    }
    
    // =========================================================================
    // AGENTS
    // =========================================================================
    
    pub fn parse_agent(&mut self) -> ParseResult<AgentDef> {
        let start = self.span();
        self.expect(TokenKind::Agent)?;
        let name = self.parse_ident()?;
        
        self.expect(TokenKind::Colon)?;
        self.expect(TokenKind::Newline)?;
        self.expect(TokenKind::Indent)?;
        
        let mut members = Vec::new();
        while !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
            self.skip_newlines();
            if self.at(TokenKind::Dedent) {
                break;
            }
            members.push(self.parse_agent_member()?);
        }
        
        if self.at(TokenKind::Dedent) {
            self.advance();
        }
        
        Ok(AgentDef {
            name,
            config: Vec::new(),
            members,
            decorators: Vec::new(),
            span: start.merge(self.span()),
        })
    }
    
    fn parse_agent_member(&mut self) -> ParseResult<AgentMember> {
        match self.peek() {
            TokenKind::Task | TokenKind::Async => {
                Ok(AgentMember::Task(self.parse_task()?))
            }
            TokenKind::Fn => {
                Ok(AgentMember::Method(self.parse_function()?))
            }
            TokenKind::On => {
                Ok(AgentMember::OnEvent(self.parse_on_event()?))
            }
            // Config: model, memory, tools, budget, policy
            TokenKind::Model | TokenKind::Memory | TokenKind::Budget | TokenKind::Policy => {
                Ok(AgentMember::Config(self.parse_agent_config()?))
            }
            TokenKind::Ident => {
                // Could be tools: [...] or other config
                let name = self.current().text.as_str();
                if name == "tools" || name == "model" || name == "memory" || name == "budget" {
                    Ok(AgentMember::Config(self.parse_agent_config()?))
                } else {
                    Ok(AgentMember::Method(self.parse_function()?))
                }
            }
            _ => Err(ParseError::InvalidSyntax {
                message: "expected agent member".to_string(),
                span: self.span(),
            })
        }
    }
    
    fn parse_task(&mut self) -> ParseResult<TaskDef> {
        let start = self.span();
        let is_async = self.consume_if(TokenKind::Async);
        self.expect(TokenKind::Task)?;
        
        let name = self.parse_ident()?;
        
        self.expect(TokenKind::LParen)?;
        let params = self.parse_parameters()?;
        self.expect(TokenKind::RParen)?;
        
        let return_type = if self.consume_if(TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        let body = self.parse_block()?;
        
        Ok(TaskDef {
            name,
            params,
            return_type,
            body,
            is_async: is_async || true, // Tasks are always async
            span: start.merge(self.span()),
        })
    }
    
    fn parse_on_event(&mut self) -> ParseResult<OnEventDef> {
        let start = self.span();
        self.expect(TokenKind::On)?;
        let event = self.parse_ident()?;
        
        self.expect(TokenKind::LParen)?;
        let params = self.parse_parameters()?;
        self.expect(TokenKind::RParen)?;
        
        let body = self.parse_block()?;
        
        Ok(OnEventDef {
            event,
            params,
            body,
            span: start.merge(self.span()),
        })
    }
    
    fn parse_agent_config(&mut self) -> ParseResult<AgentConfig> {
        let name = self.parse_ident_or_keyword()?;
        self.expect(TokenKind::Colon)?;
        
        match name.name.as_str() {
            "model" => {
                let expr = self.parse_expression()?;
                self.consume_if(TokenKind::Newline);
                Ok(AgentConfig::Model(expr))
            }
            "memory" => {
                self.expect(TokenKind::LBracket)?;
                let mut items = Vec::new();
                while !self.at(TokenKind::RBracket) {
                    items.push(self.parse_expression()?);
                    if !self.consume_if(TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RBracket)?;
                self.consume_if(TokenKind::Newline);
                Ok(AgentConfig::Memory(items))
            }
            "tools" => {
                self.expect(TokenKind::LBracket)?;
                let mut items = Vec::new();
                while !self.at(TokenKind::RBracket) {
                    items.push(self.parse_expression()?);
                    if !self.consume_if(TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RBracket)?;
                self.consume_if(TokenKind::Newline);
                Ok(AgentConfig::Tools(items))
            }
            "budget" => {
                let mut items = Vec::new();
                loop {
                    let bname = self.parse_ident()?;
                    self.expect(TokenKind::Assign)?;
                    let value = self.parse_expression()?;
                    items.push(BudgetItem { name: bname, value });
                    if !self.consume_if(TokenKind::Comma) {
                        break;
                    }
                }
                self.consume_if(TokenKind::Newline);
                Ok(AgentConfig::Budget(items))
            }
            _ => Err(ParseError::InvalidSyntax {
                message: format!("unknown agent config: {}", name.name),
                span: name.span,
            })
        }
    }
    
    // =========================================================================
    // TOOLS
    // =========================================================================
    
    pub fn parse_tool(&mut self) -> ParseResult<ToolDef> {
        let start = self.span();
        self.expect(TokenKind::Tool)?;
        let name = self.parse_ident()?;
        
        self.expect(TokenKind::Colon)?;
        self.expect(TokenKind::Newline)?;
        self.expect(TokenKind::Indent)?;
        
        let mut members = Vec::new();
        while !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
            self.skip_newlines();
            if self.at(TokenKind::Dedent) {
                break;
            }
            members.push(self.parse_tool_member()?);
        }
        
        if self.at(TokenKind::Dedent) {
            self.advance();
        }
        
        Ok(ToolDef {
            name,
            members,
            decorators: Vec::new(),
            span: start.merge(self.span()),
        })
    }
    
    fn parse_tool_member(&mut self) -> ParseResult<ToolMember> {
        if self.at(TokenKind::Fn) {
            return Ok(ToolMember::Method(self.parse_function()?));
        }
        
        let name = self.parse_ident_or_keyword()?;
        self.expect(TokenKind::Colon)?;
        
        match name.name.as_str() {
            "permissions" => {
                self.expect(TokenKind::LBracket)?;
                let mut perms = Vec::new();
                while !self.at(TokenKind::RBracket) {
                    perms.push(self.parse_ident_or_keyword()?);
                    if !self.consume_if(TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RBracket)?;
                self.consume_if(TokenKind::Newline);
                Ok(ToolMember::Permissions(perms))
            }
            "sandbox" => {
                let val = self.parse_ident_or_keyword()?;
                self.consume_if(TokenKind::Newline);
                Ok(ToolMember::Sandbox(val))
            }
            "rate_limit" => {
                let val = self.parse_expression()?;
                self.consume_if(TokenKind::Newline);
                Ok(ToolMember::RateLimit(val))
            }
            "audit" => {
                let val = self.parse_ident_or_keyword()?;
                self.consume_if(TokenKind::Newline);
                Ok(ToolMember::Audit(val))
            }
            "schema" => {
                let ty = self.parse_type()?;
                self.consume_if(TokenKind::Newline);
                Ok(ToolMember::Schema(ty))
            }
            _ => Err(ParseError::InvalidSyntax {
                message: format!("unknown tool member: {}", name.name),
                span: name.span,
            })
        }
    }
    
    // =========================================================================
    // EXTERN BLOCKS (FFI)
    // =========================================================================
    
    /// Parse an extern block: `extern "lib_name":`
    pub fn parse_extern(&mut self) -> ParseResult<ExternBlock> {
        let start = self.span();
        self.expect(TokenKind::Extern)?;
        
        // Expect library name as string literal
        let lib_name = if self.at(TokenKind::String) {
            let token = self.advance();
            // Strip quotes from the string literal
            let s = token.text.as_str();
            let trimmed = s.trim_matches('"').trim_matches('\'');
            smol_str::SmolStr::new(trimmed)
        } else {
            return Err(ParseError::InvalidSyntax {
                message: "expected string literal for library name after 'extern'".to_string(),
                span: self.span(),
            });
        };
        
        self.expect(TokenKind::Colon)?;
        self.expect(TokenKind::Newline)?;
        self.expect(TokenKind::Indent)?;
        
        let mut functions = Vec::new();
        while !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
            self.skip_newlines();
            if self.at(TokenKind::Dedent) || self.at(TokenKind::Eof) {
                break;
            }
            functions.push(self.parse_extern_func()?);
            self.consume_if(TokenKind::Newline);
        }
        
        if self.at(TokenKind::Dedent) {
            self.advance();
        }
        
        Ok(ExternBlock {
            lib_name,
            functions,
            span: start.merge(self.span()),
        })
    }
    
    /// Parse a single extern function declaration (no body)
    fn parse_extern_func(&mut self) -> ParseResult<ExternFunc> {
        let start = self.span();
        self.expect(TokenKind::Fn)?;
        
        let name = self.parse_ident()?;
        
        self.expect(TokenKind::LParen)?;
        let params = self.parse_parameters()?;
        self.expect(TokenKind::RParen)?;
        
        let return_type = if self.consume_if(TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        Ok(ExternFunc {
            name,
            params,
            return_type,
            span: start.merge(self.span()),
        })
    }
}
