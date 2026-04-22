use crate::error::{DomainError, EvalError};
use crate::expr::ast::{BinaryOp, UnaryOp};

use super::ast::{BoundOperator, ScalarExpr};

#[derive(Debug, Clone, PartialEq)]
pub struct GraphEvaluationContext {
    pub x: f64,
    pub y: f64,
    bindings: Vec<(String, f64)>,
}

impl GraphEvaluationContext {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            bindings: Vec::new(),
        }
    }

    fn with_binding(&self, name: String, value: f64) -> Self {
        let mut next = self.clone();
        next.bindings.push((name, value));
        next
    }

    fn lookup(&self, name: &str) -> Option<f64> {
        match name {
            "x" => Some(self.x),
            "y" => Some(self.y),
            _ => self
                .bindings
                .iter()
                .rev()
                .find_map(|(binding, value)| (binding == name).then_some(*value)),
        }
    }
}

pub fn evaluate_scalar(expr: &ScalarExpr, context: &GraphEvaluationContext) -> Result<f64, EvalError> {
    let value = match expr {
        ScalarExpr::Literal(value) => *value,
        ScalarExpr::Variable(name) => context
            .lookup(name)
            .ok_or_else(|| EvalError::new(format!("unknown variable '{name}'")))?,
        ScalarExpr::ConstantPi => core::f64::consts::PI,
        ScalarExpr::ConstantE => core::f64::consts::E,
        ScalarExpr::Unary { op, expr } => {
            let inner = evaluate_scalar(expr, context)?;
            match op {
                UnaryOp::Plus => inner,
                UnaryOp::Minus => -inner,
            }
        }
        ScalarExpr::Binary { op, left, right } => {
            let left = evaluate_scalar(left, context)?;
            let right = evaluate_scalar(right, context)?;
            match op {
                BinaryOp::Add => left + right,
                BinaryOp::Subtract => left - right,
                BinaryOp::Multiply => left * right,
                BinaryOp::Divide => {
                    if right.abs() < 1e-12 {
                        return Err(EvalError::new("division by zero"));
                    }
                    left / right
                }
                BinaryOp::Power => evaluate_power(left, right)?,
            }
        }
        ScalarExpr::FunctionCall { name, argument } => {
            let argument = evaluate_scalar(argument, context)?;
            evaluate_function(name, argument)?
        }
        ScalarExpr::BoundCall {
            op,
            variable,
            lower,
            upper,
            body,
        } => {
            let lower = evaluate_scalar(lower, context)?;
            let upper = evaluate_scalar(upper, context)?;
            evaluate_bound_call(*op, variable, lower, upper, body, context)?
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

fn evaluate_bound_call(
    op: BoundOperator,
    variable: &str,
    lower: f64,
    upper: f64,
    body: &ScalarExpr,
    context: &GraphEvaluationContext,
) -> Result<f64, EvalError> {
    ensure(lower.is_finite() && upper.is_finite(), "bound limits must be finite")?;
    match op {
        BoundOperator::Integral => evaluate_integral(variable, lower, upper, body, context),
        BoundOperator::Summation => evaluate_sum(variable, lower, upper, body, context),
        BoundOperator::Product => evaluate_product(variable, lower, upper, body, context),
    }
}

fn evaluate_integral(
    variable: &str,
    lower: f64,
    upper: f64,
    body: &ScalarExpr,
    context: &GraphEvaluationContext,
) -> Result<f64, EvalError> {
    if (upper - lower).abs() < 1e-12 {
        return Ok(0.0);
    }

    let (start, end, sign) = if upper >= lower {
        (lower, upper, 1.0)
    } else {
        (upper, lower, -1.0)
    };

    let subdivisions = 256usize;
    let width = (end - start) / subdivisions as f64;
    let mut sum = 0.0;

    for index in 0..=subdivisions {
        let point = start + width * index as f64;
        let weight = if index == 0 || index == subdivisions { 0.5 } else { 1.0 };
        let scoped = context.with_binding(variable.to_string(), point);
        sum += weight * evaluate_scalar(body, &scoped)?;
    }

    Ok(sign * sum * width)
}

fn evaluate_sum(
    variable: &str,
    lower: f64,
    upper: f64,
    body: &ScalarExpr,
    context: &GraphEvaluationContext,
) -> Result<f64, EvalError> {
    let (start, end) = integer_bounds(lower, upper)?;
    let term_count = end - start + 1;
    ensure(term_count <= 100_000, "summation range is too large")?;

    let mut total = 0.0;
    for value in start..=end {
        let scoped = context.with_binding(variable.to_string(), value as f64);
        total += evaluate_scalar(body, &scoped)?;
    }
    Ok(total)
}

fn evaluate_product(
    variable: &str,
    lower: f64,
    upper: f64,
    body: &ScalarExpr,
    context: &GraphEvaluationContext,
) -> Result<f64, EvalError> {
    let (start, end) = integer_bounds(lower, upper)?;
    let term_count = end - start + 1;
    ensure(term_count <= 10_000, "product range is too large")?;

    let mut total = 1.0;
    for value in start..=end {
        let scoped = context.with_binding(variable.to_string(), value as f64);
        total *= evaluate_scalar(body, &scoped)?;
    }
    Ok(total)
}

fn integer_bounds(lower: f64, upper: f64) -> Result<(i64, i64), EvalError> {
    ensure(is_effectively_integer(lower), "bound operators require integer lower limits")?;
    ensure(is_effectively_integer(upper), "bound operators require integer upper limits")?;

    let start = lower.round() as i64;
    let end = upper.round() as i64;
    ensure(end >= start, "bound operator upper limit must be at least the lower limit")?;
    Ok((start, end))
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
