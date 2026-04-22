use calc_core::expr::{ast::{BinaryOp, Expr}, parse_expression};

#[test]
fn parses_variable_symbol() {
    let expr = parse_expression("x").expect("x should parse");
    assert_eq!(expr, Expr::Variable);
}

#[test]
fn keeps_power_right_associative() {
    let expr = parse_expression("2^3^2").expect("expression should parse");

    match expr {
        Expr::Binary {
            op: BinaryOp::Power,
            right,
            ..
        } => match *right {
            Expr::Binary {
                op: BinaryOp::Power,
                ..
            } => {}
            _ => panic!("expected right-associative power expression"),
        },
        _ => panic!("expected power expression"),
    }
}

#[test]
fn reports_invalid_character() {
    let error = parse_expression("x @ 2").expect_err("invalid character should fail");
    assert_eq!(error.offset, 2);
}
