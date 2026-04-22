use crate::error::EngineError;
use crate::eval::{context::EvaluationContext, scalar::evaluate};
use crate::expr::ast::Expr;
use crate::types::Interval;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DomainSample {
    pub x: f64,
    pub y: f64,
}

pub fn evenly_spaced_points(interval: Interval, count: usize) -> Result<Vec<f64>, EngineError> {
    if count < 2 {
        return Err(EngineError::InvalidInput(
            "sample count must be at least 2".to_string(),
        ));
    }

    let step = interval.width() / (count - 1) as f64;
    Ok((0..count)
        .map(|index| interval.start + step * index as f64)
        .collect())
}

pub fn sample_expression_curve(
    expr: &Expr,
    interval: Interval,
    count: usize,
) -> Result<Vec<DomainSample>, EngineError> {
    let xs = evenly_spaced_points(interval, count)?;
    xs.into_iter()
        .map(|x| {
            let y = evaluate(expr, EvaluationContext::new(x))?;
            Ok(DomainSample { x, y })
        })
        .collect()
}
