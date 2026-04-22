pub mod error;
pub mod rectangles;
pub mod types;

pub use error::{build_error_series, build_error_series_graph, estimate_reference_value_graph, RiemannErrorSeries};
pub use rectangles::{build_riemann_geometry, build_riemann_geometry_graph};
pub use types::{RectanglePrimitive, RiemannGeometry, RiemannMethod, RiemannSummary, TrapezoidPrimitive};
