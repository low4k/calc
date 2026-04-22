use calc_wasm::WasmEngine;

#[test]
fn stores_explicit_graph_expression_summary() {
    let mut engine = WasmEngine::new();
    engine
        .set_graph_expression("y = sin(x)".to_string())
        .expect("graph expression should parse");

    assert_eq!(engine.graph_expression_kind().as_deref(), Some("explicit-function"));
    assert_eq!(engine.graph_expression_display().as_deref(), Some("y = sin(x)"));
    assert_eq!(engine.graph_backend_expression().as_deref(), Some("sin(x)"));
    assert!(engine.graph_backend_eligible());
}

#[test]
fn stores_implicit_graph_expression_summary() {
    let mut engine = WasmEngine::new();
    engine
        .set_graph_expression("x^2 + y^2 = 1".to_string())
        .expect("graph expression should parse");

    assert_eq!(engine.graph_expression_kind().as_deref(), Some("implicit-relation"));
    assert_eq!(engine.graph_relation_operator().as_deref(), Some("="));
    assert_eq!(engine.graph_left_expression().as_deref(), Some("x^2+y^2"));
    assert_eq!(engine.graph_right_expression().as_deref(), Some("1"));
    assert!(!engine.graph_backend_eligible());
}

#[test]
fn samples_graph_curve_through_wasm() {
    let mut engine = WasmEngine::new();
    engine
        .set_graph_expression("y = int(t, 0, x, t)".to_string())
        .expect("graph expression should parse");
    engine
        .sample_graph_curve(0.0, 2.0, 3)
        .expect("graph curve should sample");

    let y = engine.graph_curve_y_values().to_vec();
    assert_eq!(y.len(), 3);
    assert!((y[2] as f64 - 2.0).abs() < 5e-2);
}

#[test]
fn samples_relation_grid_through_wasm() {
    let mut engine = WasmEngine::new();
    engine
        .set_graph_expression("x^2 + y^2 = 1".to_string())
        .expect("graph expression should parse");
    engine
        .build_graph_relation_grid(-1.0, 1.0, -1.0, 1.0, 4)
        .expect("relation grid should sample");

    assert_eq!(engine.relation_grid_resolution(), 4);
    assert_eq!(engine.relation_vertex_values().len(), 25);
    assert_eq!(engine.relation_cell_values().len(), 16);
}