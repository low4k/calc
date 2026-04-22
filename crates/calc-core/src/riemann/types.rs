use crate::types::WarningFlag;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiemannMethod {
    Left,
    Right,
    Midpoint,
    Trapezoid,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectanglePrimitive {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub signed_area: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrapezoidPrimitive {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    pub baseline_y: f64,
    pub signed_area: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RiemannSummary {
    pub approximation: f64,
    pub reference_value: f64,
    pub absolute_error: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RiemannGeometry {
    pub rectangles: Vec<RectanglePrimitive>,
    pub trapezoids: Vec<TrapezoidPrimitive>,
    pub summary: RiemannSummary,
    pub warnings: Vec<WarningFlag>,
}
