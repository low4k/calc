#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EvaluationContext {
    pub x: f64,
}

impl EvaluationContext {
    pub fn new(x: f64) -> Self {
        Self { x }
    }
}
