use calc_core::{
    graph::{
        eval::{evaluate_scalar, GraphEvaluationContext},
        parse_graph_expression,
        sampling::{sample_graph_curve, sample_relation_grid},
    },
    types::Interval,
};

#[test]
fn evaluates_bound_integral_with_runtime_x() {
    let expr = parse_graph_expression("int(t, 0, x, t)").expect("graph expression should parse");
    let scalar = match expr {
        calc_core::graph::ast::GraphExpr::Scalar(expr) => expr,
        _ => panic!("expected scalar graph expression"),
    };

    let value = evaluate_scalar(&scalar, &GraphEvaluationContext::new(4.0, 0.0))
        .expect("integral should evaluate");
    assert!((value - 8.0).abs() < 5e-3);
}

#[test]
fn evaluates_bound_sum() {
    let expr = parse_graph_expression("sum(n,1,3,n*x)").expect("graph expression should parse");
    let scalar = match expr {
        calc_core::graph::ast::GraphExpr::Scalar(expr) => expr,
        _ => panic!("expected scalar graph expression"),
    };

    let value = evaluate_scalar(&scalar, &GraphEvaluationContext::new(2.0, 0.0))
        .expect("sum should evaluate");
    assert_eq!(value, 12.0);
}

#[test]
fn samples_explicit_graph_curve_with_bound_operator() {
    let expr = parse_graph_expression("y = int(t, 0, x, t)").expect("graph expression should parse");
    let samples = sample_graph_curve(
        &expr,
        Interval::new(0.0, 2.0).expect("interval should be valid"),
        3,
    )
    .expect("curve should sample");

    assert_eq!(samples.len(), 3);
    assert!((samples[2].y - 2.0).abs() < 5e-3);
}

#[test]
fn samples_relation_grid_for_circle() {
    let expr = parse_graph_expression("x^2 + y^2 = 1").expect("graph expression should parse");
    let grid = sample_relation_grid(
        &expr,
        Interval::new(-1.0, 1.0).expect("x interval should be valid"),
        Interval::new(-1.0, 1.0).expect("y interval should be valid"),
        4,
    )
    .expect("relation grid should sample");

    assert_eq!(grid.vertex_values.len(), 25);
    assert_eq!(grid.cell_values.len(), 16);
    assert!(grid.cell_values[5] < 0.0);
}