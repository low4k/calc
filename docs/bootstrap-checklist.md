# Bootstrap Checklist

This checklist is the exact order for the first real coding pass.

## Step 1: Root workspace files
Create:
- Cargo.toml
- .gitignore

Cargo workspace should declare members:
- crates/calc-core
- crates/calc-wasm

.gitignore should cover:
- target/
- node_modules/
- dist/
- pkg/
- .DS_Store
- frontend/.vite/

## Step 2: calc-core crate shell
Create:
- crates/calc-core/Cargo.toml
- crates/calc-core/src/lib.rs
- crates/calc-core/src/error.rs
- crates/calc-core/src/types.rs

Initial responsibilities:
- lib.rs re-exports stable public modules
- error.rs contains project-wide core error enums
- types.rs contains interval and geometry structs that multiple modules reuse

## Step 3: expression system files
Create:
- crates/calc-core/src/expr/mod.rs
- crates/calc-core/src/expr/ast.rs
- crates/calc-core/src/expr/lexer.rs
- crates/calc-core/src/expr/parser.rs
- crates/calc-core/src/expr/validate.rs

Do first:
- AST definitions
- token enum
- lexer
- precedence parser
- semantic validation

Do not do first:
- syntax sugar expansions
- optimizer passes
- exotic function support

## Step 4: evaluator files
Create:
- crates/calc-core/src/eval/mod.rs
- crates/calc-core/src/eval/context.rs
- crates/calc-core/src/eval/scalar.rs
- crates/calc-core/src/eval/dual.rs

Implementation order:
1. scalar evaluator
2. error plumbing
3. dual number type
4. derivative-capable evaluator

## Step 5: sampling and geometry files
Create:
- crates/calc-core/src/sampling/mod.rs
- crates/calc-core/src/sampling/domain.rs
- crates/calc-core/src/geometry/mod.rs
- crates/calc-core/src/geometry/polyline.rs
- crates/calc-core/src/geometry/mesh.rs

Focus:
- deterministic interval stepping
- discontinuity markers
- packed geometry-friendly outputs

## Step 6: first visible feature
Create:
- crates/calc-core/src/riemann/mod.rs
- crates/calc-core/src/riemann/types.rs
- crates/calc-core/src/riemann/rectangles.rs
- crates/calc-core/src/riemann/error.rs
- crates/calc-core/tests/riemann.rs

Reason:
Riemann geometry gives the quickest end-to-end proof that parser, evaluator, sampling, geometry, and summary metrics all work together.

## Step 7: Taylor and revolution modules
Create only after the above passes tests.

## Step 8: WASM bridge
Create:
- crates/calc-wasm/Cargo.toml
- crates/calc-wasm/src/lib.rs
- crates/calc-wasm/src/buffers.rs
- crates/calc-wasm/src/commands.rs

Bridge rules:
- command API only
- internal stateful engine object
- stable packed buffer layouts

## Step 9: frontend shell
Create:
- frontend/package.json
- frontend/tsconfig.json
- frontend/vite.config.ts
- frontend/index.html
- frontend/src/main.ts
- frontend/src/app/App.ts
- frontend/src/app/state.ts
- frontend/src/app/types.ts
- frontend/src/wasm/engine.ts
- frontend/src/wasm/worker.ts
- frontend/src/render2d/canvasPlot.ts
- frontend/src/render2d/svgOverlay.ts
- frontend/src/render3d/scene.ts
- frontend/src/render3d/revolutionMesh.ts
- frontend/src/features/functionInput.ts
- frontend/src/features/riemannPanel.ts
- frontend/src/features/revolutionPanel.ts
- frontend/src/features/taylorPanel.ts
- frontend/src/features/errorPanel.ts
- frontend/src/styles.css

## Step 10: wire features in this order
1. parse expression
2. 2D curve sample
3. Riemann geometry
4. error series
5. Taylor series
6. disk mesh

This order minimizes debugging surface area.
