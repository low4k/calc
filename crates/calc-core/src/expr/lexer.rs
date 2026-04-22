use crate::error::ParseError;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
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
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub offset: usize,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
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
            '+' => {
                chars.next();
                tokens.push(Token {
                    kind: TokenKind::Plus,
                    offset,
                });
            }
            '-' => {
                chars.next();
                tokens.push(Token {
                    kind: TokenKind::Minus,
                    offset,
                });
            }
            '*' => {
                chars.next();
                tokens.push(Token {
                    kind: TokenKind::Star,
                    offset,
                });
            }
            '/' => {
                chars.next();
                tokens.push(Token {
                    kind: TokenKind::Slash,
                    offset,
                });
            }
            '^' => {
                chars.next();
                tokens.push(Token {
                    kind: TokenKind::Caret,
                    offset,
                });
            }
            '(' => {
                chars.next();
                tokens.push(Token {
                    kind: TokenKind::LeftParen,
                    offset,
                });
            }
            ')' => {
                chars.next();
                tokens.push(Token {
                    kind: TokenKind::RightParen,
                    offset,
                });
            }
            ',' => {
                chars.next();
                tokens.push(Token {
                    kind: TokenKind::Comma,
                    offset,
                });
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
