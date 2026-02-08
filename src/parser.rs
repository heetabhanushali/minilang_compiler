// src/parser.rs - Recursive descent parser

use crate::ast::*;
use crate::errors::ParserError;
use crate::lexer::{Token, TokenWithSpan};
use std::collections::VecDeque;

/// The parser struct
pub struct Parser {
    tokens: VecDeque<TokenWithSpan>,
    current: usize,
    _source: String,
}

impl Parser {
    /// Create a new parser from tokens
    pub fn new(tokens: Vec<TokenWithSpan>, source: String) -> Self {
        Self {
            tokens: tokens.into(),
            current: 0,
            _source : source,
        }
    }
    
    /// Parse a complete program
    pub fn parse_program(&mut self) -> Result<Program, ParserError> {
        let mut functions = Vec::new();
        
        while !self.is_at_end() {
            functions.push(self.parse_function()?);
        }
        
        Ok(Program { functions })
    }
    
    /// Parse a function definition
    fn parse_function(&mut self) -> Result<Function, ParserError> {
        let start = self.current_span().start;
        
        // Expect 'func' keyword
        self.expect_token(Token::Func)?;
        
        // Get function name
        let name = self.expect_identifier()?;
        
        // Parse parameter list
        self.expect_token(Token::LeftParen)?;
        let params = self.parse_parameters()?;
        self.expect_token(Token::RightParen)?;
        
        // Parse optional return type
        let return_type = if self.match_token(&Token::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        // Parse function body
        let body = self.parse_block()?;
        
        let end = self.previous_span().end;
        
        Ok(Function {
            name,
            params,
            return_type,
            body,
            span: Span::new(start, end),
        })
    }
    
    /// Parse function parameters
    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, ParserError> {
        let mut params = Vec::new();
        
        // Empty parameter list
        if self.check(&Token::RightParen) {
            return Ok(params);
        }
        
        loop {
            let start = self.current_span().start;
            let name = self.expect_identifier()?;
            self.expect_token(Token::Colon)?;
            let typ = self.parse_type()?;
            let end = self.previous_span().end;
            
            params.push(Parameter {
                name,
                typ,
                span: Span::new(start, end),
            });
            
            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        
        Ok(params)
    }
    
    /// Parse a type annotation
    fn parse_type(&mut self) -> Result<Type, ParserError> {
        let typ = match self.advance() {
            Some(TokenWithSpan { token: Token::TypeInt, .. }) => Type::Int,
            Some(TokenWithSpan { token: Token::TypeFloat, .. }) => Type::Float,
            Some(TokenWithSpan { token: Token::TypeString, .. }) => Type::String,
            Some(TokenWithSpan { token: Token::TypeBool, .. }) => Type::Bool,
            Some(token) => {
                return Err(ParserError::UnexpectedToken {
                    expected: "type".to_string(),
                    found: format!("{:?}", token.token),
                    span: miette::SourceSpan::from(token.span.start..token.span.end),
                });
            }
            None => return Err(ParserError::UnexpectedEof {
                expected: "type".to_string(),
            }),
        };
        
        // Check for array type
        if self.match_token(&Token::LeftBracket) {
            let size = self.expect_integer()?;
            self.expect_token(Token::RightBracket)?;
            Ok(Type::Array(Box::new(typ), size as usize))
        } else {
            Ok(typ)
        }
    }
    
    /// Parse a block of statements
    fn parse_block(&mut self) -> Result<Block, ParserError> {
        let start = self.current_span().start;
        self.expect_token(Token::LeftBrace)?;
        
        let mut statements = Vec::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        
        self.expect_token(Token::RightBrace)?;
        let end = self.previous_span().end;
        
        Ok(Block {
            statements,
            span: Span::new(start, end),
        })
    }

    /// Parse a string literal and detect interpolation
    fn parse_string_literal(&mut self, string_value: &str) -> Result<Literal, ParserError> {
        // No {} means regular string
        if !string_value.contains('{') {
            return Ok(Literal::String(string_value.to_string()));
        }
        
        // Has {} so we need to parse it
        let mut parts = Vec::new();
        let mut current_text = String::new();
        let mut chars = string_value.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Save text before {
                if !current_text.is_empty() {
                    parts.push(StringPart::Text(current_text.clone()));
                    current_text.clear();
                }
                
                // Get everything between { and }
                let mut expr_text = String::new();
                let mut depth = 1;
                
                while let Some(ch) = chars.next() {
                    if ch == '{' {
                        depth += 1;
                        expr_text.push(ch);
                    } else if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        expr_text.push(ch);
                    } else {
                        expr_text.push(ch);
                    }
                }
                
                if expr_text.is_empty() {
                    return Err(ParserError::InvalidExpression {
                        span: (0..1).into(),
                    });
                }
                
                // Parse the expression using the standalone function
                let expr = self.parse_interpolation_expression(&expr_text)?;
                parts.push(StringPart::Expression(expr));
                
            } else {
                current_text.push(ch);
            }
        }
        
        // Save any remaining text
        if !current_text.is_empty() {
            parts.push(StringPart::Text(current_text));
        }
        
        Ok(Literal::InterpolatedString(parts))
    }

    /// Helper to parse expressions inside string interpolation
    fn parse_interpolation_expression(&mut self, expr_text: &str) -> Result<Expression, ParserError> {
        let trimmed = expr_text.trim();
        
        // Simple case: just a variable name like {name}
        if trimmed.chars().all(|c| c.is_alphanumeric() || c == '_') && !trimmed.is_empty() {
            return Ok(Expression::Identifier(IdentifierExpr {
                name: trimmed.to_string(),
                span: Span::default(),
            }));
        }
        
        // Complex case: expressions like {x + y}
        // We create a NEW parser instance, not using self
        let mut lexer = crate::Lexer::new(expr_text);
        let tokens = lexer.tokenize().map_err(|_| ParserError::InvalidExpression {
            span: (0..1).into(),
        })?;
        
        let mut parser = Parser::new(tokens, expr_text.to_string());
        parser.parse_expression()
    }

        
    /// Parse a statement
    fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        // Check what kind of statement this is
        if self.match_token(&Token::Const) {
            return Ok(Statement::Const(self.parse_const_statement()?));
        }

        if self.match_token(&Token::Let) {
            return Ok(Statement::Let(self.parse_let_statement()?));
        }
        
        if self.match_token(&Token::Display) {
            return Ok(Statement::Display(self.parse_display_statement()?));
        }
        
        if self.match_token(&Token::If) {
            return Ok(Statement::If(self.parse_if_statement()?));
        }
        
        if self.match_token(&Token::While) {
            return Ok(Statement::While(self.parse_while_statement()?));
        }
        
        if self.match_token(&Token::Do) {
            return Ok(Statement::DoWhile(self.parse_do_while_statement()?));
        }
        
        if self.match_token(&Token::For) {
            return Ok(Statement::For(self.parse_for_statement()?));
        }
        
        if self.match_token(&Token::Send) {
            return Ok(Statement::Return(self.parse_return_statement()?));
        }

        if self.match_token(&Token::Break) {
            return Ok(Statement::Break(self.parse_break_statement()?));
        }

        if self.match_token(&Token::Continue) {
            return Ok(Statement::Continue(self.parse_continue_statement()?));
        }
        
        if self.match_token(&Token::LeftBrace) {
            self.current -= 1; // Put back the brace
            return Ok(Statement::Block(self.parse_block()?));
        }

        let checkpoint = self.current;
    
        if let Some(TokenWithSpan { token: Token::Identifier(_), .. }) = self.peek() {
            self.advance(); // consume identifier
            
            // Check for direct assignment: x = ...
            if self.check(&Token::Assign) {
                self.current = checkpoint;
                return self.parse_assignment_statement();
            }
            
            // Check for array assignment: arr[...] = ...
            if self.check(&Token::LeftBracket) {
                // Skip array indexing to check for assignment
                let mut depth = 0;
                while !self.is_at_end() {
                    if self.check(&Token::LeftBracket) {
                        depth += 1;
                        self.advance();
                    } else if self.check(&Token::RightBracket) {
                        depth -= 1;
                        self.advance();
                        if depth == 0 {
                            break;
                        }
                    } else {
                        self.advance();
                    }
                }
                
                // Now check for =
                if self.check(&Token::Assign) {
                    self.current = checkpoint;
                    return self.parse_assignment_statement();
                }
            }
            
            // Not an assignment, reset
            self.current = checkpoint;
        }
        
        // Otherwise, try to parse as expression statement
        Ok(Statement::Expression(self.parse_expression_statement()?))
    }

    /// Parse const statement
    fn parse_const_statement(&mut self) -> Result<ConstStmt, ParserError> {
        let start = self.previous_span().start;
        
        let name = self.expect_identifier()?;
        self.expect_token(Token::Colon)?;
        let typ = self.parse_type()?;
        
        // Constants MUST be initialized
        self.expect_token(Token::Assign)?;
        let value = self.parse_expression()?;
        
        self.expect_token(Token::Semicolon)?;
        let end = self.previous_span().end;
        
        Ok(ConstStmt {
            name,
            typ,
            value,
            span: Span::new(start, end),
        })
    }
    
    /// Parse let statement
    fn parse_let_statement(&mut self) -> Result<LetStmt, ParserError> {
        let start = self.previous_span().start;
        
        let name = self.expect_identifier()?;
        self.expect_token(Token::Colon)?;
        let typ = self.parse_type()?;
        
        let value = if self.match_token(&Token::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.expect_token(Token::Semicolon)?;
        let end = self.previous_span().end;
        
        Ok(LetStmt {
            name,
            typ,
            value,
            span: Span::new(start, end),
        })
    }
    
    /// Parse display statement
    fn parse_display_statement(&mut self) -> Result<DisplayStmt, ParserError> {
        let start = self.previous_span().start;
        let mut expressions = Vec::new();
        
        // Parse first expression
        expressions.push(self.parse_expression()?);
        
        // Parse remaining expressions
        while self.match_token(&Token::Comma) {
            expressions.push(self.parse_expression()?);
        }
        
        self.expect_token(Token::Semicolon)?;
        let end = self.previous_span().end;
        
        Ok(DisplayStmt {
            expressions,
            span: Span::new(start, end),
        })
    }
    
    /// Parse if statement
    fn parse_if_statement(&mut self) -> Result<IfStmt, ParserError> {
        let start = self.previous_span().start;
        
        let condition = self.parse_expression()?;
        let then_block = self.parse_block()?;
        
        let else_block = if self.match_token(&Token::Else) {
            if self.check(&Token::If) {
                // else if - parse as nested if
                self.advance(); // consume 'if'
                let nested_if = self.parse_if_statement()?;
                let span= nested_if.span.clone();
                Some(Block {
                    statements: vec![Statement::If(nested_if)],
                    span,
                })
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };
        
        let end = else_block.as_ref()
            .map(|b| b.span.end)
            .unwrap_or(then_block.span.end);
        
        Ok(IfStmt {
            condition,
            then_block,
            else_block,
            span: Span::new(start, end),
        })
    }
    
    /// Parse while statement
    fn parse_while_statement(&mut self) -> Result<WhileStmt, ParserError> {
        let start = self.previous_span().start;
        
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        let end = body.span.end;
        
        Ok(WhileStmt {
            condition,
            body,
            span: Span::new(start, end),
        })
    }
    
    /// Parse do-while statement
    fn parse_do_while_statement(&mut self) -> Result<DoWhileStmt, ParserError> {
        let start = self.previous_span().start;
        
        let body = self.parse_block()?;
        self.expect_token(Token::While)?;
        let condition = self.parse_expression()?;
        self.expect_token(Token::Semicolon)?;
        let end = self.previous_span().end;
        
        Ok(DoWhileStmt {
            body,
            condition,
            span: Span::new(start, end),
        })
    }
    
    /// Parse for statement
    fn parse_for_statement(&mut self) -> Result<ForStmt, ParserError> {
        let start = self.previous_span().start;
        
        // Parse init
        let init = if self.match_token(&Token::Semicolon) {
            None
        } else if self.match_token(&Token::Let) {
            Some(Box::new(Statement::Let(self.parse_let_statement()?)))
        } else {
            // Could be assignment (i = 0) or expression
            // Try parsing as assignment first
            let checkpoint = self.current;
            
            if let Some(TokenWithSpan { token: Token::Identifier(_), .. }) = self.peek() {
                self.advance();
                if self.check(&Token::Assign) {
                    // It's an assignment
                    self.current = checkpoint;
                    let name = self.expect_identifier()?;
                    self.expect_token(Token::Assign)?;
                    let value = self.parse_expression()?;
                    self.expect_token(Token::Semicolon)?;
                    
                    Some(Box::new(Statement::Expression(ExprStmt {
                        expression: Expression::Assign(AssignExpr {
                            target: name,
                            value: Box::new(value),
                            span: Span::default(),
                        }),
                        span: Span::default(),
                    })))
                } else {
                    // Not assignment, parse as expression
                    self.current = checkpoint;
                    let expr = self.parse_expression()?;
                    self.expect_token(Token::Semicolon)?;
                    Some(Box::new(Statement::Expression(ExprStmt {
                        expression: expr,
                        span: Span::default(),
                    })))
                }
            } else {
                let expr = self.parse_expression()?;
                self.expect_token(Token::Semicolon)?;
                Some(Box::new(Statement::Expression(ExprStmt {
                    expression: expr,
                    span: Span::default(),
                })))
            }
        };
        
        // Parse condition
        let condition = if self.check(&Token::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.expect_token(Token::Semicolon)?;
        
        // Parse update
        let update = if self.check(&Token::LeftBrace) {
            None
        } else {
            // Check if it's an assignment
            let checkpoint = self.current;
            
            if let Some(TokenWithSpan { token: Token::Identifier(name), .. }) = self.peek() {
                let name = name.clone();
                self.advance();
                
                if self.check(&Token::Assign) {
                    // It's an assignment in update
                    self.advance(); // consume =
                    let value = self.parse_expression()?;
                    
                    Some(Expression::Assign(AssignExpr {
                        target: name,
                        value: Box::new(value),
                        span: Span::default(),
                    }))
                } else {
                    // Not assignment, parse as normal expression
                    self.current = checkpoint;
                    Some(self.parse_expression()?)
                }
            } else {
                Some(self.parse_expression()?)
            }
        };
        
        // Parse body
        let body = self.parse_block()?;
        let end = body.span.end;
        
        Ok(ForStmt {
            init,
            condition,
            update,
            body,
            span: Span::new(start, end),
        })
    }
    
    /// Parse return statement
    fn parse_return_statement(&mut self) -> Result<ReturnStmt, ParserError> {
        let start = self.previous_span().start;
        
        let value = if self.check(&Token::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        
        self.expect_token(Token::Semicolon)?;
        let end = self.previous_span().end;
        
        Ok(ReturnStmt {
            value,
            span: Span::new(start, end),
        })
    }

    /// Parse break statement
    fn parse_break_statement(&mut self) -> Result<BreakStmt, ParserError> {
        let start = self.previous_span().start;
        self.expect_token(Token::Semicolon)?;
        let end = self.previous_span().end;
        
        Ok(BreakStmt {
            span: Span::new(start, end),
        })
    }

    /// Parse continue statement
    fn parse_continue_statement(&mut self) -> Result<ContinueStmt, ParserError> {
        let start = self.previous_span().start;
        self.expect_token(Token::Semicolon)?;
        let end = self.previous_span().end;
        
        Ok(ContinueStmt {
            span: Span::new(start, end),
        })
    }
    
    /// Parse expression statement
    fn parse_expression_statement(&mut self) -> Result<ExprStmt, ParserError> {
        let start = self.current_span().start;
        let expression = self.parse_expression()?;
        self.expect_token(Token::Semicolon)?;
        let end = self.previous_span().end;
        
        Ok(ExprStmt {
            expression,
            span: Span::new(start, end),
        })
    }

    /// Parse assignment statement (x = value; or arr[i] = value;)
    fn parse_assignment_statement(&mut self) -> Result<Statement, ParserError> {
        let start = self.current_span().start;
        
        // Get the identifier
        let name = self.expect_identifier()?;
        
        // Check if it's array assignment
        if self.check(&Token::LeftBracket) {
            // Array assignment: arr[index] = value;
            self.advance(); // consume [
            let index = self.parse_expression()?;
            self.expect_token(Token::RightBracket)?;
            self.expect_token(Token::Assign)?;
            let value = self.parse_expression()?;
            self.expect_token(Token::Semicolon)?;
            let end = self.previous_span().end;
            
            // Create proper indexed assignment
            // We'll encode this as a special assignment expression
            let array_expr = Expression::Index(IndexExpr {
                array: Box::new(Expression::Identifier(IdentifierExpr {
                    name: name.clone(),
                    span: Span::new(start, start + name.len()),
                })),
                index: Box::new(index),
                span: Span::new(start, end),
            });
            
            // Create a special marker assignment
            Ok(Statement::Expression(ExprStmt {
                expression: Expression::Assign(AssignExpr {
                    target: format!("__ARRAY_INDEX__:{}", name),
                    value: Box::new(Expression::Binary(BinaryExpr {
                        left: Box::new(array_expr),
                        op: BinaryOp::Equal, // Reusing Equal as assignment marker
                        right: Box::new(value),
                        span: Span::new(start, end),
                        optimization_hint: None,
                    })),
                    span: Span::new(start, end),
                }),
                span: Span::new(start, end),
            }))
        } else {
            // Simple assignment: x = value;
            self.expect_token(Token::Assign)?;
            let value = self.parse_expression()?;
            self.expect_token(Token::Semicolon)?;
            let end = self.previous_span().end;
            
            Ok(Statement::Expression(ExprStmt {
                expression: Expression::Assign(AssignExpr {
                    target: name,
                    value: Box::new(value),
                    span: Span::new(start, end),
                }),
                span: Span::new(start, end),
            }))
        }
    }
    
    /// Parse an expression (with precedence)
    fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        self.parse_logical_or()
    }
    
    /// Parse logical OR expression
    fn parse_logical_or(&mut self) -> Result<Expression, ParserError> {
        let mut left = self.parse_logical_and()?;
        
        while self.match_token(&Token::Or) {
            let start_span = self.get_expression_span(&left).start;
            let op = BinaryOp::Or;
            let right = self.parse_logical_and()?;
            let end_span = self.get_expression_span(&right).end;
            
            left = Expression::Binary(BinaryExpr {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span: Span::new(start_span, end_span),
                optimization_hint: None,
            });
        }
        
        Ok(left)
    }
    
    /// Parse logical AND expression
    fn parse_logical_and(&mut self) -> Result<Expression, ParserError> {
        let mut left = self.parse_equality()?;
        
        while self.match_token(&Token::And) {
            let start_span = self.get_expression_span(&left).start;
            let op = BinaryOp::And;
            let right = self.parse_equality()?;
            let end_span = self.get_expression_span(&right).end;
            
            left = Expression::Binary(BinaryExpr {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span: Span::new(start_span, end_span),
                optimization_hint: None,
            });
        }
        
        Ok(left)
    }
    
    /// Parse equality expression
    fn parse_equality(&mut self) -> Result<Expression, ParserError> {
        let mut left = self.parse_comparison()?;
        
        while let Some(op) = self.match_tokens(&[Token::Equal, Token::NotEqual]) {
            let start_span = self.get_expression_span(&left).start;
            let op = match op {
                Token::Equal => BinaryOp::Equal,
                Token::NotEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            let end_span = self.get_expression_span(&right).end;
            
            left = Expression::Binary(BinaryExpr {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span: Span::new(start_span, end_span),
                optimization_hint: None,
            });
        }
        
        Ok(left)
    }
    
    /// Parse comparison expression
    fn parse_comparison(&mut self) -> Result<Expression, ParserError> {
        let mut left = self.parse_addition()?;
        
        while let Some(op) = self.match_tokens(&[
            Token::LessThan,
            Token::GreaterThan,
            Token::LessEqual,
            Token::GreaterEqual,
        ]) {
            let op = match op {
                Token::LessThan => BinaryOp::Less,
                Token::GreaterThan => BinaryOp::Greater,
                Token::LessEqual => BinaryOp::LessEqual,
                Token::GreaterEqual => BinaryOp::GreaterEqual,
                _ => unreachable!(),
            };
            let right = self.parse_addition()?;
            let start_span = self.get_expression_span(&left).start;
            let end_span = self.get_expression_span(&right).end;
            
            left = Expression::Binary(BinaryExpr {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span: Span::new(start_span,end_span),
                optimization_hint: None,
            });
        }
        
        Ok(left)
    }
    
    /// Parse addition/subtraction expression
    fn parse_addition(&mut self) -> Result<Expression, ParserError> {
        let mut left = self.parse_multiplication()?;
        
        while let Some(op) = self.match_tokens(&[Token::Plus, Token::Minus]) {
            let op = match op {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };
            let right = self.parse_multiplication()?;
            let start_span = self.get_expression_span(&left).start;
            let end_span = self.get_expression_span(&right).end;
            
            left = Expression::Binary(BinaryExpr {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span: Span::new(start_span, end_span),
                optimization_hint: None,
            });
        }
        
        Ok(left)
    }
    
    /// Parse multiplication/division expression
    fn parse_multiplication(&mut self) -> Result<Expression, ParserError> {
        let mut left = self.parse_unary()?;
        
        while let Some(op) = self.match_tokens(&[Token::Star, Token::Slash, Token::Percent]) {
            let op = match op {
                Token::Star => BinaryOp::Multiply,
                Token::Slash => BinaryOp::Divide,
                Token::Percent => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            let start_span = self.get_expression_span(&left).start;
            let end_span = self.get_expression_span(&right).end;
            
            left = Expression::Binary(BinaryExpr {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span: Span::new(start_span, end_span),
                optimization_hint: None,
            });
        }
        
        Ok(left)
    }
    
    /// Parse unary expression
    fn parse_unary(&mut self) -> Result<Expression, ParserError> {
        if self.match_token(&Token::Not) {
            let op = UnaryOp::Not;
            let operand = self.parse_unary()?;
            let span = Span::default();
            
            return Ok(Expression::Unary(UnaryExpr {
                op,
                operand: Box::new(operand),
                span,
            }));
        }
        
        if self.match_token(&Token::Minus) {
            let op = UnaryOp::Negate;
            let operand = self.parse_unary()?;
            let span = Span::default();
            
            return Ok(Expression::Unary(UnaryExpr {
                op,
                operand: Box::new(operand),
                span,
            }));
        }
        
        self.parse_postfix()
    }
    
    /// Parse postfix expression (calls, indexing)
    fn parse_postfix(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.parse_primary()?;
        
        loop {
            if self.match_token(&Token::LeftParen) {
                // Function call
                let start_span = if let Expression::Identifier(ref id_expr) = expr {
                    id_expr.span.start
                } else {
                    self.previous_span().start
                };
                
                let args = self.parse_arguments()?;
                self.expect_token(Token::RightParen)?;
                let end_span = self.previous_span().end;
                
                if let Expression::Identifier(id_expr) = expr {
                    expr = Expression::Call(CallExpr {
                        function: id_expr.name,
                        args,
                        span: Span::new(start_span, end_span),
                    });
                } else {
                    return Err(ParserError::InvalidExpression {
                        span: miette::SourceSpan::from(self.current_span()),
                    });
                }
            } else if self.match_token(&Token::LeftBracket) {
                // Array indexing
                let start_span = match &expr {
                    Expression::Identifier(id_expr) => id_expr.span.start,
                    Expression::Index(idx_expr) => idx_expr.span.start,
                    _ => self.previous_span().start,
                };
                
                let index = self.parse_expression()?;
                self.expect_token(Token::RightBracket)?;
                let end_span = self.previous_span().end;
                
                expr = Expression::Index(IndexExpr {
                    array: Box::new(expr),
                    index: Box::new(index),
                    span: Span::new(start_span, end_span),
                });
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    /// Parse function arguments
    fn parse_arguments(&mut self) -> Result<Vec<Expression>, ParserError> {
        let mut args = Vec::new();
        
        if !self.check(&Token::RightParen) {
            loop {
                args.push(self.parse_expression()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        Ok(args)
    }
    
    /// Parse primary expression
    fn parse_primary(&mut self) -> Result<Expression, ParserError> {
        // Literals
        if let Some(token) = self.advance() {
            match &token.token {
                Token::Integer(n) => {
                    let span = token.span.clone();
                    return Ok(Expression::Literal(LiteralExpr {
                        value: Literal::Integer(*n),
                        span: Span::new(span.start, span.end),
                    }));
                }
                Token::Float(f) => {
                    let span = token.span.clone();
                    return Ok(Expression::Literal(LiteralExpr {
                        value: Literal::Float(*f),
                        span: Span::new(span.start, span.end),
                    }));
                }
                Token::String(s) => {
                    let span = token.span.clone();
                    let s_clone = s.clone();
                    let literal = self.parse_string_literal(&s_clone)?; 
                    return Ok(Expression::Literal(LiteralExpr {
                        value: literal,
                        span: Span::new(span.start, span.end),
                    }));
                }
                Token::True => {
                    let span = token.span.clone();
                    return Ok(Expression::Literal(LiteralExpr {
                        value: Literal::Boolean(true),
                        span: Span::new(span.start, span.end),
                    }));
                }
                Token::False => {
                    let span = token.span.clone();
                    return Ok(Expression::Literal(LiteralExpr {
                        value: Literal::Boolean(false),
                        span: Span::new(span.start, span.end),
                    }));
                }
                Token::Identifier(name) => {
                    let span = token.span.clone();
                    return Ok(Expression::Identifier(IdentifierExpr {
                        name: name.clone(),
                        span: Span::new(span.start, span.end),
                    }));
                }
                Token::LeftParen => {
                    // Grouped expression
                    let expr = self.parse_expression()?;
                    self.expect_token(Token::RightParen)?;
                    return Ok(expr);
                }
                Token::LeftBracket => {
                    let start = token.span.start;
                    // Array literal
                    let mut elements = Vec::new();
                    
                    if !self.check(&Token::RightBracket) {
                        loop {
                            elements.push(self.parse_expression()?);
                            if !self.match_token(&Token::Comma) {
                                break;
                            }
                        }
                    }
                    
                    let end = self.current_span().end;
                    self.expect_token(Token::RightBracket)?;
                    return Ok(Expression::Literal(LiteralExpr {
                        value: Literal::Array(elements),
                        span: Span::new(start, end),
                    }));
                }
                _ => {}
            }
            
            // Put the token back
            self.current -= 1;
        }
        
        Err(ParserError::InvalidExpression {
            span: miette::SourceSpan::from(self.current_span()),
        })
    }
    
    // ==================== HELPER METHODS ====================
    
    /// Check if we're at the end of tokens
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
    
    /// Get current token without consuming
    fn peek(&self) -> Option<&TokenWithSpan> {
        self.tokens.get(self.current)
    }
    
    /// Get current span
    fn current_span(&self) -> std::ops::Range<usize> {
        self.peek()
            .map(|t| t.span.clone())
            .unwrap_or(self.previous_span())
    }
    
    /// Get previous span
    fn previous_span(&self) -> std::ops::Range<usize> {
        if self.current > 0 {
            self.tokens.get(self.current - 1)
                .map(|t| t.span.clone())
                .unwrap_or(0..0)
        } else {
            0..0
        }
    }
    
    /// Advance to next token
    fn advance(&mut self) -> Option<&TokenWithSpan> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens.get(self.current - 1)
    }
    
    /// Check if current token matches
    fn check(&self, token: &Token) -> bool {
        self.peek().map_or(false, |t| std::mem::discriminant(&t.token) == std::mem::discriminant(token))
    }
    
    /// Consume token if it matches
    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    /// Match any of the given tokens
    fn match_tokens(&mut self, tokens: &[Token]) -> Option<Token> {
        for token in tokens {
            if self.check(token) {
                let matched = self.advance()?.token.clone();
                return Some(matched);
            }
        }
        None
    }
    
    /// Expect a specific token
    fn expect_token(&mut self, expected: Token) -> Result<(), ParserError> {
        if self.check(&expected) {
            self.advance();
            Ok(())
        } else {
            let found = self.peek()
                .map(|t| format!("{:?}", t.token))
                .unwrap_or_else(|| "end of input".to_string());
            
            Err(ParserError::UnexpectedToken {
                expected: format!("{:?}", expected),
                found,
                span: miette::SourceSpan::from(self.current_span()),
            })
        }
    }
    
    /// Expect an identifier
    fn expect_identifier(&mut self) -> Result<String, ParserError> {
        match self.advance() {
            Some(TokenWithSpan { token: Token::Identifier(name), .. }) => Ok(name.clone()),
            Some(token) => Err(ParserError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: format!("{:?}", token.token),
                span: miette::SourceSpan::from(token.span.clone()),
            }),
            None => Err(ParserError::UnexpectedEof {
                expected: "identifier".to_string(),
            }),
        }
    }
    
    /// Expect an integer
    fn expect_integer(&mut self) -> Result<i32, ParserError> {
        match self.advance() {
            Some(TokenWithSpan { token: Token::Integer(n), .. }) => Ok(*n),
            Some(token) => Err(ParserError::UnexpectedToken {
                expected: "integer".to_string(),
                found: format!("{:?}", token.token),
                span: miette::SourceSpan::from(token.span.clone()),
            }),
            None => Err(ParserError::UnexpectedEof {
                expected: "integer".to_string(),
            }),
        }
    }

    /// Get span from an expression
    fn get_expression_span(&self, expr: &Expression) -> Span {
        match expr {
            Expression::Literal(lit_expr) => lit_expr.span.clone(),
            Expression::Identifier(id_expr) => id_expr.span.clone(),
            Expression::Binary(binary) => binary.span.clone(),
            Expression::Unary(unary) => unary.span.clone(),
            Expression::Call(call) => call.span.clone(),
            Expression::Index(index) => index.span.clone(),
            Expression::Assign(assign) => assign.span.clone(),
        }
    }
}