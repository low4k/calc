use crate::error::ParseError;

use super::ast::Expr;

const SUPPORTED_FUNCTIONS: &[&str] = &[
    "sin", "cos", "tan", "asin", "acos", "atan", "exp", "ln", "log", "sqrt", "abs",
];

pub fn validate_expression(expr: &Expr) -> Result<(), ParseError> {
    match expr {
        Expr::Literal(_) | Expr::Variable | Expr::ConstantPi | Expr::ConstantE => Ok(()),
        Expr::Unary { expr, .. } => validate_expression(expr),
        Expr::Binary { left, right, .. } => {
            validate_expression(left)?;
            validate_expression(right)
        }
        Expr::FunctionCall { name, argument } => {
            if !SUPPORTED_FUNCTIONS.contains(&name.as_str()) {
                return Err(ParseError::new(format!("unsupported function '{name}'"), 0));
            }
            validate_expression(argument)
        }
    }
}
