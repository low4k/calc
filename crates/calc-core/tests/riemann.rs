use calc_core::expr::parse_expression;
use calc_core::riemann::{build_error_series, build_riemann_geometry, RiemannMethod};
use calc_core::types::Interval;

#[test]
fn builds_midpoint_rectangles_for_constant_function() {
    let expr = parse_expression("2").expect("expression should parse");
    let interval = Interval::new(0.0, 2.0).expect("interval should be valid");
    let geometry = build_riemann_geometry(&expr, interval, 4, RiemannMethod::Midpoint)
        .expect("riemann geometry should build");

    assert_eq!(geometry.rectangles.len(), 4);
    assert_eq!(geometry.summary.approximation, 4.0);
    assert!((geometry.summary.reference_value - 4.0).abs() < 1e-10);
}

#[test]
fn trapezoid_rule_is_exact_for_linear_function() {
    let expr = parse_expression("x").expect("expression should parse");
    let interval = Interval::new(0.0, 2.0).expect("interval should be valid");
    let geometry = build_riemann_geometry(&expr, interval, 8, RiemannMethod::Trapezoid)
        .expect("trapezoid geometry should build");

    assert!((geometry.summary.approximation - 2.0).abs() < 1e-12);
    assert!(geometry.summary.absolute_error < 1e-10);
}

#[test]
fn error_series_shrinks_for_midpoint_on_quadratic() {
    let expr = parse_expression("x^2").expect("expression should parse");
    let interval = Interval::new(0.0, 1.0).expect("interval should be valid");
    let series = build_error_series(&expr, interval, &[4, 8, 16, 32], RiemannMethod::Midpoint)
        .expect("error series should build");

    assert_eq!(series.subdivision_counts, vec![4, 8, 16, 32]);
    assert!(series.absolute_errors[3] < series.absolute_errors[0]);
}
