use crate::{
    error::EngineError,
    sampling::DomainSample,
    types::Interval,
};

use super::{
    ast::{GraphExpr, RelationOp, ScalarExpr},
    eval::{evaluate_scalar, GraphEvaluationContext},
};

#[derive(Debug, Clone, PartialEq)]
pub struct RelationGrid {
    pub relation: RelationOp,
    pub resolution: usize,
    pub x_interval: Interval,
    pub y_interval: Interval,
    pub vertex_values: Vec<f64>,
    pub cell_values: Vec<f64>,
}

pub fn sample_graph_curve(
    expr: &GraphExpr,
    interval: Interval,
    count: usize,
) -> Result<Vec<DomainSample>, EngineError> {
    if count < 2 {
        return Err(EngineError::InvalidInput(
            "sample count must be at least 2".to_string(),
        ));
    }

    let body = explicit_curve_body(expr).ok_or_else(|| {
        EngineError::InvalidInput(
            "graph curve sampling requires either a scalar expression or an explicit y = f(x) relation"
                .to_string(),
        )
    })?;

    let step = interval.width() / (count - 1) as f64;
    let mut samples: Vec<DomainSample> = (0..count)
        .map(|index| {
            let x = interval.start + step * index as f64;
            let context = GraphEvaluationContext::new(x, 0.0);
            let y = evaluate_scalar(body, &context).unwrap_or(f64::NAN);
            DomainSample { x, y }
        })
        .collect();

    for index in 0..samples.len().saturating_sub(1) {
        let x0 = samples[index].x;
        let y0 = samples[index].y;
        let x1 = samples[index + 1].x;
        let y1 = samples[index + 1].y;
        let midpoint_x = (x0 + x1) * 0.5;
        let midpoint_y = evaluate_scalar(body, &GraphEvaluationContext::new(midpoint_x, 0.0))
            .unwrap_or(f64::NAN);

        if looks_discontinuous(y0, midpoint_y, y1) {
            if (midpoint_y - y0).abs() >= (midpoint_y - y1).abs() {
                samples[index].y = f64::NAN;
            } else {
                samples[index + 1].y = f64::NAN;
            }
        }
    }

    Ok(samples)
}

pub fn sample_relation_grid(
    expr: &GraphExpr,
    x_interval: Interval,
    y_interval: Interval,
    resolution: usize,
) -> Result<RelationGrid, EngineError> {
    if resolution < 2 {
        return Err(EngineError::InvalidInput(
            "grid resolution must be at least 2".to_string(),
        ));
    }

    let (relation, left, right) = relation_components(expr).ok_or_else(|| {
        EngineError::InvalidInput(
            "relation grid sampling requires an implicit relation".to_string(),
        )
    })?;

    let dx = x_interval.width() / resolution as f64;
    let dy = y_interval.width() / resolution as f64;
    let mut vertex_values = Vec::with_capacity((resolution + 1) * (resolution + 1));
    let mut cell_values = Vec::with_capacity(resolution * resolution);

    for iy in 0..=resolution {
        let y = y_interval.start + dy * iy as f64;
        for ix in 0..=resolution {
            let x = x_interval.start + dx * ix as f64;
            vertex_values.push(relation_difference(left, right, x, y));
        }
    }

    for iy in 0..resolution {
        let y = y_interval.start + dy * (iy as f64 + 0.5);
        for ix in 0..resolution {
            let x = x_interval.start + dx * (ix as f64 + 0.5);
            cell_values.push(sample_cell_difference(left, right, x, y, dx, dy));
        }
    }

    Ok(RelationGrid {
        relation,
        resolution,
        x_interval,
        y_interval,
        vertex_values,
        cell_values,
    })
}

fn looks_discontinuous(y0: f64, midpoint_y: f64, y1: f64) -> bool {
    if !y0.is_finite() || !midpoint_y.is_finite() || !y1.is_finite() {
        return true;
    }

    let endpoint_span = (y1 - y0).abs();
    let linear_midpoint = (y0 + y1) * 0.5;
    let deviation = (midpoint_y - linear_midpoint).abs();
    let scale = y0.abs().max(y1.abs()).max(midpoint_y.abs()).max(1.0);

    deviation > scale * 1.5 && deviation > endpoint_span * 3.0
}

fn sample_cell_difference(
    left: &ScalarExpr,
    right: &ScalarExpr,
    x: f64,
    y: f64,
    dx: f64,
    dy: f64,
) -> f64 {
    let offsets = [
        (0.0, 0.0),
        (-0.25 * dx, 0.0),
        (0.25 * dx, 0.0),
        (0.0, -0.25 * dy),
        (0.0, 0.25 * dy),
    ];
    let mut total = 0.0;
    let mut count = 0usize;

    for (offset_x, offset_y) in offsets {
        let value = relation_difference(left, right, x + offset_x, y + offset_y);
        if value.is_finite() {
            total += value;
            count += 1;
        }
    }

    if count == 0 {
        f64::NAN
    } else {
        total / count as f64
    }
}

fn relation_difference(left: &ScalarExpr, right: &ScalarExpr, x: f64, y: f64) -> f64 {
    let context = GraphEvaluationContext::new(x, y);
    let left = evaluate_scalar(left, &context).unwrap_or(f64::NAN);
    let right = evaluate_scalar(right, &context).unwrap_or(f64::NAN);
    left - right
}

fn explicit_curve_body(expr: &GraphExpr) -> Option<&ScalarExpr> {
    match expr {
        GraphExpr::Scalar(expr) => Some(expr),
        GraphExpr::Relation {
            op: RelationOp::Equal,
            left,
            right,
        } if matches!(&**left, ScalarExpr::Variable(name) if name == "y") => Some(right),
        GraphExpr::Relation {
            op: RelationOp::Equal,
            left,
            right,
        } if matches!(&**right, ScalarExpr::Variable(name) if name == "y") => Some(left),
        _ => None,
    }
}

fn relation_components(expr: &GraphExpr) -> Option<(RelationOp, &ScalarExpr, &ScalarExpr)> {
    match expr {
        GraphExpr::Relation { op, left, right } => Some((*op, left, right)),
        _ => None,
    }
}