use crate::error::ParseError;

use super::ast::{BinaryOp, Expr, UnaryOp};
use super::lexer::{tokenize, Token, TokenKind};

pub struct ExpressionParser {
    tokens: Vec<Token>,
    current: usize,
}

impl ExpressionParser {
    pub fn new(input: &str) -> Result<Self, ParseError> {
        let tokens = tokenize(input)?;
        Ok(Self { tokens, current: 0 })
    }

    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        if self.tokens.is_empty() {
            return Err(ParseError::new("empty token stream", 0));
        }

        if self.tokens.len() == 1 {
            return Err(ParseError::new("unable to parse expression", self.tokens[0].offset));
        }

        let expr = self.parse_expression()?;
        match self.peek() {
            TokenKind::End => Ok(expr),
            _ => Err(ParseError::new("unexpected trailing tokens", self.offset())),
        }
    }

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_multiplicative()?;

        loop {
            let op = match self.peek() {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Subtract,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_power()?;

        loop {
            let op = match self.peek() {
                TokenKind::Star => BinaryOp::Multiply,
                TokenKind::Slash => BinaryOp::Divide,
                _ => break,
            };
            self.advance();
            let right = self.parse_power()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_power(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_unary()?;
        if matches!(self.peek(), TokenKind::Caret) {
            self.advance();
            let right = self.parse_power()?;
            return Ok(Expr::Binary {
                op: BinaryOp::Power,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            TokenKind::Plus => {
                self.advance();
                Ok(Expr::Unary {
                    op: UnaryOp::Plus,
                    expr: Box::new(self.parse_unary()?),
                })
            }
            TokenKind::Minus => {
                self.advance();
                Ok(Expr::Unary {
                    op: UnaryOp::Minus,
                    expr: Box::new(self.parse_unary()?),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.peek().clone() {
            TokenKind::Number(value) => {
                self.advance();
                Ok(Expr::Literal(value))
            }
            TokenKind::Identifier(name) => {
                let offset = self.offset();
                self.advance();
                if matches!(self.peek(), TokenKind::LeftParen) {
                    self.advance();
                    let argument = self.parse_expression()?;
                    self.expect(TokenKind::RightParen, "expected ')' after function argument")?;
                    return Ok(Expr::FunctionCall {
                        name,
                        argument: Box::new(argument),
                    });
                }

                match name.as_str() {
                    "x" => Ok(Expr::Variable),
                    "pi" => Ok(Expr::ConstantPi),
                    "e" => Ok(Expr::ConstantE),
                    _ => Err(ParseError::new(format!("unknown symbol '{name}'"), offset)),
                }
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RightParen, "expected ')' to close expression")?;
                Ok(expr)
            }
            TokenKind::End => Err(ParseError::new("unexpected end of input", self.offset())),
            _ => Err(ParseError::new("expected expression", self.offset())),
        }
    }

    fn expect(&mut self, expected: TokenKind, message: &str) -> Result<(), ParseError> {
        if core::mem::discriminant(self.peek()) == core::mem::discriminant(&expected) {
            self.advance();
            return Ok(());
        }
        Err(ParseError::new(message, self.offset()))
    }

    fn peek(&self) -> &TokenKind {
        &self.tokens[self.current].kind
    }

    fn advance(&mut self) {
        if self.current < self.tokens.len() - 1 {
            self.current += 1;
        }
    }

    fn offset(&self) -> usize {
        self.tokens
            .get(self.current)
            .map(|token| token.offset)
            .unwrap_or_default()
    }
}
