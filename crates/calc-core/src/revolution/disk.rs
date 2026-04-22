use crate::error::EngineError;
use crate::expr::ast::Expr;
use crate::graph::{ast::{GraphExpr, ScalarExpr}, explicit_panel_expression};
use crate::riemann::{build_riemann_geometry, RiemannMethod};
use crate::types::Interval;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiskVolumeResult {
    pub estimated_volume: f64,
}

pub fn estimate_disk_volume(
    expr: &Expr,
    interval: Interval,
    subdivisions: usize,
) -> Result<DiskVolumeResult, EngineError> {
    let squared_expr = Expr::Binary {
        op: crate::expr::ast::BinaryOp::Power,
        left: Box::new(expr.clone()),
        right: Box::new(Expr::Literal(2.0)),
    };

    let geometry = build_riemann_geometry(&squared_expr, interval, subdivisions, RiemannMethod::Midpoint)?;
    Ok(DiskVolumeResult {
        estimated_volume: core::f64::consts::PI * geometry.summary.approximation,
    })
}

pub fn estimate_disk_volume_graph(
    expr: &GraphExpr,
    interval: Interval,
    subdivisions: usize,
) -> Result<DiskVolumeResult, EngineError> {
    let body = explicit_panel_expression(expr).ok_or_else(|| {
        EngineError::InvalidInput(
            "disk volume requires a scalar expression or explicit y = f(x) graph expression"
                .to_string(),
        )
    })?;

    let squared_expr = GraphExpr::Scalar(ScalarExpr::Binary {
        op: crate::expr::ast::BinaryOp::Power,
        left: Box::new(body.clone()),
        right: Box::new(ScalarExpr::Literal(2.0)),
    });

    let geometry = crate::riemann::build_riemann_geometry_graph(
        &squared_expr,
        interval,
        subdivisions,
        RiemannMethod::Midpoint,
    )?;
    Ok(DiskVolumeResult {
        estimated_volume: core::f64::consts::PI * geometry.summary.approximation,
    })
}
