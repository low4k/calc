# Implementation Plan

## 1. Project Objective

Build a browser-based calculus engine where Rust is the system of record for mathematical computation and TypeScript is responsible only for interaction and rendering.

The first release should support:
- Function parsing from user input
- 2D curve plotting
- Riemann sum visualization using left, right, midpoint, and trapezoid rules
- Numerical error visualization as subdivision count changes
- Disk-method solids of revolution around the x-axis
- Taylor-series approximation with live comparison against the source function

The system should be designed so new numerical-analysis modules can be added without changing the frontend contract shape.

## 2. Product Boundaries

### In scope for the initial major build
- Real-valued single-variable functions of x
- Browser delivery via WebAssembly
- Deterministic geometry generation in Rust
- Stateful engine object in Rust with update commands
- Packed array outputs for fast transfer to JS
- Canvas-based 2D rendering
- Three.js-based 3D rendering for revolution solids
- Unit and integration tests for all mathematical modules

### Out of scope for the initial major build
- Symbolic algebra system
- Differential equations
- Multi-variable surfaces beyond the disk-method solid generated from one function
- Collaborative editing
- Cloud sync
- Native desktop packaging
- Mobile app packaging
- Arbitrary axes of revolution beyond the x-axis in release 1

## 3. Engineering Principles

1. Rust owns mathematical correctness.
2. The parser is not optional; all downstream features depend on it.
3. Geometry returned from Rust stays in world coordinates.
4. Frontend code must never recalculate the core math for correctness-critical features.
5. All public engine outputs require explicit tests and documented invariants.
6. Performance optimizations happen after correctness and buffer shapes are stable.
7. Error visualization is a primary feature, not a summary statistic.

## 4. Repository Layout

The repository is organized into the following main components:

- crates/calc-core
- crates/calc-wasm
- frontend
- docs
- scripts
- examples

### calc-core purpose
Provide all pure math logic.

### calc-wasm purpose
Expose a stable JS-facing command surface and efficient packed buffers.

### frontend purpose
Render views, manage interaction state, and call the wasm bridge.

## 5. High-Level Architecture

### Layer 1: calc-core
Core responsibilities:
- AST definitions
- Parsing and validation
- Scalar evaluation
- Dual-number evaluation
- Sampling routines
- Numerical integration helpers
- Riemann geometry output
- Disk mesh output
- Taylor coefficient and sample generation
- Error estimation

This crate must compile and test without wasm-bindgen or browser assumptions.

### Layer 2: calc-wasm
Core responsibilities:
- wasm-bindgen exports
- Engine state persistence between UI updates
- Buffer allocation and reuse
- Conversion from core structs to packed arrays
- Stable command-oriented API for JS consumers

This layer should be thin and unsurprising.

### Layer 3: frontend
Core responsibilities:
- Controls and UI panels
- Camera and viewport transforms
- Canvas drawing for 2D data
- Three.js scene creation for 3D data
- Event handling and optimistic interaction flow
- Displaying warnings, parse errors, and domain problems returned by Rust

## 6. Recommended Data Contracts

These are the shapes that should exist conceptually, even if exact Rust struct names vary.

### Shared conceptual inputs
- ExpressionInput: expression string and optional metadata
- Interval: a, b with validation that a < b for most operations
- ViewWindow2D: x_min, x_max, y_min, y_max
- Resolution: sample counts or mesh partition counts

### Expression-related outputs
- ParsedExpressionHandle or parsed engine state entry
- Parse diagnostics with token location and human-readable reason

### Curve outputs
- Polyline2D with packed points [x0, y0, x1, y1, ...]
- Segment breaks or discontinuity indices
- Warnings for invalid sample segments

### Riemann outputs
- Rectangles buffer packed as [x, y, width, height, signed_area]
- Trapezoids buffer packed as [x0, y0, x1, y1, baseline_y, signed_area]
- Summary metrics: approximation, reference_value, absolute_error, relative_error
- Flags for discontinuity, invalid sample, or unreliable reference value

### Error-series outputs
- Packed points [n, error]
- Optional log-transformed series if requested
- Comparison series per method

### Disk-method outputs
- Positions [x, y, z]
- Normals [nx, ny, nz]
- Indices [i0, i1, i2]
- Optional edge list for wireframe rendering
- Metadata: estimated volume, max radius, bounds, warnings

### Taylor outputs
- Coefficients [c0, c1, ...]
- Polynomial sample points
- Source function sample points
- Error series sample points
- Diagnostics for unstable or unavailable higher-order derivatives

## 7. Detailed Phase Plan

## Phase 0: Lock architecture and conventions

### Goal
Avoid structural rework later.

### Tasks
- Confirm Rust workspace layout.
- Confirm Vite + TypeScript frontend choice.
- Confirm wasm-bindgen + wasm-pack packaging path.
- Lock numeric policy: f64 in core, f32 only for render buffers when appropriate.
- Lock coordinate conventions and buffer packing policy.
- Document non-goals to prevent scope drift.

### Exit criteria
- ADR created
- Folder structure created
- Planning documents committed to the repo

## Phase 1: Bootstrap repository manifests

### Goal
Create buildable empty shells.

### Tasks
- Add root Cargo.toml workspace members.
- Add calc-core Cargo.toml.
- Add calc-wasm Cargo.toml.
- Add frontend package.json, tsconfig.json, vite.config.ts.
- Add root .gitignore.
- Add minimal build scripts under scripts/.

### Exit criteria
- cargo metadata succeeds
- frontend package installation succeeds
- wasm package build command path is documented

## Phase 2: Core type system and error model

### Goal
Define stable contracts before feature logic.

### Tasks
- Add EngineError enum with parse, eval, domain, and internal variants.
- Add ParseError with source-span support.
- Add Interval type with validation constructor.
- Add geometry containers for Polyline2D and Mesh3D.
- Add Riemann primitive types.
- Add Taylor result and error-series types.
- Add warning and diagnostic structs so recoverable issues do not become panics.

### Exit criteria
- Core types compile
- Type-level invariants are documented
- Unit tests cover interval and error categorization basics

## Phase 3: Expression parsing system

### Goal
Turn user strings into reliable ASTs.

### Tasks
- Implement tokenizer for numbers, identifiers, operators, and parentheses.
- Implement recursive-descent parser with explicit operator precedence.
- Support unary operators, binary operators, functions, constants, and x.
- Enforce right-associative exponentiation.
- Add post-parse semantic validation.
- Add parser size limits to prevent abuse cases.
- Provide detailed diagnostics with positions.

### Minimum grammar
- literals: integers and decimals
- variable: x
- constants: pi, e
- unary: +, -
- binary: +, -, *, /, ^
- parentheses
- functions: sin, cos, tan, asin, acos, atan, exp, ln, log, sqrt, abs

### Tests
- operator precedence matrix
- associativity cases for exponentiation
- nested function parsing
- invalid token errors
- unmatched parenthesis errors
- unknown function errors

### Exit criteria
- Parser test suite passes
- AST output is stable enough for evaluator use

## Phase 4: Scalar evaluation

### Goal
Evaluate any valid AST safely at a given x.

### Tasks
- Implement scalar evaluation visitor.
- Add constant and function evaluation.
- Add runtime domain checks.
- Return rich errors for division by zero, invalid log/sqrt inputs, and non-finite results.
- Add a reusable EvaluationContext type if needed.

### Important edge cases
- ln(x) for x <= 0
- sqrt(x) for x < 0
- tan(x) near singularities
- 0^0 policy must be defined and documented
- negative base with non-integer exponent must stay in reals only if valid under the chosen policy

### Exit criteria
- Eval tests cover valid and invalid domains
- Evaluation is deterministic and panic-free for user inputs

## Phase 5: Sampling and curve generation

### Goal
Generate 2D plot-ready function samples.

### Tasks
- Build fixed-step interval sampler.
- Mark discontinuities and invalid intervals.
- Produce polyline-compatible buffers and segment breaks.
- Add optional oversampling near rapid variation later, but not initially.

### Rules
- Do not connect points across known invalid regions.
- Preserve world coordinates.
- Return warnings rather than silently hiding evaluation problems.

### Exit criteria
- Basic function plots can be drawn correctly in the frontend once wired
- Discontinuities no longer produce incorrect connecting lines

## Phase 6: Automatic differentiation foundation

### Goal
Support Taylor series without hardcoding derivatives.

### Tasks
- Implement a Dual type containing value and first derivative.
- Overload arithmetic and supported unary functions.
- Add evaluator path for AST on dual numbers.
- Define strategy for higher-order derivatives.

### Recommended release-1 strategy
- Support first-order dual numbers directly.
- Use nested dual or repeated derivative evaluation for higher-order derivatives up to a guaranteed degree threshold.
- Cap guaranteed Taylor degree initially at 12.

### Risk management
If nested dual complexity becomes too high early, isolate a fallback numerical derivative module behind a clear abstraction and mark its output as approximate.

### Exit criteria
- First derivative matches known analytic results across test set
- Coefficient generation works reliably for supported orders

## Phase 7: Taylor series module

### Goal
Generate polynomial approximations and convergence visuals.

### Tasks
- Compute coefficients around center a.
- Evaluate polynomial samples across a chosen interval.
- Generate source function samples over the same interval.
- Generate pointwise absolute error samples.
- Return diagnostics when derivatives fail or become unstable.

### Validation set
- exp(x) around 0
- sin(x) around 0
- cos(x) around 0
- ln(1 + x) around 0 with domain-limited interval

### Exit criteria
- Polynomial visibly converges in known-good ranges
- Error plot matches expected local behavior

## Phase 8: Riemann geometry module

### Goal
Produce pedagogically correct shapes, not just totals.

### Tasks
- Validate interval and subdivision count n.
- Compute delta x.
- Implement left, right, midpoint, and trapezoid rules.
- Return packed rectangle or trapezoid geometry.
- Return signed area contributions and aggregate summaries.
- Add reference estimate using adaptive Simpson or much higher-resolution numeric estimate.

### Method-specific rules
- Left: sample at left endpoint
- Right: sample at right endpoint
- Midpoint: sample at center of each interval
- Trapezoid: connect endpoint heights linearly for each subinterval

### Important behavior
- Negative function values stay negative and produce downward primitives.
- Geometry should remain faithful to the selected numerical method.
- Crossing the x-axis inside a subinterval does not alter the primitive shape unless a later feature explicitly adds clipped pedagogical overlays.

### Tests
- Constant functions
- Linear functions where trapezoid should be exact
- Symmetric functions over symmetric intervals
- Negative-valued function cases
- Increasing n should reduce error for stable test functions

### Exit criteria
- Riemann visuals and summary metrics agree with external references

## Phase 9: Numerical error visualization

### Goal
Make convergence behavior a first-class visual.

### Tasks
- Add engine calls that compute E(n) across a configurable series of n values.
- Support one or multiple methods in the same response.
- Return point series for direct plotting.
- Support log-scale transformation in the frontend or precompute it in Rust when requested.
- Flag unreliable results when the reference estimate fails.

### Recommended n sequence
- 4, 8, 16, 32, 64, 128 by default
- Allow user configuration later

### Exit criteria
- Error panel can render a shrinking curve for standard smooth functions
- Method comparison visibly distinguishes midpoint/trapezoid from left/right on common examples

## Phase 10: Disk-method volume and mesh module

### Goal
Generate both scalar volume estimates and renderable 3D geometry.

### Tasks
- Sample x positions across the interval.
- Evaluate radius r = f(x).
- Enforce release-1 policy for radius validity.
- Generate circular rings at each x using theta partitions.
- Build triangle indices between adjacent rings.
- Generate normals with consistent winding.
- Add optional end caps.
- Compute scalar volume using pi integral of f(x)^2.

### Release-1 radius policy
- Default: require non-negative radius over the interval for direct interpretation.
- If negative values are encountered, return warning and either reject or require an explicit absolute-radius mode later.

### Tests
- Constant radius cylinder
- Linear radius cone/frustum-like profile depending on interval
- Zero-radius edge cases at one or both ends

### Exit criteria
- Mesh renders without inverted normals
- Volume estimate matches known references within tolerance

## Phase 11: WASM bridge

### Goal
Expose the engine to JS efficiently.

### Tasks
- Create stateful WasmEngine struct.
- Add methods to parse expression and update parameters.
- Store buffers internally to avoid repeated allocations when possible.
- Expose typed-array-compatible views.
- Keep command names stable and minimal.

### Likely command surface
- set_expression
- sample_curve
- build_riemann_geometry
- estimate_riemann_error_series
- build_disk_mesh
- build_taylor_series

### Performance rules
- Prefer parse-once, evaluate-many workflow.
- Reuse allocated vectors where practical.
- Avoid returning deeply nested JS objects for large geometry payloads.

### Exit criteria
- JS can request each major feature without manual memory copying hacks beyond standard wasm-bindgen patterns

## Phase 12: Frontend shell

### Goal
Make the engine interactive and legible.

### Tasks
- Build app state model.
- Add expression input with parse diagnostics.
- Add plot canvas and overlay axes.
- Add Riemann panel controls: method, interval, n.
- Add error panel controls and chart.
- Add Taylor panel controls: center, degree, comparison visibility.
- Add revolution panel controls: interval, axial resolution, radial resolution.
- Add loading and warning states for all feature requests.

### UI rules
- Frontend must display engine warnings clearly.
- Do not hide invalid math inputs behind silent fallbacks.
- Keep controls responsive by pushing heavy calls to a worker if main-thread performance suffers.

### Exit criteria
- Core interaction loop works end-to-end
- A user can change function, parameters, and see all target visualizations update

## Phase 13: Validation and benchmarking

### Goal
Prove correctness and identify performance ceilings.

### Validation references
- Desmos for visual sanity checks
- WolframAlpha and MathWorld for known formulas and edge behavior
- Independent analytic cases for exactness tests

### Benchmark targets
- parse-once evaluate-many throughput
- Riemann geometry generation for n up to high interactive limits
- Disk mesh generation at common axial/radial resolutions
- Taylor coefficient generation at target guaranteed degrees

### Suggested thresholds to watch
- 2D updates should feel near-instant for common resolutions
- Repeated parameter changes should avoid full cold-start overhead
- Mesh generation should remain comfortably interactive at standard classroom-quality detail levels

### Exit criteria
- Known-good fixtures pass
- Benchmark results are recorded and reasonable for the selected resolutions

## 8. File Creation Order

The next coding pass should create files in this order to minimize churn:

1. Root manifests and ignores
2. calc-core lib.rs, error.rs, types.rs
3. expr module files
4. eval module files
5. sampling and geometry files
6. riemann files and tests
7. taylor files and tests
8. revolution files and tests
9. calc-wasm files
10. frontend bootstrap files
11. frontend feature modules
12. validation docs and scripts

## 9. Testing Strategy

### Unit tests in calc-core
- Parser tokenization and precedence
- Scalar evaluation
- Domain failure behavior
- Dual-number derivative correctness
- Riemann primitive generation
- Disk mesh topology counts and normals orientation
- Taylor coefficient correctness for reference functions

### Integration tests
- Parse then evaluate same expression across many x values
- Build geometry from realistic UI inputs
- Compare summary metrics against analytic references

### Frontend tests later
- Engine wrapper tests for buffer decoding
- UI state transitions for parse errors and successful updates

## 10. Mathematical Validation Matrix

Use these early fixtures:
- f(x) = x^2 on [0, 1]
- f(x) = sin(x) on [0, pi]
- f(x) = exp(x) on [0, 1]
- f(x) = 2 on [0, 3]
- f(x) = x on [0, 2]
- f(x) = sqrt(x) on [0, 4]
- f(x) = 1 / x on [1, 4] only for safe non-singular interval tests

Known checks:
- Trapezoid rule is exact for linear functions.
- Disk method for constant function over [0, L] yields cylinder volume pi r^2 L.
- Taylor series for exp(x) around 0 has all coefficients 1/k!.
- Taylor series for sin(x) alternates with zero even terms.

## 11. Risk Register

### Parser complexity risk
Mitigation:
- Keep grammar intentionally small first.
- Add precise tests before expanding supported syntax.

### Higher-order derivative complexity risk
Mitigation:
- Guarantee limited degree first.
- Isolate derivative strategy behind a module boundary.

### WASM transfer overhead risk
Mitigation:
- Use packed buffers.
- Reuse allocations.
- Avoid large nested JS objects.

### UI performance risk
Mitigation:
- Keep render math in Rust.
- Use canvas for dense 2D drawing.
- Consider worker offload when integration begins.

### Numerical instability risk
Mitigation:
- Prefer f64 in core.
- Explicitly flag unreliable reference estimates.
- Clamp release-1 feature claims to validated ranges.

## 12. Immediate Next Actions

The next concrete implementation session should do exactly this:

1. Create the root Cargo workspace and crate manifests.
2. Add calc-core error and type skeletons.
3. Implement the AST and parser tests before any UI code.
4. Implement scalar evaluator and sampling.
5. Build Riemann geometry because it is the fastest visible proof that the architecture works.
6. Only after that, start the wasm bridge and frontend shell.

This order is deliberate. It produces a mathematically trustworthy vertical slice with the least rework.
