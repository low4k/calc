use crate::error::{DomainError, EvalError};
use crate::expr::ast::{BinaryOp, Expr, UnaryOp};

use super::context::EvaluationContext;

pub fn evaluate(expr: &Expr, context: EvaluationContext) -> Result<f64, EvalError> {
    let value = match expr {
        Expr::Literal(value) => *value,
        Expr::Variable => context.x,
        Expr::ConstantPi => core::f64::consts::PI,
        Expr::ConstantE => core::f64::consts::E,
        Expr::Unary { op, expr } => {
            let inner = evaluate(expr, context)?;
            match op {
                UnaryOp::Plus => inner,
                UnaryOp::Minus => -inner,
            }
        }
        Expr::Binary { op, left, right } => {
            let left = evaluate(left, context)?;
            let right = evaluate(right, context)?;
            match op {
                BinaryOp::Add => left + right,
                BinaryOp::Subtract => left - right,
                BinaryOp::Multiply => left * right,
                BinaryOp::Divide => {
                    if right == 0.0 {
                        return Err(EvalError::new("division by zero"));
                    }
                    left / right
                }
                BinaryOp::Power => evaluate_power(left, right)?,
            }
        }
        Expr::FunctionCall { name, argument } => {
            let value = evaluate(argument, context)?;
            evaluate_function(name, value)?
        }
    };

    if value.is_finite() {
        Ok(value)
    } else {
        Err(EvalError::new("non-finite evaluation result"))
    }
}

fn evaluate_function(name: &str, value: f64) -> Result<f64, EvalError> {
    match name {
        "sin" => Ok(value.sin()),
        "cos" => Ok(value.cos()),
        "tan" => {
            ensure(value.cos().abs() > 1e-12, "tan is undefined at this value")?;
            Ok(value.tan())
        }
        "asin" => {
            ensure((-1.0..=1.0).contains(&value), "asin requires input in [-1, 1]")?;
            Ok(value.asin())
        }
        "acos" => {
            ensure((-1.0..=1.0).contains(&value), "acos requires input in [-1, 1]")?;
            Ok(value.acos())
        }
        "atan" => Ok(value.atan()),
        "exp" => Ok(value.exp()),
        "ln" => {
            ensure(value > 0.0, "ln requires a positive value")?;
            Ok(value.ln())
        }
        "log" => {
            ensure(value > 0.0, "log requires a positive value")?;
            Ok(value.log10())
        }
        "sqrt" => {
            ensure(value >= 0.0, "sqrt requires a non-negative value")?;
            Ok(value.sqrt())
        }
        "abs" => Ok(value.abs()),
        _ => Err(EvalError::new(format!("unsupported function '{name}'"))),
    }
}

fn evaluate_power(base: f64, exponent: f64) -> Result<f64, EvalError> {
    if base == 0.0 && exponent == 0.0 {
        return Err(DomainError::new("0^0 is undefined").into());
    }

    if base < 0.0 && !is_effectively_integer(exponent) {
        return Err(DomainError::new(
            "negative bases require an integer exponent in the real domain",
        )
        .into());
    }

    Ok(base.powf(exponent))
}

fn is_effectively_integer(value: f64) -> bool {
    (value - value.round()).abs() < 1e-10
}

fn ensure(condition: bool, message: &str) -> Result<(), EvalError> {
    if condition {
        Ok(())
    } else {
        Err(DomainError::new(message).into())
    }
}

impl From<DomainError> for EvalError {
    fn from(value: DomainError) -> Self {
        Self::new(value.message)
    }
}
