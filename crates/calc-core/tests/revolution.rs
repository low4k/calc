use calc_core::expr::parse_expression;
use calc_core::revolution::{build_disk_mesh, estimate_disk_volume, DiskMeshRequest};
use calc_core::types::Interval;

#[test]
fn estimates_constant_radius_volume() {
    let expr = parse_expression("2").expect("expression should parse");
    let interval = Interval::new(0.0, 3.0).expect("interval should be valid");
    let volume = estimate_disk_volume(&expr, interval, 32).expect("volume estimate should succeed");

    let expected = 12.0 * core::f64::consts::PI;
    assert!((volume.estimated_volume - expected).abs() < 1e-10);
}

#[test]
fn builds_disk_mesh_for_constant_radius() {
    let expr = parse_expression("2").expect("expression should parse");
    let mesh = build_disk_mesh(
        &expr,
        DiskMeshRequest {
            interval: Interval::new(0.0, 3.0).expect("interval should be valid"),
            axial_segments: 4,
            radial_segments: 8,
        },
    )
    .expect("mesh should build");

    assert!(!mesh.positions.is_empty());
    assert_eq!(mesh.positions.len() % 3, 0);
    assert_eq!(mesh.normals.len(), mesh.positions.len());
    assert!(!mesh.indices.is_empty());
    assert!(mesh.warnings.is_empty());
    assert_eq!(mesh.max_radius, 2.0);
    assert_eq!(mesh.bounds_min, [0.0, -2.0, -2.0]);
    assert_eq!(mesh.bounds_max, [3.0, 2.0, 2.0]);
    assert!((mesh.estimated_volume - 12.0 * core::f64::consts::PI).abs() < 1e-10);
}

#[test]
fn flags_negative_radius_input() {
    let expr = parse_expression("-2").expect("expression should parse");
    let mesh = build_disk_mesh(
        &expr,
        DiskMeshRequest {
            interval: Interval::new(0.0, 1.0).expect("interval should be valid"),
            axial_segments: 2,
            radial_segments: 6,
        },
    )
    .expect("mesh should build");

    assert!(mesh
        .warnings
        .iter()
        .any(|warning| matches!(warning, calc_core::types::WarningFlag::NegativeRadius)));
}
