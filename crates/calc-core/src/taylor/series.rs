use crate::error::EngineError;
use crate::eval::dual::evaluate_taylor;
use crate::eval::{context::EvaluationContext, scalar::evaluate};
use crate::expr::ast::Expr;
use crate::graph::{
    ast::{GraphExpr, ScalarExpr},
    eval::{evaluate_scalar, GraphEvaluationContext},
    explicit_panel_expression,
};
use crate::types::Interval;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TaylorSeriesRequest {
    pub center: f64,
    pub degree: usize,
    pub interval: Interval,
    pub sample_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TaylorSeriesResult {
    pub center: f64,
    pub coefficients: Vec<f64>,
    pub sample_x: Vec<f64>,
    pub function_values: Vec<f64>,
    pub polynomial_values: Vec<f64>,
    pub absolute_error: Vec<f64>,
}

pub fn build_taylor_series(
    expr: &Expr,
    request: TaylorSeriesRequest,
) -> Result<TaylorSeriesResult, EngineError> {
    validate_request(&request)?;
    let series = evaluate_taylor(expr, request.center, request.degree)?;
    let coefficients = series.coeffs().to_vec();
    build_result(request, coefficients, |x| {
        evaluate(expr, EvaluationContext::new(x)).map_err(EngineError::from)
    })
}

pub fn build_taylor_series_graph(
    expr: &GraphExpr,
    request: TaylorSeriesRequest,
) -> Result<TaylorSeriesResult, EngineError> {
    validate_request(&request)?;
    let body = explicit_panel_expression(expr).ok_or_else(|| {
        EngineError::InvalidInput(
            "Taylor series requires a scalar expression or explicit y = f(x) graph expression"
                .to_string(),
        )
    })?;

    let coefficients = estimate_graph_taylor_coefficients(body, request.center, request.degree, request.interval)?;
    build_result(request, coefficients, |x| {
        evaluate_scalar(body, &GraphEvaluationContext::new(x, 0.0)).map_err(EngineError::from)
    })
}

fn validate_request(request: &TaylorSeriesRequest) -> Result<(), EngineError> {
    if !request.center.is_finite() {
        return Err(EngineError::InvalidInput(
            "taylor center must be finite".to_string(),
        ));
    }

    if request.sample_count < 2 {
        return Err(EngineError::InvalidInput(
            "taylor sample_count must be at least 2".to_string(),
        ));
    }

    if request.degree > 12 {
        return Err(EngineError::InvalidInput(
            "taylor degree above 12 is not yet supported".to_string(),
        ));
    }

    Ok(())
}

fn build_result<F>(
    request: TaylorSeriesRequest,
    coefficients: Vec<f64>,
    mut evaluate_fn: F,
) -> Result<TaylorSeriesResult, EngineError>
where
    F: FnMut(f64) -> Result<f64, EngineError>,
{
    let mut sample_x = Vec::with_capacity(request.sample_count);
    let mut function_values = Vec::with_capacity(request.sample_count);
    let mut polynomial_values = Vec::with_capacity(request.sample_count);
    let mut absolute_error = Vec::with_capacity(request.sample_count);

    let step = request.interval.width() / (request.sample_count - 1) as f64;
    for index in 0..request.sample_count {
        let x = request.interval.start + step * index as f64;
        let function_value = evaluate_fn(x)?;
        let polynomial_value = evaluate_polynomial(&coefficients, request.center, x);

        sample_x.push(x);
        function_values.push(function_value);
        polynomial_values.push(polynomial_value);
        absolute_error.push((function_value - polynomial_value).abs());
    }

    Ok(TaylorSeriesResult {
        center: request.center,
        coefficients,
        sample_x,
        function_values,
        polynomial_values,
        absolute_error,
    })
}

fn estimate_graph_taylor_coefficients(
    expr: &ScalarExpr,
    center: f64,
    degree: usize,
    interval: Interval,
) -> Result<Vec<f64>, EngineError> {
    let stencil_radius = degree.max(1);
    let node_count = stencil_radius * 2 + 1;
    let base_scale = (interval.width() / (degree.max(2) as f64 * 8.0)).abs();
    let step = base_scale.max(1e-3).min(0.25);
    let offsets: Vec<f64> = (0..node_count)
        .map(|index| (index as isize - stencil_radius as isize) as f64 * step)
        .collect();

    let values: Vec<f64> = offsets
        .iter()
        .map(|offset| {
            evaluate_scalar(expr, &GraphEvaluationContext::new(center + offset, 0.0))
                .map_err(EngineError::from)
        })
        .collect::<Result<_, _>>()?;

    let mut coefficients = Vec::with_capacity(degree + 1);
    for derivative_order in 0..=degree {
        let weights = finite_difference_weights(&offsets, derivative_order)?;
        let derivative = weights
            .iter()
            .zip(values.iter())
            .map(|(weight, value)| weight * value)
            .sum::<f64>();
        coefficients.push(derivative / factorial(derivative_order) as f64);
    }

    Ok(coefficients)
}

fn finite_difference_weights(offsets: &[f64], derivative_order: usize) -> Result<Vec<f64>, EngineError> {
    let n = offsets.len();
    if derivative_order >= n {
        return Err(EngineError::InvalidInput(
            "finite-difference stencil is too small for the requested Taylor degree".to_string(),
        ));
    }

    let mut matrix = vec![vec![0.0; n + 1]; n];
    for (row, matrix_row) in matrix.iter_mut().enumerate() {
        for (col, offset) in offsets.iter().enumerate() {
            matrix_row[col] = offset.powi(row as i32);
        }
        matrix_row[n] = if row == derivative_order {
            factorial(derivative_order) as f64
        } else {
            0.0
        };
    }

    solve_linear_system(matrix)
}

fn solve_linear_system(mut matrix: Vec<Vec<f64>>) -> Result<Vec<f64>, EngineError> {
    let n = matrix.len();
    for pivot in 0..n {
        let mut best_row = pivot;
        for row in pivot + 1..n {
            if matrix[row][pivot].abs() > matrix[best_row][pivot].abs() {
                best_row = row;
            }
        }

        if matrix[best_row][pivot].abs() < 1e-14 {
            return Err(EngineError::InvalidInput(
                "unable to build a stable Taylor approximation for this expression".to_string(),
            ));
        }

        if best_row != pivot {
            matrix.swap(best_row, pivot);
        }

        let pivot_value = matrix[pivot][pivot];
        for col in pivot..=n {
            matrix[pivot][col] /= pivot_value;
        }

        for row in 0..n {
            if row == pivot {
                continue;
            }
            let factor = matrix[row][pivot];
            if factor.abs() < 1e-14 {
                continue;
            }
            for col in pivot..=n {
                matrix[row][col] -= factor * matrix[pivot][col];
            }
        }
    }

    Ok(matrix.into_iter().map(|row| row[n]).collect())
}

fn evaluate_polynomial(coefficients: &[f64], center: f64, x: f64) -> f64 {
    let mut sum = 0.0;
    let delta = x - center;
    let mut power = 1.0;

    for coefficient in coefficients {
        sum += coefficient * power;
        power *= delta;
    }

    sum
}

fn factorial(value: usize) -> usize {
    (1..=value).product::<usize>().max(1)
}

