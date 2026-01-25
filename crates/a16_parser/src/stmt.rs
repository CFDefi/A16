//! Statement parsing

use crate::parser::Parser;
use crate::error::ParseResult;
use a16_ast::*;
use a16_lexer::TokenKind;

impl Parser<'_> {
    /// Parse a statement
    pub fn parse_statement(&mut self) -> ParseResult<Stmt> {
        match self.peek() {
            TokenKind::Let => self.parse_let(),
            TokenKind::Const => {
                let c = self.parse_const()?;
                Ok(Stmt::Let(LetStmt {
                    pattern: Pattern::Ident(c.name),
                    ty: c.ty,
                    value: Some(c.value),
                    is_const: true,
                    is_mutable: false,
                    span: c.span,
                }))
            }
            TokenKind::Return => self.parse_return(),
            TokenKind::If => self.parse_if(),
            TokenKind::For => self.parse_for(),
            TokenKind::While => self.parse_while(),
            TokenKind::Match => self.parse_match(),
            TokenKind::Try => self.parse_try(),
            TokenKind::With => self.parse_with(),
            TokenKind::Raise => self.parse_raise(),
            TokenKind::Assert => self.parse_assert(),
            TokenKind::Break => {
                let span = self.span();
                self.advance();
                self.consume_if(TokenKind::Newline);
                Ok(Stmt::Break(span))
            }
            TokenKind::Continue => {
                let span = self.span();
                self.advance();
                self.consume_if(TokenKind::Newline);
                Ok(Stmt::Continue(span))
            }
            TokenKind::Pass => {
                let span = self.span();
                self.advance();
                self.consume_if(TokenKind::Newline);
                Ok(Stmt::Pass(span))
            }
            TokenKind::Async => self.parse_async_block(),
            _ => self.parse_expr_or_assign(),
        }
    }
    
    /// Parse let statement
    fn parse_let(&mut self) -> ParseResult<Stmt> {
        let start = self.span();
        self.expect(TokenKind::Let)?;
        
        // Check for 'mut' keyword
        let is_mutable = if self.current().text.as_str() == "mut" {
            self.advance();
            true
        } else {
            false
        };
        
        let pattern = self.parse_pattern()?;
        
        let ty = if self.consume_if(TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        let value = if self.consume_if(TokenKind::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.consume_if(TokenKind::Newline);
        
        Ok(Stmt::Let(LetStmt {
            pattern,
            ty,
            value,
            is_const: false,
            is_mutable,
            span: start.merge(self.span()),
        }))
    }
    
    /// Parse return statement
    fn parse_return(&mut self) -> ParseResult<Stmt> {
        let start = self.span();
        self.expect(TokenKind::Return)?;
        
        let value = if !self.at(TokenKind::Newline) && !self.at(TokenKind::Eof) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.consume_if(TokenKind::Newline);
        
        Ok(Stmt::Return(ReturnStmt {
            value,
            span: start.merge(self.span()),
        }))
    }
    
    /// Parse if statement
    fn parse_if(&mut self) -> ParseResult<Stmt> {
        let start = self.span();
        self.expect(TokenKind::If)?;
        
        let condition = self.parse_expression()?;
        let then_block = self.parse_block()?;
        
        let mut elif_blocks = Vec::new();
        while self.at(TokenKind::Elif) {
            self.advance();
            let cond = self.parse_expression()?;
            let block = self.parse_block()?;
            elif_blocks.push((cond, block));
        }
        
        let else_block = if self.at(TokenKind::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        
        Ok(Stmt::If(IfStmt {
            condition,
            then_block,
            elif_blocks,
            else_block,
            span: start.merge(self.span()),
        }))
    }
    
    /// Parse for statement
    fn parse_for(&mut self) -> ParseResult<Stmt> {
        let start = self.span();
        self.expect(TokenKind::For)?;
        
        let target = self.parse_pattern()?;
        self.expect(TokenKind::In)?;
        let iter = self.parse_expression()?;
        let body = self.parse_block()?;
        
        let else_block = if self.at(TokenKind::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        
        Ok(Stmt::For(ForStmt {
            target,
            iter,
            body,
            else_block,
            span: start.merge(self.span()),
        }))
    }
    
    /// Parse while statement
    fn parse_while(&mut self) -> ParseResult<Stmt> {
        let start = self.span();
        self.expect(TokenKind::While)?;
        
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        
        let else_block = if self.at(TokenKind::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        
        Ok(Stmt::While(WhileStmt {
            condition,
            body,
            else_block,
            span: start.merge(self.span()),
        }))
    }
    
    /// Parse match statement
    fn parse_match(&mut self) -> ParseResult<Stmt> {
        let start = self.span();
        self.expect(TokenKind::Match)?;
        
        let subject = self.parse_expression()?;
        self.expect(TokenKind::Colon)?;
        self.expect(TokenKind::Newline)?;
        self.expect(TokenKind::Indent)?;
        
        let mut arms = Vec::new();
        while self.at(TokenKind::Case) {
            arms.push(self.parse_match_arm()?);
        }
        
        if self.at(TokenKind::Dedent) {
            self.advance();
        }
        
        Ok(Stmt::Match(MatchStmt {
            subject,
            arms,
            span: start.merge(self.span()),
        }))
    }
    
    fn parse_match_arm(&mut self) -> ParseResult<MatchArm> {
        let start = self.span();
        self.expect(TokenKind::Case)?;
        
        let pattern = self.parse_pattern()?;
        
        let guard = if self.consume_if(TokenKind::If) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        let body = self.parse_block()?;
        
        Ok(MatchArm {
            pattern,
            guard,
            body,
            span: start.merge(self.span()),
        })
    }
    
    /// Parse try statement
    fn parse_try(&mut self) -> ParseResult<Stmt> {
        let start = self.span();
        self.expect(TokenKind::Try)?;
        
        let body = self.parse_block()?;
        
        let mut handlers = Vec::new();
        while self.at(TokenKind::Except) {
            handlers.push(self.parse_except_handler()?);
        }
        
        let else_block = if self.at(TokenKind::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        
        let finally_block = if self.at(TokenKind::Finally) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        
        Ok(Stmt::Try(TryStmt {
            body,
            handlers,
            else_block,
            finally_block,
            span: start.merge(self.span()),
        }))
    }
    
    fn parse_except_handler(&mut self) -> ParseResult<ExceptHandler> {
        let start = self.span();
        self.expect(TokenKind::Except)?;
        
        let (ty, name) = if !self.at(TokenKind::Colon) {
            let ty = Some(self.parse_expression()?);
            let name = if self.consume_if(TokenKind::As) {
                Some(self.parse_ident()?)
            } else {
                None
            };
            (ty, name)
        } else {
            (None, None)
        };
        
        let body = self.parse_block()?;
        
        Ok(ExceptHandler {
            ty,
            name,
            body,
            span: start.merge(self.span()),
        })
    }
    
    /// Parse with statement
    fn parse_with(&mut self) -> ParseResult<Stmt> {
        let start = self.span();
        self.expect(TokenKind::With)?;
        
        let mut items = Vec::new();
        loop {
            let context = self.parse_expression()?;
            let alias = if self.consume_if(TokenKind::As) {
                Some(self.parse_ident()?)
            } else {
                None
            };
            items.push(WithItem { context, alias });
            
            if !self.consume_if(TokenKind::Comma) {
                break;
            }
        }
        
        let body = self.parse_block()?;
        
        Ok(Stmt::With(WithStmt {
            items,
            body,
            span: start.merge(self.span()),
        }))
    }
    
    /// Parse raise statement
    fn parse_raise(&mut self) -> ParseResult<Stmt> {
        let start = self.span();
        self.expect(TokenKind::Raise)?;
        
        let exception = if !self.at(TokenKind::Newline) && !self.at(TokenKind::Eof) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        // TODO: from clause
        
        self.consume_if(TokenKind::Newline);
        
        Ok(Stmt::Raise(RaiseStmt {
            exception,
            cause: None,
            span: start.merge(self.span()),
        }))
    }
    
    /// Parse assert statement
    fn parse_assert(&mut self) -> ParseResult<Stmt> {
        let start = self.span();
        self.expect(TokenKind::Assert)?;
        
        let test = self.parse_expression()?;
        
        let msg = if self.consume_if(TokenKind::Comma) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.consume_if(TokenKind::Newline);
        
        Ok(Stmt::Assert(AssertStmt {
            test,
            msg,
            span: start.merge(self.span()),
        }))
    }
    
    /// Parse async block
    fn parse_async_block(&mut self) -> ParseResult<Stmt> {
        let start = self.span();
        self.expect(TokenKind::Async)?;
        
        // Check for parallel block
        let is_parallel = if self.at(TokenKind::Colon) {
            false
        } else if self.current().text.as_str() == "parallel" {
            self.advance();
            true
        } else {
            false
        };
        
        self.expect(TokenKind::Colon)?;
        self.expect(TokenKind::Newline)?;
        self.expect(TokenKind::Indent)?;
        
        let mut stmts = Vec::new();
        while !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
            self.skip_newlines();
            if self.at(TokenKind::Dedent) {
                break;
            }
            stmts.push(self.parse_statement()?);
        }
        
        if self.at(TokenKind::Dedent) {
            self.advance();
        }
        
        Ok(Stmt::Async(AsyncBlock {
            stmts,
            is_parallel,
            span: start.merge(self.span()),
        }))
    }
    
    /// Parse expression statement or assignment
    fn parse_expr_or_assign(&mut self) -> ParseResult<Stmt> {
        let start = self.span();
        let expr = self.parse_expression()?;
        
        // Check for assignment
        if self.at(TokenKind::Assign) {
            self.advance();
            let value = self.parse_expression()?;
            self.consume_if(TokenKind::Newline);
            return Ok(Stmt::Assign(AssignStmt {
                target: expr,
                value,
                span: start.merge(self.span()),
            }));
        }
        
        // Check for augmented assignment
        if let Some(op) = self.aug_assign_op() {
            self.advance();
            let value = self.parse_expression()?;
            self.consume_if(TokenKind::Newline);
            return Ok(Stmt::AugAssign(AugAssignStmt {
                target: expr,
                op,
                value,
                span: start.merge(self.span()),
            }));
        }
        
        self.consume_if(TokenKind::Newline);
        
        Ok(Stmt::Expr(ExprStmt {
            expr,
            span: start.merge(self.span()),
        }))
    }
    
    fn aug_assign_op(&self) -> Option<AugOp> {
        Some(match self.peek() {
            TokenKind::PlusEq => AugOp::Add,
            TokenKind::MinusEq => AugOp::Sub,
            TokenKind::StarEq => AugOp::Mul,
            TokenKind::SlashEq => AugOp::Div,
            TokenKind::DoubleSlashEq => AugOp::FloorDiv,
            TokenKind::PercentEq => AugOp::Mod,
            TokenKind::DoubleStarEq => AugOp::Pow,
            TokenKind::AmpEq => AugOp::BitAnd,
            TokenKind::PipeEq => AugOp::BitOr,
            TokenKind::CaretEq => AugOp::BitXor,
            TokenKind::LShiftEq => AugOp::Shl,
            TokenKind::RShiftEq => AugOp::Shr,
            _ => return None,
        })
    }
}
