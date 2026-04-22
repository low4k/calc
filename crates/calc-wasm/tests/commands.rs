use calc_core::{
    geometry::mesh::Mesh3D,
    riemann::{RectanglePrimitive, RiemannErrorSeries, RiemannMethod, TrapezoidPrimitive},
    sampling::DomainSample,
    taylor::TaylorSeriesResult,
};
use calc_wasm::commands::{
    pack_error_series_counts, pack_error_series_values,
    pack_curve_x, pack_curve_y,
    mesh_warning_count, pack_mesh_bounds_max, pack_mesh_bounds_min, pack_mesh_indices,
    pack_mesh_normals, pack_mesh_positions,
    pack_rectangles, pack_taylor_absolute_error, pack_taylor_coefficients,
    pack_taylor_function_values, pack_taylor_polynomial_values, pack_taylor_sample_x,
    pack_trapezoids, parse_riemann_method,
};

#[test]
fn parses_supported_riemann_method_names() {
    let method = parse_riemann_method("midpoint").expect("method should parse");
    assert_eq!(method, RiemannMethod::Midpoint);
}

#[test]
fn packs_rectangle_geometry_into_flat_buffer() {
    let packed = pack_rectangles(&[RectanglePrimitive {
        x: 1.0,
        y: 2.0,
        width: 3.0,
        height: 4.0,
        signed_area: 12.0,
    }]);

    assert_eq!(packed, vec![1.0, 2.0, 3.0, 4.0, 12.0]);
}

#[test]
fn packs_trapezoid_geometry_into_flat_buffer() {
    let packed = pack_trapezoids(&[TrapezoidPrimitive {
        x0: 0.0,
        y0: 1.0,
        x1: 2.0,
        y1: 3.0,
        baseline_y: 0.0,
        signed_area: 4.0,
    }]);

    assert_eq!(packed, vec![0.0, 1.0, 2.0, 3.0, 0.0, 4.0]);
}

#[test]
fn packs_taylor_series_vectors() {
    let series = TaylorSeriesResult {
        center: 0.0,
        coefficients: vec![1.0, 2.0],
        sample_x: vec![-1.0, 1.0],
        function_values: vec![0.5, 3.0],
        polynomial_values: vec![0.75, 2.75],
        absolute_error: vec![0.25, 0.25],
    };

    assert_eq!(pack_taylor_coefficients(&series), vec![1.0, 2.0]);
    assert_eq!(pack_taylor_sample_x(&series), vec![-1.0, 1.0]);
    assert_eq!(pack_taylor_function_values(&series), vec![0.5, 3.0]);
    assert_eq!(pack_taylor_polynomial_values(&series), vec![0.75, 2.75]);
    assert_eq!(pack_taylor_absolute_error(&series), vec![0.25, 0.25]);
}

#[test]
fn packs_mesh_vectors_and_counts_warnings() {
    let mut mesh = Mesh3D::new();
    mesh.positions = vec![0.0, 1.0, 2.0];
    mesh.normals = vec![1.0, 0.0, 0.0];
    mesh.indices = vec![0, 1, 2];
    mesh.bounds_min = [-1.0, -2.0, -3.0];
    mesh.bounds_max = [4.0, 5.0, 6.0];
    mesh.max_radius = 6.0;
    mesh.estimated_volume = 12.0;
    mesh.warnings.push(calc_core::types::WarningFlag::NegativeRadius);

    assert_eq!(pack_mesh_positions(&mesh), vec![0.0, 1.0, 2.0]);
    assert_eq!(pack_mesh_normals(&mesh), vec![1.0, 0.0, 0.0]);
    assert_eq!(pack_mesh_indices(&mesh), vec![0, 1, 2]);
    assert_eq!(pack_mesh_bounds_min(&mesh), vec![-1.0, -2.0, -3.0]);
    assert_eq!(pack_mesh_bounds_max(&mesh), vec![4.0, 5.0, 6.0]);
    assert_eq!(mesh_warning_count(&mesh), 1);
}

#[test]
fn packs_error_series_vectors() {
    let series = RiemannErrorSeries {
        subdivision_counts: vec![4, 8, 16],
        absolute_errors: vec![0.5, 0.2, 0.1],
        reference_value: 2.0,
    };

    assert_eq!(pack_error_series_counts(&series), vec![4, 8, 16]);
    assert_eq!(pack_error_series_values(&series), vec![0.5, 0.2, 0.1]);
}

#[test]
fn packs_curve_samples() {
    let samples = vec![
        DomainSample { x: -1.0, y: 1.0 },
        DomainSample { x: 0.0, y: 0.0 },
        DomainSample { x: 1.0, y: 1.0 },
    ];

    assert_eq!(pack_curve_x(&samples), vec![-1.0, 0.0, 1.0]);
    assert_eq!(pack_curve_y(&samples), vec![1.0, 0.0, 1.0]);
}
