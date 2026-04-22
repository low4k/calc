# Math Validation Plan

## Purpose
This document defines how to verify that the engine is mathematically correct before trusting the UI.

## Validation Principles

1. Prefer analytic references when available.
2. Use external tools only as cross-checks, not as hidden runtime dependencies.
3. Validate both summary metrics and geometry behavior.
4. Treat singularities and invalid domains as explicit test cases.

## External reference tools
- Desmos for visual sanity checks
- WolframAlpha for exact results and derivative comparison
- MathWorld for convergence and edge-case definitions
- Paul's Online Math Notes for disk-method and related calculus examples

## Core function fixture set

### Polynomial fixtures
- x
- x^2
- x^3 - 2x + 1

Use for:
- parser precedence checks
- evaluator checks
- exact or near-exact Riemann expectations in simple cases
- Taylor checks around 0 or 1

### Trigonometric fixtures
- sin(x)
- cos(x)

Use for:
- oscillation behavior
- midpoint and trapezoid convergence comparisons
- Taylor convergence around 0

### Exponential/log fixtures
- exp(x)
- ln(1 + x)

Use for:
- Taylor coefficient validation
- domain handling
- convergence interval behavior

### Root and reciprocal fixtures
- sqrt(x)
- 1 / x

Use for:
- domain boundary handling
- sampling discontinuity behavior
- error-visualization caution flags

## Riemann validation cases

### Constant function
Function: 2
Interval: [0, 3]
Expected integral: 6
Expected behavior:
- all rules produce exact result
- rectangles all same height
- trapezoids degenerate to rectangles visually

### Linear function
Function: x
Interval: [0, 2]
Expected integral: 2
Expected behavior:
- trapezoid rule exact
- left underestimates
- right overestimates

### Quadratic function
Function: x^2
Interval: [0, 1]
Expected integral: 1/3
Expected behavior:
- midpoint and trapezoid converge faster than left/right on visual error chart

### Sinusoidal function
Function: sin(x)
Interval: [0, pi]
Expected integral: 2
Expected behavior:
- all methods converge as n increases
- midpoint should perform strongly for smooth oscillatory function

## Disk-method validation cases

### Cylinder
Function: 2
Interval: [0, 3]
Expected volume: 12pi
Expected geometry:
- constant radius rings
- straight side wall
- optional circular end caps

### Cone-like profile
Function: x
Interval: [0, 2]
Expected volume: 8pi/3
Expected geometry:
- radius grows linearly with x
- one end cap radius 0

## Taylor validation cases

### Exponential series
Function: exp(x)
Center: 0
Expected coefficients: 1/k!
Expected behavior:
- low-order approximations already good near zero
- error increases away from center

### Sine series
Function: sin(x)
Center: 0
Expected pattern:
- odd powers only
- alternating signs

### Log series
Function: ln(1 + x)
Center: 0
Expected warning:
- convergence only reliable on a bounded interval around the center

## Tolerance policy

Recommended default tolerances during early development:
- scalar evaluation: 1e-12 for simple analytic comparisons in f64
- numerical integrals and Taylor approximations: feature-specific tolerances based on method and sample count
- mesh geometry: exact topology counts, approximate floating-point position tolerance

## Failure handling
If a reference comparison fails:
1. Determine whether the parser, evaluator, derivative logic, or geometry layer is responsible.
2. Reproduce with the smallest possible fixture.
3. Add that fixture as a regression test before changing code.
