use calc_core::eval::context::EvaluationContext;
use calc_core::eval::scalar::evaluate;
use calc_core::expr::parse_expression;
use calc_core::sampling::sample_expression_curve;
use calc_core::types::Interval;

#[test]
fn evaluates_simple_expression() {
    let expr = parse_expression("x + 2").expect("expression should parse");
    let value = evaluate(&expr, EvaluationContext::new(3.0)).expect("evaluation should succeed");
    assert_eq!(value, 5.0);
}

#[test]
fn rejects_log_domain_violation() {
    let expr = parse_expression("ln(x)").expect("expression should parse");
    let error = evaluate(&expr, EvaluationContext::new(0.0)).expect_err("ln(0) should fail");
    assert!(error.message.contains("positive value"));
}

#[test]
fn rejects_negative_fractional_power_in_real_domain() {
    let expr = parse_expression("(-1)^0.5").expect("expression should parse");
    let error = evaluate(&expr, EvaluationContext::new(0.0)).expect_err("real-domain power should fail");
    assert!(error.message.contains("integer exponent"));
}

#[test]
fn rejects_tangent_singularity() {
    let expr = parse_expression("tan(x)").expect("expression should parse");
    let error = evaluate(&expr, EvaluationContext::new(core::f64::consts::FRAC_PI_2))
        .expect_err("tan(pi/2) should fail");
    assert!(error.message.contains("undefined"));
}

#[test]
fn samples_curve_points() {
    let expr = parse_expression("x^2").expect("expression should parse");
    let samples = sample_expression_curve(
        &expr,
        Interval::new(0.0, 2.0).expect("interval should be valid"),
        3,
    )
    .expect("curve sampling should succeed");

    assert_eq!(samples.len(), 3);
    assert_eq!(samples[0].x, 0.0);
    assert_eq!(samples[1].x, 1.0);
    assert_eq!(samples[2].y, 4.0);
}
