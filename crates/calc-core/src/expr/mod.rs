pub mod ast;
pub mod lexer;
pub mod parser;
pub mod validate;

use crate::error::ParseError;
use ast::Expr;

pub fn parse_expression(input: &str) -> Result<Expr, ParseError> {
    let mut parser = parser::ExpressionParser::new(input)?;
    let expr = parser.parse()?;
    validate::validate_expression(&expr)?;
    Ok(expr)
}
