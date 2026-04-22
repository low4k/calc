# ADR 0001: Architecture Baseline

## Status
Accepted

## Context
The project goal is a high-performance calculus visualization engine where mathematical computation runs in Rust and the browser renders the results interactively through WebAssembly. The source brief includes Riemann sums, disk-method solids of revolution, Taylor-series approximation, and live numerical error visualization.

The main architectural risk is mixing mathematical logic, buffer transport, and rendering concerns too early. That would make correctness validation harder and would force rewrites when performance work begins.

## Decision
Adopt a three-layer design:

1. calc-core
Pure Rust domain engine with no web dependencies.

2. calc-wasm
A thin wasm-bindgen adapter that holds stateful engine buffers and exposes compact commands to JavaScript.

3. frontend
A TypeScript UI shell that transforms world-space outputs into 2D and 3D rendering primitives.

## Detailed decisions

### Numeric policy
- Use f64 internally in calc-core.
- Convert to f32 packed buffers only for rendering-oriented exports when justified.
- Preserve f64 summary metrics for volumes, integrals, and errors.

### Expression system policy
- Build and own an AST-based parser instead of delegating core behavior to a black-box evaluation crate.
- Support a constrained grammar first and widen only after tests prove the evaluator is stable.

### Differentiation policy
- Start with in-house forward-mode dual numbers.
- Guarantee Taylor support for lower degrees before attempting very high-order derivatives.
- Isolate derivative strategy behind a module boundary so it can later swap to a more advanced implementation if needed.

### Rendering policy
- 2D plotting targets canvas as the primary dense renderer.
- SVG is reserved for overlays, labels, and lightweight annotation shapes.
- 3D solids use Three.js in the frontend, with mesh vertices produced by Rust.

### Error-visualization policy
- Treat error visualization as a core engine feature.
- Compute reference values using a higher-accuracy internal numerical strategy, not the same coarse method being visualized.

## Consequences

Positive:
- Math logic becomes unit-testable outside the browser.
- The WASM layer stays replaceable and easy to profile.
- Frontend iteration remains fast without duplicating numerical code.
- New calculus modules can reuse parser, evaluator, geometry, and sampling layers.

Negative:
- More upfront design work is required before visible UI results.
- Higher-order Taylor support may require iteration if nested dual numbers prove complex.
- A custom parser increases initial implementation cost.

## Follow-up decisions needed
- Exact parser implementation approach: handwritten recursive descent vs parser combinators.
- Precise buffer export layout for each geometry type.
- Whether heavy calculations should move to a web worker at the first frontend integration pass.
