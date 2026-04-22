use crate::types::Diagnostic;

#[derive(Debug, Clone, PartialEq)]
pub struct Polyline2D {
    pub points: Vec<f64>,
    pub segment_breaks: Vec<usize>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Polyline2D {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            segment_breaks: Vec::new(),
            diagnostics: Vec::new(),
        }
    }
}

impl Default for Polyline2D {
    fn default() -> Self {
        Self::new()
    }
}
