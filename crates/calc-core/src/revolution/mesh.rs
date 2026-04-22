use crate::error::EngineError;
use crate::expr::ast::Expr;
use crate::eval::{context::EvaluationContext, scalar::evaluate};
use crate::graph::{ast::GraphExpr, eval::{evaluate_scalar, GraphEvaluationContext}, explicit_panel_expression};
use crate::geometry::mesh::Mesh3D;
use crate::types::{Interval, WarningFlag};

use super::disk::{estimate_disk_volume, estimate_disk_volume_graph};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiskMeshRequest {
    pub interval: Interval,
    pub axial_segments: usize,
    pub radial_segments: usize,
}

pub fn build_disk_mesh(expr: &Expr, request: DiskMeshRequest) -> Result<Mesh3D, EngineError> {
    if request.axial_segments == 0 || request.radial_segments < 3 {
        return Err(EngineError::InvalidInput(
            "axial_segments must be > 0 and radial_segments must be >= 3".to_string(),
        ));
    }

    let mut mesh = Mesh3D::new();
    let axial_count = request.axial_segments + 1;
    let dx = request.interval.width() / request.axial_segments as f64;
    let mut radii = Vec::with_capacity(axial_count);
    let mut xs = Vec::with_capacity(axial_count);
    let mut saw_negative_radius = false;

    for index in 0..axial_count {
        let x = request.interval.start + dx * index as f64;
        let value = evaluate(expr, EvaluationContext::new(x))?;
        if value < 0.0 {
            saw_negative_radius = true;
        }
        xs.push(x);
        radii.push(value.abs());
    }

    if saw_negative_radius {
        mesh.warnings.push(WarningFlag::NegativeRadius);
    }

    mesh.max_radius = radii.iter().copied().fold(0.0, f64::max);
    mesh.bounds_min = [request.interval.start, -mesh.max_radius, -mesh.max_radius];
    mesh.bounds_max = [request.interval.end, mesh.max_radius, mesh.max_radius];
    mesh.estimated_volume = estimate_disk_volume(expr, request.interval, request.axial_segments * 4)?
        .estimated_volume;

    let derivatives = estimate_radius_derivatives(&radii, dx);
    build_side_surface(&mut mesh, &xs, &radii, &derivatives, request.radial_segments);
    add_caps(&mut mesh, &xs, &radii, request.radial_segments);

    Ok(mesh)
}

pub fn build_disk_mesh_graph(expr: &GraphExpr, request: DiskMeshRequest) -> Result<Mesh3D, EngineError> {
    let body = explicit_panel_expression(expr).ok_or_else(|| {
        EngineError::InvalidInput(
            "disk mesh requires a scalar expression or explicit y = f(x) graph expression"
                .to_string(),
        )
    })?;

    if request.axial_segments == 0 || request.radial_segments < 3 {
        return Err(EngineError::InvalidInput(
            "axial_segments must be > 0 and radial_segments must be >= 3".to_string(),
        ));
    }

    let mut mesh = Mesh3D::new();
    let axial_count = request.axial_segments + 1;
    let dx = request.interval.width() / request.axial_segments as f64;
    let mut radii = Vec::with_capacity(axial_count);
    let mut xs = Vec::with_capacity(axial_count);
    let mut saw_negative_radius = false;

    for index in 0..axial_count {
      let x = request.interval.start + dx * index as f64;
      let value = evaluate_scalar(body, &GraphEvaluationContext::new(x, 0.0))?;
      if value < 0.0 {
          saw_negative_radius = true;
      }
      xs.push(x);
      radii.push(value.abs());
    }

    if saw_negative_radius {
        mesh.warnings.push(WarningFlag::NegativeRadius);
    }

    mesh.max_radius = radii.iter().copied().fold(0.0, f64::max);
    mesh.bounds_min = [request.interval.start, -mesh.max_radius, -mesh.max_radius];
    mesh.bounds_max = [request.interval.end, mesh.max_radius, mesh.max_radius];
    mesh.estimated_volume = estimate_disk_volume_graph(expr, request.interval, request.axial_segments * 4)?
        .estimated_volume;

    let derivatives = estimate_radius_derivatives(&radii, dx);
    build_side_surface(&mut mesh, &xs, &radii, &derivatives, request.radial_segments);
    add_caps(&mut mesh, &xs, &radii, request.radial_segments);

    Ok(mesh)
}

fn estimate_radius_derivatives(radii: &[f64], dx: f64) -> Vec<f64> {
    let mut derivatives = Vec::with_capacity(radii.len());

    for index in 0..radii.len() {
        let derivative = if index == 0 {
            (radii[1] - radii[0]) / dx
        } else if index == radii.len() - 1 {
            (radii[index] - radii[index - 1]) / dx
        } else {
            (radii[index + 1] - radii[index - 1]) / (2.0 * dx)
        };
        derivatives.push(derivative);
    }

    derivatives
}

fn build_side_surface(
    mesh: &mut Mesh3D,
    xs: &[f64],
    radii: &[f64],
    derivatives: &[f64],
    radial_segments: usize,
) {
    for (ring_index, ((x, radius), derivative)) in xs
        .iter()
        .zip(radii.iter())
        .zip(derivatives.iter())
        .enumerate()
    {
        let _ = ring_index;
        for segment in 0..radial_segments {
            let theta = 2.0 * core::f64::consts::PI * segment as f64 / radial_segments as f64;
            let cos_theta = theta.cos();
            let sin_theta = theta.sin();

            mesh.positions.extend_from_slice(&[
                *x as f32,
                (*radius * cos_theta) as f32,
                (*radius * sin_theta) as f32,
            ]);

            let normal = normalize([-*derivative, cos_theta, sin_theta]);
            mesh.normals.extend_from_slice(&[
                normal[0] as f32,
                normal[1] as f32,
                normal[2] as f32,
            ]);
        }
    }

    for ring in 0..(xs.len() - 1) {
        let ring_start = ring * radial_segments;
        let next_ring_start = (ring + 1) * radial_segments;

        for segment in 0..radial_segments {
            let next_segment = (segment + 1) % radial_segments;
            let a = (ring_start + segment) as u32;
            let b = (next_ring_start + segment) as u32;
            let c = (next_ring_start + next_segment) as u32;
            let d = (ring_start + next_segment) as u32;

            mesh.indices.extend_from_slice(&[a, b, c, a, c, d]);
        }
    }
}

fn add_caps(mesh: &mut Mesh3D, xs: &[f64], radii: &[f64], radial_segments: usize) {
    add_cap(mesh, xs[0], radii[0], radial_segments, -1.0, true);
    add_cap(
        mesh,
        xs[xs.len() - 1],
        radii[radii.len() - 1],
        radial_segments,
        1.0,
        false,
    );
}

fn add_cap(
    mesh: &mut Mesh3D,
    x: f64,
    radius: f64,
    radial_segments: usize,
    normal_x: f64,
    reverse_winding: bool,
) {
    if radius <= 0.0 {
        return;
    }

    let center_index = (mesh.positions.len() / 3) as u32;
    mesh.positions.extend_from_slice(&[x as f32, 0.0, 0.0]);
    mesh.normals.extend_from_slice(&[normal_x as f32, 0.0, 0.0]);

    let ring_start = center_index + 1;
    for segment in 0..radial_segments {
        let theta = 2.0 * core::f64::consts::PI * segment as f64 / radial_segments as f64;
        mesh.positions.extend_from_slice(&[
            x as f32,
            (radius * theta.cos()) as f32,
            (radius * theta.sin()) as f32,
        ]);
        mesh.normals.extend_from_slice(&[normal_x as f32, 0.0, 0.0]);
    }

    for segment in 0..radial_segments {
        let next_segment = (segment + 1) % radial_segments;
        let current = ring_start + segment as u32;
        let next = ring_start + next_segment as u32;

        if reverse_winding {
            mesh.indices.extend_from_slice(&[center_index, next, current]);
        } else {
            mesh.indices.extend_from_slice(&[center_index, current, next]);
        }
    }
}

fn normalize(vector: [f64; 3]) -> [f64; 3] {
    let length = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
    if length == 0.0 {
        return [1.0, 0.0, 0.0];
    }

    [vector[0] / length, vector[1] / length, vector[2] / length]
}
