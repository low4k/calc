use calc_core::graph::{
    analyze_graph_expression, parse_graph_expression, GraphExpressionKind,
};

#[test]
fn parses_explicit_function_relation() {
    let summary = analyze_graph_expression("y = sin(x)").expect("graph expression should parse");

    assert_eq!(summary.kind, GraphExpressionKind::ExplicitFunction);
    assert_eq!(summary.backend_expression.as_deref(), Some("sin(x)"));
    assert!(summary.backend_eligible);
}

#[test]
fn parses_implicit_relation_with_x_and_y() {
    let summary = analyze_graph_expression("x^2 + y^2 = 1").expect("relation should parse");

    assert_eq!(summary.kind, GraphExpressionKind::ImplicitRelation);
    assert_eq!(summary.relation.map(|relation| relation.as_str()), Some("="));
    assert_eq!(summary.left_expression.as_deref(), Some("x^2+y^2"));
    assert_eq!(summary.right_expression.as_deref(), Some("1"));
}

#[test]
fn parses_bound_operator_scalar() {
    let summary = analyze_graph_expression("int(t, 0, 1, t^2)").expect("bound operator should parse");

    assert_eq!(summary.kind, GraphExpressionKind::Scalar);
    assert!(summary.backend_eligible);
    assert_eq!(summary.backend_expression.as_deref(), Some("int(t,0,1,t^2)"));
    assert!(summary.display.contains("int(t,0,1"));
}

#[test]
fn parses_implicit_multiplication() {
    let expression = parse_graph_expression("xy=1").expect("implicit multiplication should parse");
    let summary = calc_core::graph::summarize_graph_expression(&expression);

    assert_eq!(summary.kind, GraphExpressionKind::ImplicitRelation);
    assert_eq!(summary.left_expression.as_deref(), Some("x*y"));
}