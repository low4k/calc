use crate::error::EngineError;
use crate::expr::ast::Expr;
use crate::eval::context::EvaluationContext;
use crate::eval::scalar::evaluate;
use crate::graph::{ast::GraphExpr, eval::{evaluate_scalar, GraphEvaluationContext}, explicit_panel_expression};
use crate::riemann::error::estimate_reference_value;
use crate::riemann::types::{RectanglePrimitive, RiemannGeometry, RiemannMethod, RiemannSummary, TrapezoidPrimitive};
use crate::types::Interval;

pub fn build_riemann_geometry(
    expr: &Expr,
    interval: Interval,
    subdivisions: usize,
    method: RiemannMethod,
) -> Result<RiemannGeometry, EngineError> {
    if subdivisions == 0 {
        return Err(EngineError::InvalidInput(
            "subdivisions must be greater than zero".to_string(),
        ));
    }

    let dx = interval.width() / subdivisions as f64;
    let mut rectangles = Vec::new();
    let mut trapezoids = Vec::new();
    let mut approximation = 0.0;

    for index in 0..subdivisions {
        let x0 = interval.start + index as f64 * dx;
        let x1 = x0 + dx;

        match method {
            RiemannMethod::Left | RiemannMethod::Right | RiemannMethod::Midpoint => {
                let sample_x = match method {
                    RiemannMethod::Left => x0,
                    RiemannMethod::Right => x1,
                    RiemannMethod::Midpoint => x0 + dx * 0.5,
                    RiemannMethod::Trapezoid => unreachable!(),
                };
                let height = evaluate(expr, EvaluationContext::new(sample_x))?;
                let signed_area = height * dx;
                rectangles.push(RectanglePrimitive {
                    x: x0,
                    y: height,
                    width: dx,
                    height,
                    signed_area,
                });
                approximation += signed_area;
            }
            RiemannMethod::Trapezoid => {
                let y0 = evaluate(expr, EvaluationContext::new(x0))?;
                let y1 = evaluate(expr, EvaluationContext::new(x1))?;
                let signed_area = (y0 + y1) * 0.5 * dx;
                trapezoids.push(TrapezoidPrimitive {
                    x0,
                    y0,
                    x1,
                    y1,
                    baseline_y: 0.0,
                    signed_area,
                });
                approximation += signed_area;
            }
        }
    }

    let reference_value = estimate_reference_value(expr, interval)?;
    let absolute_error = (approximation - reference_value).abs();

    Ok(RiemannGeometry {
        rectangles,
        trapezoids,
        summary: RiemannSummary {
            approximation,
            reference_value,
            absolute_error,
        },
        warnings: Vec::new(),
    })
}

pub fn build_riemann_geometry_graph(
    expr: &GraphExpr,
    interval: Interval,
    subdivisions: usize,
    method: RiemannMethod,
) -> Result<RiemannGeometry, EngineError> {
    let body = explicit_panel_expression(expr).ok_or_else(|| {
        EngineError::InvalidInput(
            "Riemann geometry requires a scalar expression or explicit y = f(x) graph expression"
                .to_string(),
        )
    })?;

    if subdivisions == 0 {
        return Err(EngineError::InvalidInput(
            "subdivisions must be greater than zero".to_string(),
        ));
    }

    let dx = interval.width() / subdivisions as f64;
    let mut rectangles = Vec::new();
    let mut trapezoids = Vec::new();
    let mut approximation = 0.0;

    for index in 0..subdivisions {
        let x0 = interval.start + index as f64 * dx;
        let x1 = x0 + dx;

        match method {
            RiemannMethod::Left | RiemannMethod::Right | RiemannMethod::Midpoint => {
                let sample_x = match method {
                    RiemannMethod::Left => x0,
                    RiemannMethod::Right => x1,
                    RiemannMethod::Midpoint => x0 + dx * 0.5,
                    RiemannMethod::Trapezoid => unreachable!(),
                };
                let height = evaluate_scalar(body, &GraphEvaluationContext::new(sample_x, 0.0))?;
                let signed_area = height * dx;
                rectangles.push(RectanglePrimitive {
                    x: x0,
                    y: height,
                    width: dx,
                    height,
                    signed_area,
                });
                approximation += signed_area;
            }
            RiemannMethod::Trapezoid => {
                let y0 = evaluate_scalar(body, &GraphEvaluationContext::new(x0, 0.0))?;
                let y1 = evaluate_scalar(body, &GraphEvaluationContext::new(x1, 0.0))?;
                let signed_area = (y0 + y1) * 0.5 * dx;
                trapezoids.push(TrapezoidPrimitive {
                    x0,
                    y0,
                    x1,
                    y1,
                    baseline_y: 0.0,
                    signed_area,
                });
                approximation += signed_area;
            }
        }
    }

    let reference_value = crate::riemann::error::estimate_reference_value_graph(expr, interval)?;
    let absolute_error = (approximation - reference_value).abs();

    Ok(RiemannGeometry {
        rectangles,
        trapezoids,
        summary: RiemannSummary {
            approximation,
            reference_value,
            absolute_error,
        },
        warnings: Vec::new(),
    })
}
