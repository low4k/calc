pub mod buffers;
pub mod commands;

use buffers::{FloatBuffer, UIntBuffer};
use calc_core::{
    expr::{ast::Expr, parse_expression},
    graph::{analyze_graph_expression, ast::GraphExpr, parse_graph_expression, sampling::{sample_graph_curve, sample_relation_grid}},
    revolution::{build_disk_mesh as build_disk_mesh_core, build_disk_mesh_graph as build_disk_mesh_graph_core, DiskMeshRequest},
    riemann::{build_error_series as build_error_series_core, build_error_series_graph as build_error_series_graph_core, build_riemann_geometry, build_riemann_geometry_graph},
    sampling::sample_expression_curve,
    taylor::{build_taylor_series as build_taylor_series_core, build_taylor_series_graph as build_taylor_series_graph_core, TaylorSeriesRequest},
    types::Interval,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmEngine {
    expression_source: Option<String>,
    parsed_expression: Option<Expr>,
    panel_graph_expression_source: Option<String>,
    panel_graph_expression: Option<GraphExpr>,
    parsed_graph_expression: Option<GraphExpr>,
    graph_expression_source: Option<String>,
    graph_expression_kind: Option<String>,
    graph_expression_display: Option<String>,
    graph_relation_operator: Option<String>,
    graph_left_expression: Option<String>,
    graph_right_expression: Option<String>,
    graph_backend_expression: Option<String>,
    graph_backend_eligible: bool,
    graph_warning: Option<String>,
    graph_curve_x_buffer: FloatBuffer,
    graph_curve_y_buffer: FloatBuffer,
    relation_vertex_values_buffer: FloatBuffer,
    relation_cell_values_buffer: FloatBuffer,
    relation_grid_resolution: usize,
    relation_grid_x_min: f64,
    relation_grid_x_max: f64,
    relation_grid_y_min: f64,
    relation_grid_y_max: f64,
    rectangle_buffer: FloatBuffer,
    trapezoid_buffer: FloatBuffer,
    approximation: f64,
    reference_value: f64,
    absolute_error: f64,
    warning_count: usize,
    disk_positions_buffer: FloatBuffer,
    disk_normals_buffer: FloatBuffer,
    disk_indices_buffer: UIntBuffer,
    disk_bounds_min_buffer: FloatBuffer,
    disk_bounds_max_buffer: FloatBuffer,
    disk_mesh_warning_count: usize,
    disk_max_radius: f64,
    disk_estimated_volume: f64,
    error_counts_buffer: UIntBuffer,
    error_values_buffer: FloatBuffer,
    error_reference_value: f64,
    curve_x_buffer: FloatBuffer,
    curve_y_buffer: FloatBuffer,
    taylor_coefficients_buffer: FloatBuffer,
    taylor_sample_x_buffer: FloatBuffer,
    taylor_function_values_buffer: FloatBuffer,
    taylor_polynomial_values_buffer: FloatBuffer,
    taylor_absolute_error_buffer: FloatBuffer,
}

#[wasm_bindgen]
impl WasmEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            expression_source: None,
            parsed_expression: None,
            panel_graph_expression_source: None,
            panel_graph_expression: None,
            parsed_graph_expression: None,
            graph_expression_source: None,
            graph_expression_kind: None,
            graph_expression_display: None,
            graph_relation_operator: None,
            graph_left_expression: None,
            graph_right_expression: None,
            graph_backend_expression: None,
            graph_backend_eligible: false,
            graph_warning: None,
            graph_curve_x_buffer: FloatBuffer::new(),
            graph_curve_y_buffer: FloatBuffer::new(),
            relation_vertex_values_buffer: FloatBuffer::new(),
            relation_cell_values_buffer: FloatBuffer::new(),
            relation_grid_resolution: 0,
            relation_grid_x_min: 0.0,
            relation_grid_x_max: 0.0,
            relation_grid_y_min: 0.0,
            relation_grid_y_max: 0.0,
            rectangle_buffer: FloatBuffer::new(),
            trapezoid_buffer: FloatBuffer::new(),
            approximation: 0.0,
            reference_value: 0.0,
            absolute_error: 0.0,
            warning_count: 0,
            disk_positions_buffer: FloatBuffer::new(),
            disk_normals_buffer: FloatBuffer::new(),
            disk_indices_buffer: UIntBuffer::new(),
            disk_bounds_min_buffer: FloatBuffer::new(),
            disk_bounds_max_buffer: FloatBuffer::new(),
            disk_mesh_warning_count: 0,
            disk_max_radius: 0.0,
            disk_estimated_volume: 0.0,
            error_counts_buffer: UIntBuffer::new(),
            error_values_buffer: FloatBuffer::new(),
            error_reference_value: 0.0,
            curve_x_buffer: FloatBuffer::new(),
            curve_y_buffer: FloatBuffer::new(),
            taylor_coefficients_buffer: FloatBuffer::new(),
            taylor_sample_x_buffer: FloatBuffer::new(),
            taylor_function_values_buffer: FloatBuffer::new(),
            taylor_polynomial_values_buffer: FloatBuffer::new(),
            taylor_absolute_error_buffer: FloatBuffer::new(),
        }
    }

    pub fn set_expression(&mut self, input: String) -> Result<(), JsValue> {
        let parsed = parse_expression(&input)
            .map_err(|error| commands::engine_error_to_js(error.to_string()))?;
        self.expression_source = Some(input);
        self.parsed_expression = Some(parsed);
        self.panel_graph_expression_source = None;
        self.panel_graph_expression = None;
        Ok(())
    }

    pub fn set_panel_graph_expression(&mut self, input: String) -> Result<(), JsValue> {
        let parsed = parse_graph_expression(&input)
            .map_err(|error| commands::engine_error_to_js(error.to_string()))?;
        self.expression_source = Some(input.clone());
        self.panel_graph_expression_source = Some(input);
        self.panel_graph_expression = Some(parsed);
        self.parsed_expression = None;
        Ok(())
    }

    pub fn clear_panel_graph_expression(&mut self) {
        self.panel_graph_expression_source = None;
        self.panel_graph_expression = None;
        self.expression_source = None;
        self.parsed_expression = None;
    }

    pub fn set_graph_expression(&mut self, input: String) -> Result<(), JsValue> {
        let parsed = parse_graph_expression(&input)
            .map_err(|error| commands::engine_error_to_js(error.to_string()))?;
        let summary = analyze_graph_expression(&input)
            .map_err(|error| commands::engine_error_to_js(error.to_string()))?;

        self.parsed_graph_expression = Some(parsed);
        self.graph_expression_source = Some(input);
        self.graph_expression_kind = Some(summary.kind.as_str().to_string());
        self.graph_expression_display = Some(summary.display);
        self.graph_relation_operator = summary.relation.map(|relation| relation.as_str().to_string());
        self.graph_left_expression = summary.left_expression;
        self.graph_right_expression = summary.right_expression;
        self.graph_backend_expression = summary.backend_expression;
        self.graph_backend_eligible = summary.backend_eligible;
        self.graph_warning = summary.warning;

        Ok(())
    }

    pub fn sample_graph_curve(
        &mut self,
        start: f64,
        end: f64,
        sample_count: usize,
    ) -> Result<(), JsValue> {
        let expr = self.parsed_graph_expression.as_ref().ok_or_else(|| {
            commands::engine_error_to_js("set_graph_expression must be called first".to_string())
        })?;
        let interval = Interval::new(start, end)
            .map_err(|error| commands::engine_error_to_js(error.to_string()))?;
        let samples = sample_graph_curve(expr, interval, sample_count)
            .map_err(|error| commands::engine_error_to_js(error.to_string()))?;

        self.graph_curve_x_buffer
            .replace(commands::pack_curve_x(&samples));
        self.graph_curve_y_buffer
            .replace(commands::pack_curve_y(&samples));
        Ok(())
    }

    pub fn build_graph_relation_grid(
        &mut self,
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
        resolution: usize,
    ) -> Result<(), JsValue> {
        let expr = self.parsed_graph_expression.as_ref().ok_or_else(|| {
            commands::engine_error_to_js("set_graph_expression must be called first".to_string())
        })?;
        let x_interval = Interval::new(x_min, x_max)
            .map_err(|error| commands::engine_error_to_js(error.to_string()))?;
        let y_interval = Interval::new(y_min, y_max)
            .map_err(|error| commands::engine_error_to_js(error.to_string()))?;
        let grid = sample_relation_grid(expr, x_interval, y_interval, resolution)
            .map_err(|error| commands::engine_error_to_js(error.to_string()))?;

        self.relation_vertex_values_buffer
            .replace(commands::pack_relation_vertex_values(&grid));
        self.relation_cell_values_buffer
            .replace(commands::pack_relation_cell_values(&grid));
        self.relation_grid_resolution = grid.resolution;
        self.relation_grid_x_min = grid.x_interval.start;
        self.relation_grid_x_max = grid.x_interval.end;
        self.relation_grid_y_min = grid.y_interval.start;
        self.relation_grid_y_max = grid.y_interval.end;
        Ok(())
    }

    pub fn build_riemann_geometry(
        &mut self,
        start: f64,
        end: f64,
        subdivisions: usize,
        method: &str,
    ) -> Result<(), JsValue> {
        let interval = Interval::new(start, end)
            .map_err(|error| commands::engine_error_to_js(error.to_string()))?;
        let method = commands::parse_riemann_method(method)?;
        let geometry = if let Some(expr) = self.panel_graph_expression.as_ref() {
            build_riemann_geometry_graph(expr, interval, subdivisions, method)
        } else {
            let expr = self
                .parsed_expression
                .as_ref()
                .ok_or_else(|| commands::engine_error_to_js("set_expression or set_panel_graph_expression must be called first".to_string()))?;
            build_riemann_geometry(expr, interval, subdivisions, method)
        }
        .map_err(|error| commands::engine_error_to_js(error.to_string()))?;

        self.rectangle_buffer
            .replace(commands::pack_rectangles(&geometry.rectangles));
        self.trapezoid_buffer
            .replace(commands::pack_trapezoids(&geometry.trapezoids));
        self.approximation = geometry.summary.approximation;
        self.reference_value = geometry.summary.reference_value;
        self.absolute_error = geometry.summary.absolute_error;
        self.warning_count = commands::warning_count(&geometry);

        Ok(())
    }

    pub fn build_taylor_series(
        &mut self,
        center: f64,
        degree: usize,
        start: f64,
        end: f64,
        sample_count: usize,
    ) -> Result<(), JsValue> {
        let interval = Interval::new(start, end)
            .map_err(|error| commands::engine_error_to_js(error.to_string()))?;
        let request = TaylorSeriesRequest {
            center,
            degree,
            interval,
            sample_count,
        };
        let series = if let Some(expr) = self.panel_graph_expression.as_ref() {
            build_taylor_series_graph_core(expr, request)
        } else {
            let expr = self
                .parsed_expression
                .as_ref()
                .ok_or_else(|| commands::engine_error_to_js("set_expression or set_panel_graph_expression must be called first".to_string()))?;
            build_taylor_series_core(expr, request)
        }
        .map_err(|error| commands::engine_error_to_js(error.to_string()))?;

        self.taylor_coefficients_buffer
            .replace(commands::pack_taylor_coefficients(&series));
        self.taylor_sample_x_buffer
            .replace(commands::pack_taylor_sample_x(&series));
        self.taylor_function_values_buffer
            .replace(commands::pack_taylor_function_values(&series));
        self.taylor_polynomial_values_buffer
            .replace(commands::pack_taylor_polynomial_values(&series));
        self.taylor_absolute_error_buffer
            .replace(commands::pack_taylor_absolute_error(&series));

        Ok(())
    }

    pub fn sample_curve(
        &mut self,
        start: f64,
        end: f64,
        sample_count: usize,
    ) -> Result<(), JsValue> {
        let expr = self
            .parsed_expression
            .as_ref()
            .ok_or_else(|| commands::engine_error_to_js("set_expression must be called first".to_string()))?;
        let interval = Interval::new(start, end)
            .map_err(|error| commands::engine_error_to_js(error.to_string()))?;
        let samples = sample_expression_curve(expr, interval, sample_count)
            .map_err(|error| commands::engine_error_to_js(error.to_string()))?;

        self.curve_x_buffer.replace(commands::pack_curve_x(&samples));
        self.curve_y_buffer.replace(commands::pack_curve_y(&samples));
        Ok(())
    }

    pub fn build_riemann_error_series(
        &mut self,
        start: f64,
        end: f64,
        method: &str,
        counts: &[u32],
    ) -> Result<(), JsValue> {
        let interval = Interval::new(start, end)
            .map_err(|error| commands::engine_error_to_js(error.to_string()))?;
        let method = commands::parse_riemann_method(method)?;
        let series = if let Some(expr) = self.panel_graph_expression.as_ref() {
            build_error_series_graph_core(expr, interval, counts, method)
        } else {
            let expr = self
                .parsed_expression
                .as_ref()
                .ok_or_else(|| commands::engine_error_to_js("set_expression or set_panel_graph_expression must be called first".to_string()))?;
            build_error_series_core(expr, interval, counts, method)
        }
        .map_err(|error| commands::engine_error_to_js(error.to_string()))?;

        self.error_counts_buffer
            .replace(commands::pack_error_series_counts(&series));
        self.error_values_buffer
            .replace(commands::pack_error_series_values(&series));
        self.error_reference_value = series.reference_value;

        Ok(())
    }

    pub fn build_disk_mesh(
        &mut self,
        start: f64,
        end: f64,
        axial_segments: usize,
        radial_segments: usize,
    ) -> Result<(), JsValue> {
        let interval = Interval::new(start, end)
            .map_err(|error| commands::engine_error_to_js(error.to_string()))?;
        let request = DiskMeshRequest {
            interval,
            axial_segments,
            radial_segments,
        };
        let mesh = if let Some(expr) = self.panel_graph_expression.as_ref() {
            build_disk_mesh_graph_core(expr, request)
        } else {
            let expr = self
                .parsed_expression
                .as_ref()
                .ok_or_else(|| commands::engine_error_to_js("set_expression or set_panel_graph_expression must be called first".to_string()))?;
            build_disk_mesh_core(expr, request)
        }
        .map_err(|error| commands::engine_error_to_js(error.to_string()))?;

        self.disk_positions_buffer
            .replace(commands::pack_mesh_positions(&mesh));
        self.disk_normals_buffer
            .replace(commands::pack_mesh_normals(&mesh));
        self.disk_indices_buffer
            .replace(commands::pack_mesh_indices(&mesh));
        self.disk_bounds_min_buffer
            .replace(commands::pack_mesh_bounds_min(&mesh));
        self.disk_bounds_max_buffer
            .replace(commands::pack_mesh_bounds_max(&mesh));
        self.disk_mesh_warning_count = commands::mesh_warning_count(&mesh);
        self.disk_max_radius = mesh.max_radius;
        self.disk_estimated_volume = mesh.estimated_volume;

        Ok(())
    }

    pub fn expression_source(&self) -> Option<String> {
        self.expression_source.clone()
    }

    pub fn graph_expression_source(&self) -> Option<String> {
        self.graph_expression_source.clone()
    }

    pub fn graph_expression_kind(&self) -> Option<String> {
        self.graph_expression_kind.clone()
    }

    pub fn graph_expression_display(&self) -> Option<String> {
        self.graph_expression_display.clone()
    }

    pub fn graph_relation_operator(&self) -> Option<String> {
        self.graph_relation_operator.clone()
    }

    pub fn graph_left_expression(&self) -> Option<String> {
        self.graph_left_expression.clone()
    }

    pub fn graph_right_expression(&self) -> Option<String> {
        self.graph_right_expression.clone()
    }

    pub fn graph_backend_expression(&self) -> Option<String> {
        self.graph_backend_expression.clone()
    }

    pub fn graph_backend_eligible(&self) -> bool {
        self.graph_backend_eligible
    }

    pub fn graph_warning(&self) -> Option<String> {
        self.graph_warning.clone()
    }

    pub fn graph_curve_x_buffer_view(&self) -> js_sys::Float32Array {
        self.graph_curve_x_buffer.view()
    }

    pub fn graph_curve_y_buffer_view(&self) -> js_sys::Float32Array {
        self.graph_curve_y_buffer.view()
    }

    pub fn relation_vertex_values_buffer_view(&self) -> js_sys::Float32Array {
        self.relation_vertex_values_buffer.view()
    }

    pub fn relation_cell_values_buffer_view(&self) -> js_sys::Float32Array {
        self.relation_cell_values_buffer.view()
    }

    pub fn relation_grid_resolution(&self) -> usize {
        self.relation_grid_resolution
    }

    pub fn relation_grid_x_min(&self) -> f64 {
        self.relation_grid_x_min
    }

    pub fn relation_grid_x_max(&self) -> f64 {
        self.relation_grid_x_max
    }

    pub fn relation_grid_y_min(&self) -> f64 {
        self.relation_grid_y_min
    }

    pub fn relation_grid_y_max(&self) -> f64 {
        self.relation_grid_y_max
    }

    pub fn rectangle_buffer_view(&self) -> js_sys::Float32Array {
        self.rectangle_buffer.view()
    }

    pub fn trapezoid_buffer_view(&self) -> js_sys::Float32Array {
        self.trapezoid_buffer.view()
    }

    pub fn approximation(&self) -> f64 {
        self.approximation
    }

    pub fn reference_value(&self) -> f64 {
        self.reference_value
    }

    pub fn absolute_error(&self) -> f64 {
        self.absolute_error
    }

    pub fn warning_count(&self) -> usize {
        self.warning_count
    }

    pub fn disk_positions_buffer_view(&self) -> js_sys::Float32Array {
        self.disk_positions_buffer.view()
    }

    pub fn disk_normals_buffer_view(&self) -> js_sys::Float32Array {
        self.disk_normals_buffer.view()
    }

    pub fn disk_indices_buffer_view(&self) -> js_sys::Uint32Array {
        self.disk_indices_buffer.view()
    }

    pub fn disk_bounds_min_buffer_view(&self) -> js_sys::Float32Array {
        self.disk_bounds_min_buffer.view()
    }

    pub fn disk_bounds_max_buffer_view(&self) -> js_sys::Float32Array {
        self.disk_bounds_max_buffer.view()
    }

    pub fn disk_mesh_warning_count(&self) -> usize {
        self.disk_mesh_warning_count
    }

    pub fn disk_max_radius(&self) -> f64 {
        self.disk_max_radius
    }

    pub fn disk_estimated_volume(&self) -> f64 {
        self.disk_estimated_volume
    }

    pub fn error_counts_buffer_view(&self) -> js_sys::Uint32Array {
        self.error_counts_buffer.view()
    }

    pub fn error_values_buffer_view(&self) -> js_sys::Float32Array {
        self.error_values_buffer.view()
    }

    pub fn error_reference_value(&self) -> f64 {
        self.error_reference_value
    }

    pub fn curve_x_buffer_view(&self) -> js_sys::Float32Array {
        self.curve_x_buffer.view()
    }

    pub fn curve_y_buffer_view(&self) -> js_sys::Float32Array {
        self.curve_y_buffer.view()
    }

    pub fn taylor_coefficients_buffer_view(&self) -> js_sys::Float32Array {
        self.taylor_coefficients_buffer.view()
    }

    pub fn taylor_sample_x_buffer_view(&self) -> js_sys::Float32Array {
        self.taylor_sample_x_buffer.view()
    }

    pub fn taylor_function_values_buffer_view(&self) -> js_sys::Float32Array {
        self.taylor_function_values_buffer.view()
    }

    pub fn taylor_polynomial_values_buffer_view(&self) -> js_sys::Float32Array {
        self.taylor_polynomial_values_buffer.view()
    }

    pub fn taylor_absolute_error_buffer_view(&self) -> js_sys::Float32Array {
        self.taylor_absolute_error_buffer.view()
    }
}

impl Default for WasmEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmEngine {
    pub fn graph_curve_x_values(&self) -> &[f32] {
        self.graph_curve_x_buffer.values()
    }

    pub fn graph_curve_y_values(&self) -> &[f32] {
        self.graph_curve_y_buffer.values()
    }

    pub fn relation_vertex_values(&self) -> &[f32] {
        self.relation_vertex_values_buffer.values()
    }

    pub fn relation_cell_values(&self) -> &[f32] {
        self.relation_cell_values_buffer.values()
    }
}
