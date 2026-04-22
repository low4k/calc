use crate::error::ParseError;

use super::ast::{GraphExpr, ScalarExpr};

const SUPPORTED_FUNCTIONS: &[&str] = &[
    "sin", "cos", "tan", "asin", "acos", "atan", "exp", "ln", "log", "sqrt", "abs",
];

pub fn validate_graph_expression(expr: &GraphExpr) -> Result<(), ParseError> {
    match expr {
        GraphExpr::Scalar(expr) => validate_scalar(expr),
        GraphExpr::Relation { left, right, .. } => {
            validate_scalar(left)?;
            validate_scalar(right)
        }
    }
}

fn validate_scalar(expr: &ScalarExpr) -> Result<(), ParseError> {
    match expr {
        ScalarExpr::Literal(_)
        | ScalarExpr::Variable(_)
        | ScalarExpr::ConstantPi
        | ScalarExpr::ConstantE => Ok(()),
        ScalarExpr::Unary { expr, .. } => validate_scalar(expr),
        ScalarExpr::Binary { left, right, .. } => {
            validate_scalar(left)?;
            validate_scalar(right)
        }
        ScalarExpr::FunctionCall { name, argument } => {
            if !SUPPORTED_FUNCTIONS.contains(&name.as_str()) {
                return Err(ParseError::new(format!("unsupported function '{name}'"), 0));
            }
            validate_scalar(argument)
        }
        ScalarExpr::BoundCall {
            variable,
            lower,
            upper,
            body,
            ..
        } => {
            if variable == "pi" || variable == "e" {
                return Err(ParseError::new(
                    "bound variables cannot shadow mathematical constants",
                    0,
                ));
            }
            validate_scalar(lower)?;
            validate_scalar(upper)?;
            validate_scalar(body)
        }
    }
}