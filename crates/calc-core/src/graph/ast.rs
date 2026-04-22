use std::collections::BTreeSet;

use crate::expr::ast::{BinaryOp, UnaryOp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundOperator {
    Integral,
    Summation,
    Product,
}

impl BoundOperator {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Integral => "int",
            Self::Summation => "sum",
            Self::Product => "prod",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationOp {
    Equal,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    NotEqual,
}

impl RelationOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Equal => "=",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
            Self::NotEqual => "!=",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScalarExpr {
    Literal(f64),
    Variable(String),
    ConstantPi,
    ConstantE,
    Unary {
        op: UnaryOp,
        expr: Box<ScalarExpr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<ScalarExpr>,
        right: Box<ScalarExpr>,
    },
    FunctionCall {
        name: String,
        argument: Box<ScalarExpr>,
    },
    BoundCall {
        op: BoundOperator,
        variable: String,
        lower: Box<ScalarExpr>,
        upper: Box<ScalarExpr>,
        body: Box<ScalarExpr>,
    },
}

impl ScalarExpr {
    pub fn free_variables(&self) -> BTreeSet<String> {
        let mut variables = BTreeSet::new();
        self.collect_free_variables(&mut Vec::new(), &mut variables);
        variables
    }

    pub fn contains_bound_operator(&self) -> bool {
        match self {
            Self::Literal(_) | Self::Variable(_) | Self::ConstantPi | Self::ConstantE => false,
            Self::Unary { expr, .. } => expr.contains_bound_operator(),
            Self::Binary { left, right, .. } => {
                left.contains_bound_operator() || right.contains_bound_operator()
            }
            Self::FunctionCall { argument, .. } => argument.contains_bound_operator(),
            Self::BoundCall {
                lower,
                upper,
                body,
                ..
            } => {
                lower.contains_bound_operator()
                    || upper.contains_bound_operator()
                    || body.contains_bound_operator()
                    || true
            }
        }
    }

    fn collect_free_variables(
        &self,
        bound_variables: &mut Vec<String>,
        variables: &mut BTreeSet<String>,
    ) {
        match self {
            Self::Literal(_) | Self::ConstantPi | Self::ConstantE => {}
            Self::Variable(name) => {
                if !bound_variables.iter().any(|bound| bound == name) {
                    variables.insert(name.clone());
                }
            }
            Self::Unary { expr, .. } => expr.collect_free_variables(bound_variables, variables),
            Self::Binary { left, right, .. } => {
                left.collect_free_variables(bound_variables, variables);
                right.collect_free_variables(bound_variables, variables);
            }
            Self::FunctionCall { argument, .. } => {
                argument.collect_free_variables(bound_variables, variables)
            }
            Self::BoundCall {
                variable,
                lower,
                upper,
                body,
                ..
            } => {
                lower.collect_free_variables(bound_variables, variables);
                upper.collect_free_variables(bound_variables, variables);
                bound_variables.push(variable.clone());
                body.collect_free_variables(bound_variables, variables);
                bound_variables.pop();
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GraphExpr {
    Scalar(ScalarExpr),
    Relation {
        op: RelationOp,
        left: Box<ScalarExpr>,
        right: Box<ScalarExpr>,
    },
}

impl std::fmt::Display for BoundOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::fmt::Display for RelationOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::fmt::Display for ScalarExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_scalar_expr(self, f, 0)
    }
}

impl std::fmt::Display for GraphExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scalar(expr) => write!(f, "{expr}"),
            Self::Relation { op, left, right } => write!(f, "{left} {op} {right}"),
        }
    }
}

fn display_binary_op(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Multiply => "*",
        BinaryOp::Divide => "/",
        BinaryOp::Power => "^",
    }
}

fn fmt_scalar_expr(
    expr: &ScalarExpr,
    f: &mut std::fmt::Formatter<'_>,
    parent_precedence: u8,
) -> std::fmt::Result {
    let precedence = scalar_precedence(expr);
    let wrap = precedence < parent_precedence;

    if wrap {
        f.write_str("(")?;
    }

    match expr {
        ScalarExpr::Literal(value) => write_number(*value, f)?,
        ScalarExpr::Variable(name) => f.write_str(name)?,
        ScalarExpr::ConstantPi => f.write_str("pi")?,
        ScalarExpr::ConstantE => f.write_str("e")?,
        ScalarExpr::Unary { op, expr } => {
            match op {
                UnaryOp::Plus => f.write_str("+")?,
                UnaryOp::Minus => f.write_str("-")?,
            }
            fmt_scalar_expr(expr, f, precedence)?;
        }
        ScalarExpr::Binary { op, left, right } => {
            let right_precedence = if matches!(op, BinaryOp::Subtract | BinaryOp::Divide) {
                precedence + 1
            } else {
                precedence
            };
            fmt_scalar_expr(left, f, precedence)?;
            f.write_str(display_binary_op(*op))?;
            fmt_scalar_expr(right, f, right_precedence)?;
        }
        ScalarExpr::FunctionCall { name, argument } => {
            write!(f, "{name}(")?;
            fmt_scalar_expr(argument, f, 0)?;
            f.write_str(")")?;
        }
        ScalarExpr::BoundCall {
            op,
            variable,
            lower,
            upper,
            body,
        } => {
            write!(f, "{op}({variable},")?;
            fmt_scalar_expr(lower, f, 0)?;
            f.write_str(",")?;
            fmt_scalar_expr(upper, f, 0)?;
            f.write_str(",")?;
            fmt_scalar_expr(body, f, 0)?;
            f.write_str(")")?;
        }
    }

    if wrap {
        f.write_str(")")?;
    }

    Ok(())
}

fn scalar_precedence(expr: &ScalarExpr) -> u8 {
    match expr {
        ScalarExpr::Binary { op, .. } => match op {
            BinaryOp::Add | BinaryOp::Subtract => 1,
            BinaryOp::Multiply | BinaryOp::Divide => 2,
            BinaryOp::Power => 3,
        },
        ScalarExpr::Unary { .. } => 4,
        ScalarExpr::Literal(_)
        | ScalarExpr::Variable(_)
        | ScalarExpr::ConstantPi
        | ScalarExpr::ConstantE
        | ScalarExpr::FunctionCall { .. }
        | ScalarExpr::BoundCall { .. } => 5,
    }
}

fn write_number(value: f64, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if value.fract() == 0.0 {
        write!(f, "{:.0}", value)
    } else {
        write!(f, "{value}")
    }
}