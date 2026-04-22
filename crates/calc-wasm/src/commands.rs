use calc_core::{
    geometry::mesh::Mesh3D,
    graph::sampling::RelationGrid,
    riemann::{RectanglePrimitive, RiemannErrorSeries, RiemannGeometry, RiemannMethod, TrapezoidPrimitive},
    sampling::DomainSample,
    taylor::TaylorSeriesResult,
};
use wasm_bindgen::JsValue;

pub fn engine_error_to_js(message: String) -> JsValue {
    JsValue::from_str(&message)
}

pub fn parse_riemann_method(method: &str) -> Result<RiemannMethod, JsValue> {
    match method {
        "left" => Ok(RiemannMethod::Left),
        "right" => Ok(RiemannMethod::Right),
        "midpoint" => Ok(RiemannMethod::Midpoint),
        "trapezoid" => Ok(RiemannMethod::Trapezoid),
        _ => Err(engine_error_to_js(format!(
            "unknown riemann method '{method}'"
        ))),
    }
}

pub fn pack_rectangles(rectangles: &[RectanglePrimitive]) -> Vec<f32> {
    let mut packed = Vec::with_capacity(rectangles.len() * 5);
    for rectangle in rectangles {
        packed.extend_from_slice(&[
            rectangle.x as f32,
            rectangle.y as f32,
            rectangle.width as f32,
            rectangle.height as f32,
            rectangle.signed_area as f32,
        ]);
    }
    packed
}

pub fn pack_trapezoids(trapezoids: &[TrapezoidPrimitive]) -> Vec<f32> {
    let mut packed = Vec::with_capacity(trapezoids.len() * 6);
    for trapezoid in trapezoids {
        packed.extend_from_slice(&[
            trapezoid.x0 as f32,
            trapezoid.y0 as f32,
            trapezoid.x1 as f32,
            trapezoid.y1 as f32,
            trapezoid.baseline_y as f32,
            trapezoid.signed_area as f32,
        ]);
    }
    packed
}

pub fn warning_count(geometry: &RiemannGeometry) -> usize {
    geometry.warnings.len()
}

pub fn pack_f64(values: &[f64]) -> Vec<f32> {
    values.iter().map(|value| *value as f32).collect()
}

pub fn pack_mesh_positions(mesh: &Mesh3D) -> Vec<f32> {
    mesh.positions.clone()
}

pub fn pack_mesh_normals(mesh: &Mesh3D) -> Vec<f32> {
    mesh.normals.clone()
}

pub fn pack_mesh_indices(mesh: &Mesh3D) -> Vec<u32> {
    mesh.indices.clone()
}

pub fn mesh_warning_count(mesh: &Mesh3D) -> usize {
    mesh.warnings.len()
}

pub fn pack_mesh_bounds_min(mesh: &Mesh3D) -> Vec<f32> {
    mesh.bounds_min.iter().map(|value| *value as f32).collect()
}

pub fn pack_mesh_bounds_max(mesh: &Mesh3D) -> Vec<f32> {
    mesh.bounds_max.iter().map(|value| *value as f32).collect()
}

pub fn pack_taylor_coefficients(series: &TaylorSeriesResult) -> Vec<f32> {
    pack_f64(&series.coefficients)
}

pub fn pack_taylor_sample_x(series: &TaylorSeriesResult) -> Vec<f32> {
    pack_f64(&series.sample_x)
}

pub fn pack_taylor_function_values(series: &TaylorSeriesResult) -> Vec<f32> {
    pack_f64(&series.function_values)
}

pub fn pack_taylor_polynomial_values(series: &TaylorSeriesResult) -> Vec<f32> {
    pack_f64(&series.polynomial_values)
}

pub fn pack_taylor_absolute_error(series: &TaylorSeriesResult) -> Vec<f32> {
    pack_f64(&series.absolute_error)
}

pub fn pack_error_series_counts(series: &RiemannErrorSeries) -> Vec<u32> {
    series.subdivision_counts.clone()
}

pub fn pack_error_series_values(series: &RiemannErrorSeries) -> Vec<f32> {
    pack_f64(&series.absolute_errors)
}

pub fn pack_curve_x(samples: &[DomainSample]) -> Vec<f32> {
    samples.iter().map(|sample| sample.x as f32).collect()
}

pub fn pack_curve_y(samples: &[DomainSample]) -> Vec<f32> {
    samples.iter().map(|sample| sample.y as f32).collect()
}

pub fn pack_relation_vertex_values(grid: &RelationGrid) -> Vec<f32> {
    pack_f64(&grid.vertex_values)
}

pub fn pack_relation_cell_values(grid: &RelationGrid) -> Vec<f32> {
    pack_f64(&grid.cell_values)
}
