use crate::error::ParseError;
use crate::expr::ast::{BinaryOp, UnaryOp};

use super::ast::{BoundOperator, GraphExpr, RelationOp, ScalarExpr};

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    Number(f64),
    Identifier(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LeftParen,
    RightParen,
    Comma,
    Equal,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    BangEqual,
    End,
}

#[derive(Debug, Clone, PartialEq)]
struct Token {
    kind: TokenKind,
    offset: usize,
}

pub struct GraphExpressionParser {
    tokens: Vec<Token>,
    current: usize,
}

impl GraphExpressionParser {
    pub fn new(input: &str) -> Result<Self, ParseError> {
        Ok(Self {
            tokens: tokenize(input)?,
            current: 0,
        })
    }

    pub fn parse(&mut self) -> Result<GraphExpr, ParseError> {
        let left = self.parse_scalar_expression()?;
        let expression = match self.peek() {
            TokenKind::Equal
            | TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::BangEqual => {
                let relation = self.parse_relation_operator();
                let right = self.parse_scalar_expression()?;
                GraphExpr::Relation {
                    op: relation,
                    left: Box::new(left),
                    right: Box::new(right),
                }
            }
            TokenKind::End => GraphExpr::Scalar(left),
            _ => return Err(ParseError::new("unexpected trailing tokens", self.offset())),
        };

        if !matches!(self.peek(), TokenKind::End) {
            return Err(ParseError::new("unexpected trailing tokens", self.offset()));
        }

        Ok(expression)
    }

    fn parse_relation_operator(&mut self) -> RelationOp {
        let relation = match self.peek() {
            TokenKind::Equal => RelationOp::Equal,
            TokenKind::Less => RelationOp::LessThan,
            TokenKind::LessEqual => RelationOp::LessThanOrEqual,
            TokenKind::Greater => RelationOp::GreaterThan,
            TokenKind::GreaterEqual => RelationOp::GreaterThanOrEqual,
            TokenKind::BangEqual => RelationOp::NotEqual,
            _ => unreachable!("relation operator expected"),
        };
        self.advance();
        relation
    }

    fn parse_scalar_expression(&mut self) -> Result<ScalarExpr, ParseError> {
        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Result<ScalarExpr, ParseError> {
        let mut expr = self.parse_multiplicative()?;

        loop {
            let op = match self.peek() {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Subtract,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            expr = ScalarExpr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<ScalarExpr, ParseError> {
        let mut expr = self.parse_power()?;

        loop {
            let op = match self.peek() {
                TokenKind::Star => Some(BinaryOp::Multiply),
                TokenKind::Slash => Some(BinaryOp::Divide),
                _ if self.can_start_implicit_factor() => Some(BinaryOp::Multiply),
                _ => None,
            };

            let Some(op) = op else { break };

            if matches!(self.peek(), TokenKind::Star | TokenKind::Slash) {
                self.advance();
            }

            let right = self.parse_power()?;
            expr = ScalarExpr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_power(&mut self) -> Result<ScalarExpr, ParseError> {
        let left = self.parse_unary()?;
        if matches!(self.peek(), TokenKind::Caret) {
            self.advance();
            let right = self.parse_power()?;
            return Ok(ScalarExpr::Binary {
                op: BinaryOp::Power,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<ScalarExpr, ParseError> {
        match self.peek() {
            TokenKind::Plus => {
                self.advance();
                Ok(ScalarExpr::Unary {
                    op: UnaryOp::Plus,
                    expr: Box::new(self.parse_unary()?),
                })
            }
            TokenKind::Minus => {
                self.advance();
                Ok(ScalarExpr::Unary {
                    op: UnaryOp::Minus,
                    expr: Box::new(self.parse_unary()?),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<ScalarExpr, ParseError> {
        match self.peek().clone() {
            TokenKind::Number(value) => {
                self.advance();
                Ok(ScalarExpr::Literal(value))
            }
            TokenKind::Identifier(name) => self.parse_identifier(name),
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_scalar_expression()?;
                self.expect(TokenKind::RightParen, "expected ')' to close expression")?;
                Ok(expr)
            }
            TokenKind::End => Err(ParseError::new("unexpected end of input", self.offset())),
            _ => Err(ParseError::new("expected expression", self.offset())),
        }
    }

    fn parse_identifier(&mut self, name: String) -> Result<ScalarExpr, ParseError> {
        self.advance();

        if matches!(self.peek(), TokenKind::LeftParen) {
            self.advance();
                return self.parse_call(name);
        }

        match name.as_str() {
            "pi" => Ok(ScalarExpr::ConstantPi),
            "e" => Ok(ScalarExpr::ConstantE),
            _ => Ok(expand_identifier_product(name)),
        }
    }

    fn parse_call(&mut self, name: String) -> Result<ScalarExpr, ParseError> {
        if let Some(bound_operator) = parse_bound_operator(&name) {
            let variable = self.parse_binding_identifier()?;
            self.expect(TokenKind::Comma, "expected ',' after bound variable")?;
            let lower = self.parse_scalar_expression()?;
            self.expect(TokenKind::Comma, "expected ',' after lower bound")?;
            let upper = self.parse_scalar_expression()?;
            self.expect(TokenKind::Comma, "expected ',' after upper bound")?;
            let body = self.parse_scalar_expression()?;
            self.expect(TokenKind::RightParen, "expected ')' after bound operator body")?;
            return Ok(ScalarExpr::BoundCall {
                op: bound_operator,
                variable,
                lower: Box::new(lower),
                upper: Box::new(upper),
                body: Box::new(body),
            });
        }

        let argument = self.parse_scalar_expression()?;
        self.expect(TokenKind::RightParen, "expected ')' after function argument")?;
        Ok(ScalarExpr::FunctionCall {
            name,
            argument: Box::new(argument),
        })
    }

    fn parse_binding_identifier(&mut self) -> Result<String, ParseError> {
        match self.peek().clone() {
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(name)
            }
            _ => Err(ParseError::new(
                "expected a binding identifier as the first argument",
                self.offset(),
            )),
        }
    }

    fn can_start_implicit_factor(&self) -> bool {
        matches!(
            self.peek(),
            TokenKind::Number(_) | TokenKind::Identifier(_) | TokenKind::LeftParen
        )
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

fn parse_bound_operator(name: &str) -> Option<BoundOperator> {
    match name {
        "int" => Some(BoundOperator::Integral),
        "sum" => Some(BoundOperator::Summation),
        "prod" => Some(BoundOperator::Product),
        _ => None,
    }
}

fn expand_identifier_product(name: String) -> ScalarExpr {
    if name.len() <= 1 || !name.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return ScalarExpr::Variable(name);
    }

    let mut parts = name.chars().map(|ch| ScalarExpr::Variable(ch.to_string()));
    let Some(first) = parts.next() else {
        return ScalarExpr::Variable(name);
    };

    parts.fold(first, |left, right| ScalarExpr::Binary {
        op: BinaryOp::Multiply,
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    let mut tokens = Vec::new();
    let mut chars = input.char_indices().peekable();

    while let Some((offset, ch)) = chars.peek().copied() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            '0'..='9' | '.' => {
                let start = offset;
                let mut end = offset;
                let mut saw_dot = ch == '.';
                chars.next();

                while let Some((next_offset, next_ch)) = chars.peek().copied() {
                    match next_ch {
                        '0'..='9' => {
                            end = next_offset;
                            chars.next();
                        }
                        '.' if !saw_dot => {
                            saw_dot = true;
                            end = next_offset;
                            chars.next();
                        }
                        _ => break,
                    }
                }

                let end_index = end + 1;
                let number = input[start..end_index]
                    .parse::<f64>()
                    .map_err(|_| ParseError::new("invalid numeric literal", start))?;
                tokens.push(Token {
                    kind: TokenKind::Number(number),
                    offset: start,
                });
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let start = offset;
                let mut end = offset;
                chars.next();
                while let Some((next_offset, next_ch)) = chars.peek().copied() {
                    if next_ch.is_ascii_alphanumeric() || next_ch == '_' {
                        end = next_offset;
                        chars.next();
                    } else {
                        break;
                    }
                }
                let end_index = end + 1;
                tokens.push(Token {
                    kind: TokenKind::Identifier(input[start..end_index].to_string()),
                    offset: start,
                });
            }
            '+' => push_simple_token(&mut chars, &mut tokens, TokenKind::Plus, offset),
            '-' => push_simple_token(&mut chars, &mut tokens, TokenKind::Minus, offset),
            '*' => push_simple_token(&mut chars, &mut tokens, TokenKind::Star, offset),
            '/' => push_simple_token(&mut chars, &mut tokens, TokenKind::Slash, offset),
            '^' => push_simple_token(&mut chars, &mut tokens, TokenKind::Caret, offset),
            '(' => push_simple_token(&mut chars, &mut tokens, TokenKind::LeftParen, offset),
            ')' => push_simple_token(&mut chars, &mut tokens, TokenKind::RightParen, offset),
            ',' => push_simple_token(&mut chars, &mut tokens, TokenKind::Comma, offset),
            '=' => push_simple_token(&mut chars, &mut tokens, TokenKind::Equal, offset),
            '<' => {
                chars.next();
                let kind = if matches!(chars.peek(), Some((_, '='))) {
                    chars.next();
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                };
                tokens.push(Token { kind, offset });
            }
            '>' => {
                chars.next();
                let kind = if matches!(chars.peek(), Some((_, '='))) {
                    chars.next();
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                };
                tokens.push(Token { kind, offset });
            }
            '!' => {
                chars.next();
                if matches!(chars.peek(), Some((_, '='))) {
                    chars.next();
                    tokens.push(Token {
                        kind: TokenKind::BangEqual,
                        offset,
                    });
                } else {
                    return Err(ParseError::new("unexpected character '!'", offset));
                }
            }
            _ => {
                return Err(ParseError::new(
                    format!("unexpected character '{ch}'"),
                    offset,
                ));
            }
        }
    }

    tokens.push(Token {
        kind: TokenKind::End,
        offset: input.len(),
    });

    Ok(tokens)
}

fn push_simple_token(
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    tokens: &mut Vec<Token>,
    kind: TokenKind,
    offset: usize,
) {
    chars.next();
    tokens.push(Token { kind, offset });
}