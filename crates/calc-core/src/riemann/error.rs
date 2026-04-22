use crate::error::EngineError;
use crate::riemann::types::RiemannSummary;
use crate::{
    error::EvalError,
    expr::ast::Expr,
    eval::{context::EvaluationContext, scalar::evaluate},
    graph::{ast::GraphExpr, eval::{evaluate_scalar, GraphEvaluationContext}, explicit_panel_expression},
    types::Interval,
};

use super::{rectangles::{build_riemann_geometry, build_riemann_geometry_graph}, types::RiemannMethod};

#[derive(Debug, Clone, PartialEq)]
pub struct RiemannErrorSeries {
    pub subdivision_counts: Vec<u32>,
    pub absolute_errors: Vec<f64>,
    pub reference_value: f64,
}

pub fn estimate_reference_value(expr: &Expr, interval: Interval) -> Result<f64, EvalError> {
    adaptive_simpson(expr, interval.start, interval.end, 1e-10, 12)
}

pub fn estimate_reference_value_graph(expr: &GraphExpr, interval: Interval) -> Result<f64, EngineError> {
    let body = explicit_panel_expression(expr).ok_or_else(|| {
        EngineError::InvalidInput(
            "reference value estimation requires a scalar expression or explicit y = f(x) graph expression"
                .to_string(),
        )
    })?;

    adaptive_simpson_graph(body, interval.start, interval.end, 1e-10, 12)
}

pub fn build_error_series(
    expr: &Expr,
    interval: Interval,
    subdivision_counts: &[u32],
    method: RiemannMethod,
) -> Result<RiemannErrorSeries, EngineError> {
    if subdivision_counts.is_empty() {
        return Err(EngineError::InvalidInput(
            "error series requires at least one subdivision count".to_string(),
        ));
    }

    let reference_value = estimate_reference_value(expr, interval)?;
    let mut absolute_errors = Vec::with_capacity(subdivision_counts.len());

    for &count in subdivision_counts {
        if count == 0 {
            return Err(EngineError::InvalidInput(
                "subdivision counts must all be greater than zero".to_string(),
            ));
        }

        let geometry = build_riemann_geometry(expr, interval, count as usize, method)?;
        absolute_errors.push((geometry.summary.approximation - reference_value).abs());
    }

    Ok(RiemannErrorSeries {
        subdivision_counts: subdivision_counts.to_vec(),
        absolute_errors,
        reference_value,
    })
}

pub fn build_error_series_graph(
    expr: &GraphExpr,
    interval: Interval,
    subdivision_counts: &[u32],
    method: RiemannMethod,
) -> Result<RiemannErrorSeries, EngineError> {
    if subdivision_counts.is_empty() {
        return Err(EngineError::InvalidInput(
            "error series requires at least one subdivision count".to_string(),
        ));
    }

    let reference_value = estimate_reference_value_graph(expr, interval)?;
    let mut absolute_errors = Vec::with_capacity(subdivision_counts.len());

    for &count in subdivision_counts {
        if count == 0 {
            return Err(EngineError::InvalidInput(
                "subdivision counts must all be greater than zero".to_string(),
            ));
        }

        let geometry = build_riemann_geometry_graph(expr, interval, count as usize, method)?;
        absolute_errors.push((geometry.summary.approximation - reference_value).abs());
    }

    Ok(RiemannErrorSeries {
        subdivision_counts: subdivision_counts.to_vec(),
        absolute_errors,
        reference_value,
    })
}

pub fn relative_error(summary: &RiemannSummary) -> Option<f64> {
    if summary.reference_value == 0.0 {
        return None;
    }

    Some(summary.absolute_error / summary.reference_value.abs())
}

fn adaptive_simpson(
    expr: &Expr,
    start: f64,
    end: f64,
    tolerance: f64,
    depth_remaining: u32,
) -> Result<f64, EvalError> {
    let fa = sample(expr, start)?;
    let fb = sample(expr, end)?;
    let midpoint = 0.5 * (start + end);
    let fm = sample(expr, midpoint)?;
    let whole = simpson_rule(start, end, fa, fm, fb);
    adaptive_simpson_recursive(expr, start, end, tolerance, whole, fa, fm, fb, depth_remaining)
}

fn adaptive_simpson_graph(
    expr: &crate::graph::ast::ScalarExpr,
    start: f64,
    end: f64,
    tolerance: f64,
    depth_remaining: u32,
) -> Result<f64, EngineError> {
    let fa = sample_graph(expr, start)?;
    let fb = sample_graph(expr, end)?;
    let midpoint = 0.5 * (start + end);
    let fm = sample_graph(expr, midpoint)?;
    let whole = simpson_rule(start, end, fa, fm, fb);
    adaptive_simpson_recursive_graph(expr, start, end, tolerance, whole, fa, fm, fb, depth_remaining)
}

fn adaptive_simpson_recursive(
    expr: &Expr,
    start: f64,
    end: f64,
    tolerance: f64,
    whole: f64,
    fa: f64,
    fm: f64,
    fb: f64,
    depth_remaining: u32,
) -> Result<f64, EvalError> {
    let midpoint = 0.5 * (start + end);
    let left_mid = 0.5 * (start + midpoint);
    let right_mid = 0.5 * (midpoint + end);

    let flm = sample(expr, left_mid)?;
    let frm = sample(expr, right_mid)?;

    let left = simpson_rule(start, midpoint, fa, flm, fm);
    let right = simpson_rule(midpoint, end, fm, frm, fb);
    let delta = left + right - whole;

    if depth_remaining == 0 || delta.abs() <= 15.0 * tolerance {
        return Ok(left + right + delta / 15.0);
    }

    Ok(
        adaptive_simpson_recursive(
            expr,
            start,
            midpoint,
            tolerance * 0.5,
            left,
            fa,
            flm,
            fm,
            depth_remaining - 1,
        )? + adaptive_simpson_recursive(
            expr,
            midpoint,
            end,
            tolerance * 0.5,
            right,
            fm,
            frm,
            fb,
            depth_remaining - 1,
        )?,
    )
}

fn adaptive_simpson_recursive_graph(
    expr: &crate::graph::ast::ScalarExpr,
    start: f64,
    end: f64,
    tolerance: f64,
    whole: f64,
    fa: f64,
    fm: f64,
    fb: f64,
    depth_remaining: u32,
) -> Result<f64, EngineError> {
    let midpoint = 0.5 * (start + end);
    let left_mid = 0.5 * (start + midpoint);
    let right_mid = 0.5 * (midpoint + end);

    let flm = sample_graph(expr, left_mid)?;
    let frm = sample_graph(expr, right_mid)?;

    let left = simpson_rule(start, midpoint, fa, flm, fm);
    let right = simpson_rule(midpoint, end, fm, frm, fb);
    let delta = left + right - whole;

    if depth_remaining == 0 || delta.abs() <= 15.0 * tolerance {
        return Ok(left + right + delta / 15.0);
    }

    Ok(
        adaptive_simpson_recursive_graph(
            expr,
            start,
            midpoint,
            tolerance * 0.5,
            left,
            fa,
            flm,
            fm,
            depth_remaining - 1,
        )? + adaptive_simpson_recursive_graph(
            expr,
            midpoint,
            end,
            tolerance * 0.5,
            right,
            fm,
            frm,
            fb,
            depth_remaining - 1,
        )?,
    )
}

fn simpson_rule(start: f64, end: f64, fa: f64, fm: f64, fb: f64) -> f64 {
    (end - start) * (fa + 4.0 * fm + fb) / 6.0
}

fn sample(expr: &Expr, x: f64) -> Result<f64, EvalError> {
    evaluate(expr, EvaluationContext::new(x))
}

fn sample_graph(expr: &crate::graph::ast::ScalarExpr, x: f64) -> Result<f64, EngineError> {
    evaluate_scalar(expr, &GraphEvaluationContext::new(x, 0.0)).map_err(EngineError::from)
}
