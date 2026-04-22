export type RiemannMethod = 'left' | 'right' | 'midpoint' | 'trapezoid';

export interface RiemannBuildRequest {
  start: number;
  end: number;
  subdivisions: number;
  method: RiemannMethod;
}

export interface RiemannGeometrySnapshot {
  rectangles: Float32Array;
  trapezoids: Float32Array;
  approximation: number;
  referenceValue: number;
  absoluteError: number;
  warningCount: number;
}

export interface CurveSampleRequest {
  start: number;
  end: number;
  sampleCount: number;
}

export interface CurveSampleSnapshot {
  x: Float32Array;
  y: Float32Array;
}

export interface TaylorBuildRequest {
  center: number;
  degree: number;
  start: number;
  end: number;
  sampleCount: number;
}

export interface TaylorSeriesSnapshot {
  coefficients: Float32Array;
  sampleX: Float32Array;
  functionValues: Float32Array;
  polynomialValues: Float32Array;
  absoluteError: Float32Array;
}

export interface DiskMeshBuildRequest {
  start: number;
  end: number;
  axialSegments: number;
  radialSegments: number;
}

export interface DiskMeshSnapshot {
  positions: Float32Array;
  normals: Float32Array;
  indices: Uint32Array;
  boundsMin: Float32Array;
  boundsMax: Float32Array;
  maxRadius: number;
  estimatedVolume: number;
  warningCount: number;
}

export interface RiemannErrorSeriesRequest {
  start: number;
  end: number;
  method: RiemannMethod;
  counts: number[];
}

export interface RiemannErrorSeriesSnapshot {
  counts: Uint32Array;
  absoluteErrors: Float32Array;
  referenceValue: number;
}

export type GraphExpressionKind = 'scalar' | 'explicit-function' | 'implicit-relation';

export interface GraphExpressionSnapshot {
  kind: GraphExpressionKind;
  display: string;
  relation: string | null;
  leftExpression: string | null;
  rightExpression: string | null;
  backendExpression: string | null;
  backendEligible: boolean;
  warning: string | null;
}

export interface GraphCurveSnapshot {
  x: Float32Array;
  y: Float32Array;
}

export interface GraphRelationGridSnapshot {
  relation: string | null;
  resolution: number;
  xMin: number;
  xMax: number;
  yMin: number;
  yMax: number;
  vertexValues: Float32Array;
  cellValues: Float32Array;
}

interface RawWasmEngine {
  set_expression(expression: string): void;
  set_panel_graph_expression(expression: string): void;
  clear_panel_graph_expression(): void;
  set_graph_expression(expression: string): void;
  sample_graph_curve(start: number, end: number, sampleCount: number): void;
  build_graph_relation_grid(xMin: number, xMax: number, yMin: number, yMax: number, resolution: number): void;
  sample_curve(start: number, end: number, sampleCount: number): void;
  build_riemann_geometry(start: number, end: number, subdivisions: number, method: string): void;
  build_taylor_series(center: number, degree: number, start: number, end: number, sampleCount: number): void;
  build_disk_mesh(start: number, end: number, axialSegments: number, radialSegments: number): void;
  build_riemann_error_series(start: number, end: number, method: string, counts: Uint32Array): void;
  rectangle_buffer_view(): ArrayLike<number>;
  trapezoid_buffer_view(): ArrayLike<number>;
  approximation(): number;
  reference_value(): number;
  absolute_error(): number;
  warning_count(): number;
  curve_x_buffer_view(): ArrayLike<number>;
  curve_y_buffer_view(): ArrayLike<number>;
  taylor_coefficients_buffer_view(): ArrayLike<number>;
  taylor_sample_x_buffer_view(): ArrayLike<number>;
  taylor_function_values_buffer_view(): ArrayLike<number>;
  taylor_polynomial_values_buffer_view(): ArrayLike<number>;
  taylor_absolute_error_buffer_view(): ArrayLike<number>;
  disk_positions_buffer_view(): ArrayLike<number>;
  disk_normals_buffer_view(): ArrayLike<number>;
  disk_indices_buffer_view(): ArrayLike<number>;
  disk_bounds_min_buffer_view(): ArrayLike<number>;
  disk_bounds_max_buffer_view(): ArrayLike<number>;
  disk_mesh_warning_count(): number;
  disk_max_radius(): number;
  disk_estimated_volume(): number;
  error_counts_buffer_view(): ArrayLike<number>;
  error_values_buffer_view(): ArrayLike<number>;
  error_reference_value(): number;
  graph_expression_kind(): string | undefined;
  graph_expression_display(): string | undefined;
  graph_relation_operator(): string | undefined;
  graph_left_expression(): string | undefined;
  graph_right_expression(): string | undefined;
  graph_backend_expression(): string | undefined;
  graph_backend_eligible(): boolean;
  graph_warning(): string | undefined;
  graph_curve_x_buffer_view(): ArrayLike<number>;
  graph_curve_y_buffer_view(): ArrayLike<number>;
  relation_vertex_values_buffer_view(): ArrayLike<number>;
  relation_cell_values_buffer_view(): ArrayLike<number>;
  relation_grid_resolution(): number;
  relation_grid_x_min(): number;
  relation_grid_x_max(): number;
  relation_grid_y_min(): number;
  relation_grid_y_max(): number;
}

interface WasmBindings {
  default?: () => Promise<unknown>;
  WasmEngine: new () => RawWasmEngine;
}

type WasmModuleLoader = () => Promise<WasmBindings>;

async function defaultLoader(): Promise<WasmBindings> {
  const moduleUrl = `${window.location.origin}/pkg/calc_wasm.js`;
  return import(/* @vite-ignore */ moduleUrl) as Promise<WasmBindings>;
}

export class EngineClient {
  private readonly loader: WasmModuleLoader;
  private wasm: WasmBindings | null = null;
  private engine: RawWasmEngine | null = null;
  private operationChain: Promise<void> = Promise.resolve();

  constructor(loader: WasmModuleLoader = defaultLoader) {
    this.loader = loader;
  }

  async init(): Promise<void> {
    if (this.engine) {
      return;
    }

    this.wasm = await this.loader();
    if (this.wasm.default) {
      await this.wasm.default();
    }
    this.engine = new this.wasm.WasmEngine();
  }

  async setExpression(expression: string): Promise<void> {
    await this.withEngine((engine) => {
      engine.set_expression(expression);
    });
  }

  async setPanelGraphExpression(expression: string): Promise<void> {
    await this.withEngine((engine) => {
      engine.set_panel_graph_expression(expression);
    });
  }

  async clearPanelGraphExpression(): Promise<void> {
    await this.withEngine((engine) => {
      engine.clear_panel_graph_expression();
    });
  }

  async analyzeGraphExpression(expression: string): Promise<GraphExpressionSnapshot> {
    return this.withEngine((engine) => {
      engine.set_graph_expression(expression);

      const kind = engine.graph_expression_kind();
      const display = engine.graph_expression_display();
      if (!kind || !display) {
        throw new Error('Rust graph parser did not return a summary');
      }

      return {
        kind: kind as GraphExpressionKind,
        display,
        relation: engine.graph_relation_operator() ?? null,
        leftExpression: engine.graph_left_expression() ?? null,
        rightExpression: engine.graph_right_expression() ?? null,
        backendExpression: engine.graph_backend_expression() ?? null,
        backendEligible: engine.graph_backend_eligible(),
        warning: engine.graph_warning() ?? null,
      };
    });
  }

  async sampleGraphCurve(
    expression: string,
    request: CurveSampleRequest,
  ): Promise<GraphCurveSnapshot> {
    return this.withEngine((engine) => {
      engine.set_graph_expression(expression);
      engine.sample_graph_curve(request.start, request.end, request.sampleCount);

      return {
        x: Float32Array.from(engine.graph_curve_x_buffer_view()),
        y: Float32Array.from(engine.graph_curve_y_buffer_view()),
      };
    });
  }

  async buildGraphRelationGrid(
    expression: string,
    request: {
      xMin: number;
      xMax: number;
      yMin: number;
      yMax: number;
      resolution: number;
    },
  ): Promise<GraphRelationGridSnapshot> {
    return this.withEngine((engine) => {
      engine.set_graph_expression(expression);
      engine.build_graph_relation_grid(
        request.xMin,
        request.xMax,
        request.yMin,
        request.yMax,
        request.resolution,
      );

      return {
        relation: engine.graph_relation_operator() ?? null,
        resolution: engine.relation_grid_resolution(),
        xMin: engine.relation_grid_x_min(),
        xMax: engine.relation_grid_x_max(),
        yMin: engine.relation_grid_y_min(),
        yMax: engine.relation_grid_y_max(),
        vertexValues: Float32Array.from(engine.relation_vertex_values_buffer_view()),
        cellValues: Float32Array.from(engine.relation_cell_values_buffer_view()),
      };
    });
  }

  async buildRiemannGeometry(request: RiemannBuildRequest): Promise<RiemannGeometrySnapshot> {
    return this.withEngine((engine) => {
      engine.build_riemann_geometry(
        request.start,
        request.end,
        request.subdivisions,
        request.method,
      );

      return {
        rectangles: Float32Array.from(engine.rectangle_buffer_view()),
        trapezoids: Float32Array.from(engine.trapezoid_buffer_view()),
        approximation: engine.approximation(),
        referenceValue: engine.reference_value(),
        absoluteError: engine.absolute_error(),
        warningCount: engine.warning_count(),
      };
    });
  }

  async sampleCurve(request: CurveSampleRequest): Promise<CurveSampleSnapshot> {
    return this.withEngine((engine) => {
      engine.sample_curve(request.start, request.end, request.sampleCount);

      return {
        x: Float32Array.from(engine.curve_x_buffer_view()),
        y: Float32Array.from(engine.curve_y_buffer_view()),
      };
    });
  }

  async buildTaylorSeries(request: TaylorBuildRequest): Promise<TaylorSeriesSnapshot> {
    return this.withEngine((engine) => {
      engine.build_taylor_series(
        request.center,
        request.degree,
        request.start,
        request.end,
        request.sampleCount,
      );

      return {
        coefficients: Float32Array.from(engine.taylor_coefficients_buffer_view()),
        sampleX: Float32Array.from(engine.taylor_sample_x_buffer_view()),
        functionValues: Float32Array.from(engine.taylor_function_values_buffer_view()),
        polynomialValues: Float32Array.from(engine.taylor_polynomial_values_buffer_view()),
        absoluteError: Float32Array.from(engine.taylor_absolute_error_buffer_view()),
      };
    });
  }

  async buildDiskMesh(request: DiskMeshBuildRequest): Promise<DiskMeshSnapshot> {
    return this.withEngine((engine) => {
      engine.build_disk_mesh(
        request.start,
        request.end,
        request.axialSegments,
        request.radialSegments,
      );

      return {
        positions: Float32Array.from(engine.disk_positions_buffer_view()),
        normals: Float32Array.from(engine.disk_normals_buffer_view()),
        indices: Uint32Array.from(engine.disk_indices_buffer_view()),
        boundsMin: Float32Array.from(engine.disk_bounds_min_buffer_view()),
        boundsMax: Float32Array.from(engine.disk_bounds_max_buffer_view()),
        maxRadius: engine.disk_max_radius(),
        estimatedVolume: engine.disk_estimated_volume(),
        warningCount: engine.disk_mesh_warning_count(),
      };
    });
  }

  async buildRiemannErrorSeries(
    request: RiemannErrorSeriesRequest,
  ): Promise<RiemannErrorSeriesSnapshot> {
    return this.withEngine((engine) => {
      engine.build_riemann_error_series(
        request.start,
        request.end,
        request.method,
        Uint32Array.from(request.counts),
      );

      return {
        counts: Uint32Array.from(engine.error_counts_buffer_view()),
        absoluteErrors: Float32Array.from(engine.error_values_buffer_view()),
        referenceValue: engine.error_reference_value(),
      };
    });
  }

  private async requireEngine(): Promise<RawWasmEngine> {
    if (!this.engine) {
      await this.init();
    }

    if (!this.engine) {
      throw new Error('WASM engine failed to initialize');
    }

    return this.engine;
  }

  private async withEngine<T>(operation: (engine: RawWasmEngine) => T | Promise<T>): Promise<T> {
    let release: (() => void) | null = null;
    const next = new Promise<void>((resolve) => {
      release = resolve;
    });
    const previous = this.operationChain;
    this.operationChain = previous.catch(() => undefined).then(() => next);

    await previous.catch(() => undefined);

    try {
      const engine = await this.requireEngine();
      return await operation(engine);
    } finally {
      release?.();
    }
  }
}
