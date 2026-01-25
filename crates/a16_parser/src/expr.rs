//! Expression parsing with Pratt parser for precedence

use crate::parser::Parser;
use crate::error::{ParseError, ParseResult};
use a16_ast::*;
use a16_lexer::TokenKind;
use smol_str::SmolStr;

/// Binding power for Pratt parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct BindingPower(u8);

impl Parser<'_> {
    /// Parse an expression
    pub fn parse_expression(&mut self) -> ParseResult<Expr> {
        self.parse_expr_bp(BindingPower(0))
    }
    
    /// Pratt parser with binding power
    fn parse_expr_bp(&mut self, min_bp: BindingPower) -> ParseResult<Expr> {
        let mut lhs = self.parse_prefix()?;
        
        loop {
            // Postfix operators (call, subscript, attribute)
            lhs = match self.peek() {
                TokenKind::LParen => self.parse_call(lhs)?,
                TokenKind::LBracket => self.parse_subscript(lhs)?,
                TokenKind::Dot => self.parse_attribute(lhs)?,
                _ => lhs,
            };
            
            // Infix operators
            let Some((l_bp, r_bp)) = self.infix_bp() else {
                break;
            };
            
            if l_bp < min_bp {
                break;
            }
            
            lhs = self.parse_infix(lhs, r_bp)?;
        }
        
        // Ternary conditional: x if cond else y
        if self.at(TokenKind::If) {
            lhs = self.parse_if_expr(lhs)?;
        }
        
        Ok(lhs)
    }
    
    /// Parse prefix expression
    fn parse_prefix(&mut self) -> ParseResult<Expr> {
        match self.peek() {
            // Unary operators
            TokenKind::Minus => {
                let start = self.span();
                self.advance();
                let operand = self.parse_expr_bp(BindingPower(15))?;
                Ok(Expr::Unary(UnaryExpr {
                    op: UnaryOp::Neg,
                    operand: Box::new(operand),
                    span: start.merge(self.span()),
                }))
            }
            TokenKind::Plus => {
                let start = self.span();
                self.advance();
                let operand = self.parse_expr_bp(BindingPower(15))?;
                Ok(Expr::Unary(UnaryExpr {
                    op: UnaryOp::Pos,
                    operand: Box::new(operand),
                    span: start.merge(self.span()),
                }))
            }
            TokenKind::Not => {
                let start = self.span();
                self.advance();
                let operand = self.parse_expr_bp(BindingPower(5))?;
                Ok(Expr::Unary(UnaryExpr {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                    span: start.merge(self.span()),
                }))
            }
            TokenKind::Tilde => {
                let start = self.span();
                self.advance();
                let operand = self.parse_expr_bp(BindingPower(15))?;
                Ok(Expr::Unary(UnaryExpr {
                    op: UnaryOp::BitNot,
                    operand: Box::new(operand),
                    span: start.merge(self.span()),
                }))
            }
            
            // Await
            TokenKind::Await => {
                let start = self.span();
                self.advance();
                let value = self.parse_expr_bp(BindingPower(15))?;
                Ok(Expr::Await(AwaitExpr {
                    value: Box::new(value),
                    span: start.merge(self.span()),
                }))
            }
            
            // Spawn
            TokenKind::Spawn => {
                let start = self.span();
                self.advance();
                let value = self.parse_expr_bp(BindingPower(1))?;
                Ok(Expr::Spawn(SpawnExpr {
                    value: Box::new(value),
                    span: start.merge(self.span()),
                }))
            }
            
            // Grouped expression or tuple
            TokenKind::LParen => self.parse_paren_expr(),
            
            // List or list comprehension
            TokenKind::LBracket => self.parse_list_expr(),
            
            // Dict or set
            TokenKind::LBrace => self.parse_brace_expr(),
            
            // Atoms
            _ => self.parse_atom(),
        }
    }
    
    /// Parse atomic expression
    fn parse_atom(&mut self) -> ParseResult<Expr> {
        match self.peek() {
            TokenKind::Int => {
                let token = self.advance();
                let text = token.text.replace('_', "");
                let value: i64 = text.parse().map_err(|_| ParseError::InvalidNumber {
                    message: "invalid integer".to_string(),
                    span: self.span(),
                })?;
                Ok(Expr::Int(value, self.span()))
            }
            TokenKind::Float => {
                let token = self.advance();
                let text = token.text.replace('_', "");
                let value: f64 = text.parse().map_err(|_| ParseError::InvalidNumber {
                    message: "invalid float".to_string(),
                    span: self.span(),
                })?;
                Ok(Expr::Float(value, self.span()))
            }
            TokenKind::String => {
                let token = self.advance();
                let text = self.unescape_string(&token.text);
                Ok(Expr::String(text, self.span()))
            }
            TokenKind::True => {
                let span = self.span();
                self.advance();
                Ok(Expr::Bool(true, span))
            }
            TokenKind::False => {
                let span = self.span();
                self.advance();
                Ok(Expr::Bool(false, span))
            }
            TokenKind::None => {
                let span = self.span();
                self.advance();
                Ok(Expr::None(span))
            }
            TokenKind::Ident => {
                let ident = self.parse_ident()?;
                Ok(Expr::Ident(ident))
            }
            _ => Err(ParseError::ExpectedExpression { span: self.span() }),
        }
    }
    
    /// Parse parenthesized expression or tuple
    fn parse_paren_expr(&mut self) -> ParseResult<Expr> {
        let start = self.span();
        self.expect(TokenKind::LParen)?;
        
        if self.at(TokenKind::RParen) {
            // Empty tuple
            self.advance();
            return Ok(Expr::Tuple(TupleExpr {
                elements: Vec::new(),
                span: start.merge(self.span()),
            }));
        }
        
        let first = self.parse_expression()?;
        
        if self.at(TokenKind::Comma) {
            // Tuple
            let mut elements = vec![first];
            while self.consume_if(TokenKind::Comma) {
                if self.at(TokenKind::RParen) {
                    break;
                }
                elements.push(self.parse_expression()?);
            }
            self.expect(TokenKind::RParen)?;
            Ok(Expr::Tuple(TupleExpr {
                elements,
                span: start.merge(self.span()),
            }))
        } else {
            // Grouped expression
            self.expect(TokenKind::RParen)?;
            Ok(first)
        }
    }
    
    /// Parse list literal or comprehension
    fn parse_list_expr(&mut self) -> ParseResult<Expr> {
        let start = self.span();
        self.expect(TokenKind::LBracket)?;
        
        if self.at(TokenKind::RBracket) {
            self.advance();
            return Ok(Expr::List(ListExpr {
                elements: Vec::new(),
                span: start.merge(self.span()),
            }));
        }
        
        let first = self.parse_expression()?;
        
        // Check for comprehension
        if self.at(TokenKind::For) {
            return self.parse_list_comprehension(first, start);
        }
        
        // Regular list
        let mut elements = vec![first];
        while self.consume_if(TokenKind::Comma) {
            if self.at(TokenKind::RBracket) {
                break;
            }
            elements.push(self.parse_expression()?);
        }
        
        self.expect(TokenKind::RBracket)?;
        Ok(Expr::List(ListExpr {
            elements,
            span: start.merge(self.span()),
        }))
    }
    
    fn parse_list_comprehension(&mut self, element: Expr, start: Span) -> ParseResult<Expr> {
        let generators = self.parse_generators()?;
        self.expect(TokenKind::RBracket)?;
        
        Ok(Expr::ListComp(ComprehensionExpr {
            element: Box::new(element),
            generators,
            span: start.merge(self.span()),
        }))
    }
    
    fn parse_generators(&mut self) -> ParseResult<Vec<Generator>> {
        let mut generators = Vec::new();
        
        while self.at(TokenKind::For) || self.at(TokenKind::Async) {
            let is_async = self.consume_if(TokenKind::Async);
            self.expect(TokenKind::For)?;
            
            let target = self.parse_pattern()?;
            self.expect(TokenKind::In)?;
            let iter = self.parse_expression()?;
            
            let mut conditions = Vec::new();
            while self.consume_if(TokenKind::If) {
                conditions.push(self.parse_expression()?);
            }
            
            generators.push(Generator {
                target,
                iter,
                conditions,
                is_async,
            });
        }
        
        Ok(generators)
    }
    
    /// Parse dict or set
    fn parse_brace_expr(&mut self) -> ParseResult<Expr> {
        let start = self.span();
        self.expect(TokenKind::LBrace)?;
        
        if self.at(TokenKind::RBrace) {
            // Empty dict
            self.advance();
            return Ok(Expr::Dict(DictExpr {
                pairs: Vec::new(),
                span: start.merge(self.span()),
            }));
        }
        
        let first = self.parse_expression()?;
        
        if self.at(TokenKind::Colon) {
            // Dict
            self.advance();
            let first_value = self.parse_expression()?;
            let mut pairs = vec![(first, first_value)];
            
            while self.consume_if(TokenKind::Comma) {
                if self.at(TokenKind::RBrace) {
                    break;
                }
                let key = self.parse_expression()?;
                self.expect(TokenKind::Colon)?;
                let value = self.parse_expression()?;
                pairs.push((key, value));
            }
            
            self.expect(TokenKind::RBrace)?;
            Ok(Expr::Dict(DictExpr {
                pairs,
                span: start.merge(self.span()),
            }))
        } else {
            // Set
            let mut elements = vec![first];
            while self.consume_if(TokenKind::Comma) {
                if self.at(TokenKind::RBrace) {
                    break;
                }
                elements.push(self.parse_expression()?);
            }
            
            self.expect(TokenKind::RBrace)?;
            Ok(Expr::Set(SetExpr {
                elements,
                span: start.merge(self.span()),
            }))
        }
    }
    
    /// Parse lambda expression
    #[allow(dead_code)]
    fn parse_lambda(&mut self) -> ParseResult<Expr> {
        let start = self.span();
        
        self.expect(TokenKind::LParen)?;
        let params = self.parse_parameters()?;
        self.expect(TokenKind::RParen)?;
        
        self.expect(TokenKind::FatArrow)?;
        let body = self.parse_expression()?;
        
        Ok(Expr::Lambda(LambdaExpr {
            params,
            body: Box::new(body),
            span: start.merge(self.span()),
        }))
    }
    
    #[allow(dead_code)]
    fn is_lambda(&self) -> bool {
        // Look ahead to see if this is a lambda
        let mut depth = 1;
        let mut pos = self.pos + 1;
        
        while pos < self.tokens.len() && depth > 0 {
            match self.tokens[pos].kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => depth -= 1,
                _ => {}
            }
            pos += 1;
        }
        
        // Check if => follows the closing paren
        pos < self.tokens.len() && self.tokens[pos].kind == TokenKind::FatArrow
    }
    
    /// Parse function call
    fn parse_call(&mut self, func: Expr) -> ParseResult<Expr> {
        let start = func.span();
        self.expect(TokenKind::LParen)?;
        
        let mut args = Vec::new();
        while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
            // Check for keyword argument
            let name = if self.at(TokenKind::Ident) && self.peek_ahead(1) == TokenKind::Assign {
                let ident = self.parse_ident()?;
                self.advance(); // consume =
                Some(ident)
            } else {
                None
            };
            
            let value = self.parse_expression()?;
            args.push(Arg { name, value });
            
            if !self.consume_if(TokenKind::Comma) {
                break;
            }
        }
        
        self.expect(TokenKind::RParen)?;
        
        Ok(Expr::Call(CallExpr {
            func: Box::new(func),
            args,
            span: start.merge(self.span()),
        }))
    }
    
    /// Parse subscript (a[b])
    fn parse_subscript(&mut self, value: Expr) -> ParseResult<Expr> {
        let start = value.span();
        self.expect(TokenKind::LBracket)?;
        let index = self.parse_expression()?;
        self.expect(TokenKind::RBracket)?;
        
        Ok(Expr::Subscript(SubscriptExpr {
            value: Box::new(value),
            index: Box::new(index),
            span: start.merge(self.span()),
        }))
    }
    
    /// Parse attribute access (a.b)
    fn parse_attribute(&mut self, value: Expr) -> ParseResult<Expr> {
        let start = value.span();
        self.expect(TokenKind::Dot)?;
        let attr = self.parse_ident()?;
        
        Ok(Expr::Attribute(AttributeExpr {
            value: Box::new(value),
            attr,
            span: start.merge(self.span()),
        }))
    }
    
    /// Parse infix expression
    fn parse_infix(&mut self, lhs: Expr, r_bp: BindingPower) -> ParseResult<Expr> {
        let start = lhs.span();
        let op = self.advance();
        
        match op.kind {
            // Binary operators
            TokenKind::Plus => self.make_binary(lhs, BinaryOp::Add, r_bp, start),
            TokenKind::Minus => self.make_binary(lhs, BinaryOp::Sub, r_bp, start),
            TokenKind::Star => self.make_binary(lhs, BinaryOp::Mul, r_bp, start),
            TokenKind::Slash => self.make_binary(lhs, BinaryOp::Div, r_bp, start),
            TokenKind::DoubleSlash => self.make_binary(lhs, BinaryOp::FloorDiv, r_bp, start),
            TokenKind::Percent => self.make_binary(lhs, BinaryOp::Mod, r_bp, start),
            TokenKind::DoubleStar => self.make_binary(lhs, BinaryOp::Pow, r_bp, start),
            TokenKind::Ampersand => self.make_binary(lhs, BinaryOp::BitAnd, r_bp, start),
            TokenKind::Pipe => self.make_binary(lhs, BinaryOp::BitOr, r_bp, start),
            TokenKind::Caret => self.make_binary(lhs, BinaryOp::BitXor, r_bp, start),
            TokenKind::LShift => self.make_binary(lhs, BinaryOp::Shl, r_bp, start),
            TokenKind::RShift => self.make_binary(lhs, BinaryOp::Shr, r_bp, start),
            TokenKind::And => self.make_binary(lhs, BinaryOp::And, r_bp, start),
            TokenKind::Or => self.make_binary(lhs, BinaryOp::Or, r_bp, start),
            
            // Comparison operators (chained)
            TokenKind::Eq | TokenKind::Ne | TokenKind::Lt | TokenKind::Le |
            TokenKind::Gt | TokenKind::Ge | TokenKind::In | TokenKind::Is => {
                self.parse_comparison(lhs, op.kind, start)
            }
            
            _ => Err(ParseError::InvalidSyntax {
                message: format!("unexpected operator: {:?}", op.kind),
                span: self.span(),
            }),
        }
    }
    
    fn make_binary(&mut self, lhs: Expr, op: BinaryOp, r_bp: BindingPower, start: Span) -> ParseResult<Expr> {
        let rhs = self.parse_expr_bp(r_bp)?;
        Ok(Expr::Binary(BinaryExpr {
            left: Box::new(lhs),
            op,
            right: Box::new(rhs),
            span: start.merge(self.span()),
        }))
    }
    
    fn parse_comparison(&mut self, lhs: Expr, first_op: TokenKind, start: Span) -> ParseResult<Expr> {
        let mut comparisons = Vec::new();
        
        let op = self.token_to_compare_op(first_op)?;
        let rhs = self.parse_expr_bp(BindingPower(6))?;
        comparisons.push((op, rhs));
        
        // Chain comparisons: a < b < c
        while self.is_comparison_op() {
            let op_token = self.advance().kind;
            let op = self.token_to_compare_op(op_token)?;
            let rhs = self.parse_expr_bp(BindingPower(6))?;
            comparisons.push((op, rhs));
        }
        
        Ok(Expr::Compare(CompareExpr {
            left: Box::new(lhs),
            comparisons,
            span: start.merge(self.span()),
        }))
    }
    
    fn is_comparison_op(&self) -> bool {
        matches!(
            self.peek(),
            TokenKind::Eq | TokenKind::Ne | TokenKind::Lt | TokenKind::Le |
            TokenKind::Gt | TokenKind::Ge | TokenKind::In | TokenKind::Is
        )
    }
    
    fn token_to_compare_op(&self, kind: TokenKind) -> ParseResult<CompareOp> {
        Ok(match kind {
            TokenKind::Eq => CompareOp::Eq,
            TokenKind::Ne => CompareOp::Ne,
            TokenKind::Lt => CompareOp::Lt,
            TokenKind::Le => CompareOp::Le,
            TokenKind::Gt => CompareOp::Gt,
            TokenKind::Ge => CompareOp::Ge,
            TokenKind::In => CompareOp::In,
            TokenKind::Is => CompareOp::Is,
            _ => return Err(ParseError::InvalidSyntax {
                message: format!("not a comparison operator: {:?}", kind),
                span: self.span(),
            }),
        })
    }
    
    /// Parse conditional expression: value if cond else other
    fn parse_if_expr(&mut self, then_expr: Expr) -> ParseResult<Expr> {
        let start = then_expr.span();
        self.expect(TokenKind::If)?;
        let condition = self.parse_expression()?;
        self.expect(TokenKind::Else)?;
        let else_expr = self.parse_expression()?;
        
        Ok(Expr::IfExpr(IfExprNode {
            condition: Box::new(condition),
            then_expr: Box::new(then_expr),
            else_expr: Box::new(else_expr),
            span: start.merge(self.span()),
        }))
    }
    
    /// Get infix binding power
    fn infix_bp(&self) -> Option<(BindingPower, BindingPower)> {
        Some(match self.peek() {
            // Logical or
            TokenKind::Or => (BindingPower(2), BindingPower(3)),
            // Logical and
            TokenKind::And => (BindingPower(4), BindingPower(5)),
            // Comparisons
            TokenKind::Eq | TokenKind::Ne | TokenKind::Lt | TokenKind::Le |
            TokenKind::Gt | TokenKind::Ge | TokenKind::In | TokenKind::Is => {
                (BindingPower(6), BindingPower(7))
            }
            // Bitwise or
            TokenKind::Pipe => (BindingPower(8), BindingPower(9)),
            // Bitwise xor
            TokenKind::Caret => (BindingPower(10), BindingPower(11)),
            // Bitwise and
            TokenKind::Ampersand => (BindingPower(12), BindingPower(13)),
            // Shifts
            TokenKind::LShift | TokenKind::RShift => (BindingPower(14), BindingPower(15)),
            // Add/Sub
            TokenKind::Plus | TokenKind::Minus => (BindingPower(16), BindingPower(17)),
            // Mul/Div/Mod
            TokenKind::Star | TokenKind::Slash | TokenKind::DoubleSlash | TokenKind::Percent => {
                (BindingPower(18), BindingPower(19))
            }
            // Power (right associative)
            TokenKind::DoubleStar => (BindingPower(21), BindingPower(20)),
            _ => return None,
        })
    }
    
    /// Parse pattern for destructuring
    pub fn parse_pattern(&mut self) -> ParseResult<Pattern> {
        match self.peek() {
            TokenKind::Ident => {
                let ident = self.parse_ident()?;
                Ok(Pattern::Ident(ident))
            }
            TokenKind::LParen => {
                let start = self.span();
                self.advance();
                let mut patterns = Vec::new();
                while !self.at(TokenKind::RParen) {
                    patterns.push(self.parse_pattern()?);
                    if !self.consume_if(TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen)?;
                Ok(Pattern::Tuple(patterns, start.merge(self.span())))
            }
            TokenKind::LBracket => {
                let start = self.span();
                self.advance();
                let mut patterns = Vec::new();
                while !self.at(TokenKind::RBracket) {
                    patterns.push(self.parse_pattern()?);
                    if !self.consume_if(TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RBracket)?;
                Ok(Pattern::List(patterns, start.merge(self.span())))
            }
            _ => {
                // Literal pattern
                let expr = self.parse_atom()?;
                Ok(Pattern::Literal(expr))
            }
        }
    }
    
    /// Unescape string literal
    fn unescape_string(&self, s: &str) -> SmolStr {
        // Remove quotes
        let inner = if s.starts_with("\"\"\"") || s.starts_with("'''") {
            &s[3..s.len()-3]
        } else {
            &s[1..s.len()-1]
        };
        
        // TODO: Handle escape sequences properly
        SmolStr::new(inner)
    }
}
