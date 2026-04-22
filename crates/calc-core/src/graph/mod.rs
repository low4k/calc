pub mod ast;
pub mod eval;
pub mod parser;
pub mod sampling;
pub mod validate;

use ast::{GraphExpr, RelationOp, ScalarExpr};
use crate::error::ParseError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphExpressionKind {
    Scalar,
    ExplicitFunction,
    ImplicitRelation,
}

impl GraphExpressionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Scalar => "scalar",
            Self::ExplicitFunction => "explicit-function",
            Self::ImplicitRelation => "implicit-relation",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphExpressionSummary {
    pub kind: GraphExpressionKind,
    pub display: String,
    pub relation: Option<RelationOp>,
    pub left_expression: Option<String>,
    pub right_expression: Option<String>,
    pub backend_expression: Option<String>,
    pub backend_eligible: bool,
    pub warning: Option<String>,
}

pub fn parse_graph_expression(input: &str) -> Result<GraphExpr, ParseError> {
    let mut parser = parser::GraphExpressionParser::new(input)?;
    let expr = parser.parse()?;
    validate::validate_graph_expression(&expr)?;
    Ok(expr)
}

pub fn analyze_graph_expression(input: &str) -> Result<GraphExpressionSummary, ParseError> {
    let expr = parse_graph_expression(input)?;
    Ok(summarize_graph_expression(&expr))
}

pub fn summarize_graph_expression(expr: &GraphExpr) -> GraphExpressionSummary {
    match expr {
        GraphExpr::Scalar(expr) => summarize_scalar(expr),
        GraphExpr::Relation { op, left, right } => summarize_relation(*op, left, right),
    }
}

pub fn explicit_panel_expression(expr: &GraphExpr) -> Option<&ScalarExpr> {
    match expr {
        GraphExpr::Scalar(expr) => Some(expr),
        GraphExpr::Relation {
            op: RelationOp::Equal,
            left,
            right,
        } if is_named_variable(left, "y") => Some(right),
        GraphExpr::Relation {
            op: RelationOp::Equal,
            left,
            right,
        } if is_named_variable(right, "y") => Some(left),
        _ => None,
    }
}

fn summarize_scalar(expr: &ScalarExpr) -> GraphExpressionSummary {
    let expression = trim_outer_parens(expr.to_string());
    let backend_eligible = is_backend_eligible(expr);
    GraphExpressionSummary {
        kind: GraphExpressionKind::Scalar,
        display: expression.clone(),
        relation: None,
        left_expression: Some(expression.clone()),
        right_expression: None,
        backend_expression: backend_eligible.then(|| expression.clone()),
        backend_eligible,
        warning: (!backend_eligible).then(|| {
            "scalar graph expressions that use y or bound operators are not yet executable by the Rust calculus panels".to_string()
        }),
    }
}

fn summarize_relation(
    relation: RelationOp,
    left: &ScalarExpr,
    right: &ScalarExpr,
) -> GraphExpressionSummary {
    let left_expression = trim_outer_parens(left.to_string());
    let right_expression = trim_outer_parens(right.to_string());

    if relation == RelationOp::Equal && is_named_variable(left, "y") {
        let backend_eligible = is_backend_eligible(right);
        return GraphExpressionSummary {
            kind: GraphExpressionKind::ExplicitFunction,
            display: format!("y = {right_expression}"),
            relation: Some(relation),
            left_expression: Some(left_expression),
            right_expression: Some(right_expression.clone()),
            backend_expression: backend_eligible.then(|| right_expression.clone()),
            backend_eligible,
            warning: (!backend_eligible).then(|| {
                "this explicit function cannot yet be executed by the Rust calculus panels".to_string()
            }),
        };
    }

    if relation == RelationOp::Equal && is_named_variable(right, "y") {
        let backend_eligible = is_backend_eligible(left);
        return GraphExpressionSummary {
            kind: GraphExpressionKind::ExplicitFunction,
            display: format!("y = {left_expression}"),
            relation: Some(relation),
            left_expression: Some(left_expression.clone()),
            right_expression: Some(right_expression),
            backend_expression: backend_eligible.then(|| left_expression.clone()),
            backend_eligible,
            warning: (!backend_eligible).then(|| {
                "this explicit function cannot yet be executed by the Rust calculus panels".to_string()
            }),
        };
    }

    GraphExpressionSummary {
        kind: GraphExpressionKind::ImplicitRelation,
        display: format!("{left_expression} {} {right_expression}", relation.as_str()),
        relation: Some(relation),
        left_expression: Some(left_expression),
        right_expression: Some(right_expression),
        backend_expression: None,
        backend_eligible: false,
        warning: (relation == RelationOp::NotEqual)
            .then(|| "not-equal relations are parsed, but downstream execution is not implemented yet".to_string()),
    }
}

fn is_named_variable(expr: &ScalarExpr, name: &str) -> bool {
    matches!(expr, ScalarExpr::Variable(variable) if variable == name)
}

fn is_backend_eligible(expr: &ScalarExpr) -> bool {
    expr.free_variables().iter().all(|variable| variable == "x")
}

fn trim_outer_parens(expression: String) -> String {
    let mut result = expression;
    while result.starts_with('(') && result.ends_with(')') {
        let mut depth = 0usize;
        let mut balanced = true;
        for (index, ch) in result.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 && index < result.len() - 1 {
                        balanced = false;
                        break;
                    }
                }
                _ => {}
            }
        }
        if !balanced || depth != 0 {
            break;
        }
        result = result[1..result.len() - 1].to_string();
    }
    result
}