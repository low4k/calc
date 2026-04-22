use crate::types::WarningFlag;

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh3D {
    pub positions: Vec<f32>,
    pub normals: Vec<f32>,
    pub indices: Vec<u32>,
    pub warnings: Vec<WarningFlag>,
    pub bounds_min: [f64; 3],
    pub bounds_max: [f64; 3],
    pub max_radius: f64,
    pub estimated_volume: f64,
}

impl Mesh3D {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
            warnings: Vec::new(),
            bounds_min: [0.0, 0.0, 0.0],
            bounds_max: [0.0, 0.0, 0.0],
            max_radius: 0.0,
            estimated_volume: 0.0,
        }
    }
}

impl Default for Mesh3D {
    fn default() -> Self {
        Self::new()
    }
}
