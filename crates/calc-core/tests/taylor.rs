use calc_core::expr::parse_expression;
use calc_core::taylor::{build_taylor_series, TaylorSeriesRequest};
use calc_core::types::Interval;

#[test]
fn builds_real_series_for_exp() {
    let expr = parse_expression("exp(x)").expect("expression should parse");
    let series = build_taylor_series(
        &expr,
        TaylorSeriesRequest {
            center: 0.0,
            degree: 4,
            interval: Interval::new(-1.0, 1.0).expect("interval should be valid"),
            sample_count: 17,
        },
    )
    .expect("taylor series should build");

    assert_eq!(series.coefficients.len(), 5);
    assert_eq!(series.sample_x.len(), 17);
    assert!((series.coefficients[0] - 1.0).abs() < 1e-6);
    assert!((series.coefficients[1] - 1.0).abs() < 1e-4);
    assert!((series.coefficients[2] - 0.5).abs() < 1e-3);
}

#[test]
fn builds_real_series_for_sine() {
    let expr = parse_expression("sin(x)").expect("expression should parse");
    let series = build_taylor_series(
        &expr,
        TaylorSeriesRequest {
            center: 0.0,
            degree: 5,
            interval: Interval::new(-1.0, 1.0).expect("interval should be valid"),
            sample_count: 11,
        },
    )
    .expect("taylor series should build");

    assert!(series.coefficients[0].abs() < 1e-8);
    assert!((series.coefficients[1] - 1.0).abs() < 1e-4);
    assert!(series.coefficients[2].abs() < 1e-4);
    assert!((series.coefficients[3] + 1.0 / 6.0).abs() < 1e-3);
}
