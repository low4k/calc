# Folder Structure and Responsibilities

## Top Level

### crates/
Rust workspace crates live here.

### crates/calc-core/
Pure Rust engine. No browser, DOM, or wasm-bindgen dependencies.

Responsibilities:
- Expression parsing
- AST validation
- Scalar evaluation
- Automatic differentiation
- Numerical sampling
- Riemann geometry generation
- Disk-method mesh generation
- Taylor-series construction
- Numerical error estimation
- Testable deterministic outputs in world coordinates

Expected internal layout:
- src/expr: lexer, parser, AST, validation
- src/eval: scalar and dual-number evaluation
- src/sampling: interval sampling and discontinuity handling
- src/geometry: 2D and 3D primitive containers
- src/riemann: rectangle/trapezoid generation and error metrics
- src/revolution: disk-method mesh and volume estimate logic
- src/taylor: coefficient generation and approximation/error series
- tests: parser, eval, calculus feature tests

### crates/calc-wasm/
Thin WebAssembly bridge.

Responsibilities:
- Export wasm-bindgen interfaces
- Hold packed buffers for JS reads
- Map engine commands to calc-core calls
- Expose stable, minimal JS-facing API

Constraints:
- No math logic duplication from calc-core
- No browser rendering logic
- No UI state management

### frontend/
Browser application shell.

Responsibilities:
- Application state and controls
- Screen-space transforms
- 2D rendering
- 3D scene management
- User input handling
- Invoking the wasm engine and converting returned buffers into view models

Expected internal layout:
- src/app: app entry, state, orchestration
- src/wasm: engine loading, worker boundaries, buffer adapters
- src/render2d: canvas plot and overlays
- src/render3d: Three.js scene and mesh conversion
- src/features: UI panels for function input, Riemann, revolution, Taylor, error

### docs/
Design, milestone, validation, and architectural decision records.

Recommended docs to add next:
- architecture.md
- math-validation.md
- testing-strategy.md
- performance-targets.md
- ui-interaction-model.md

### docs/adr/
Architecture decision records.

### scripts/
Convenience scripts for local development.

Planned uses:
- wasm build wrapper
- frontend dev boot script
- benchmark runner
- validation snapshot generator

### examples/
Future demo presets and reference expressions.

Planned uses:
- JSON presets for common functions
- known-good datasets for regression checks

## Design Rules

1. Rust owns mathematical truth.
2. TypeScript owns presentation and interaction.
3. World-space coordinates are produced in Rust.
4. Screen-space transforms happen in the frontend.
5. Browser-specific objects never cross back into calc-core.
6. All major feature modules must be unit-tested before UI wiring.
7. Parser and evaluator must be stable before calculus modules expand.

## Build Sequence

1. Root Cargo workspace
2. calc-core crate manifests and type skeletons
3. calc-wasm crate bindings skeleton
4. frontend Vite bootstrap
5. integration scripts and docs
