# Calc

Calc is a Rust-first calculus and graphing playground with a WebAssembly bridge and a TypeScript frontend.

The Rust side owns parsing, evaluation, sampling, Taylor series generation, Riemann geometry, and disk-method mesh generation. The frontend focuses on interaction and visualization while delegating math-heavy work to the WASM engine.

## Features

- Graphing calculator UI with live expression parsing
- Explicit curve sampling and relation-grid generation
- Riemann sum geometry with approximation and error summaries
- Taylor series coefficients plus sampled comparison curves
- Disk-method revolution meshes with bounds and metadata
- Error-series visualization for convergence inspection
- Rust tests covering parser, evaluator, graphing, Riemann, Taylor, revolution, and WASM packing helpers

## Stack

- Rust workspace for the core math engine and WASM wrapper
- wasm-bindgen for the browser bridge
- Vite + TypeScript frontend
- MathLive input and Three.js-based 3D rendering in the UI

## Quick Start

### Prerequisites

- Rust toolchain
- wasm-pack
- Node.js and npm

### Run locally

The fastest path is the helper script from the repository root:

```bash
./run.sh
```

That script:

- builds the WASM package into `frontend/public/pkg`
- installs frontend dependencies if needed
- starts the Vite dev server
- opens the app in your browser when possible

### Manual workflow

```bash
cd crates/calc-wasm
wasm-pack build --target web --out-dir ../../frontend/public/pkg

cd ../../frontend
npm install
npm run dev
```

## Build And Test

Run the Rust test suites from the repository root:

```bash
cargo test
```

Build the frontend:

```bash
cd frontend
npm run build
```

If you rebuild the frontend from scratch, regenerate the WASM package first:

```bash
cd crates/calc-wasm
wasm-pack build --target web --out-dir ../../frontend/public/pkg
```

## Status

This repository is already beyond the initial scaffold stage. The core Rust engine, the WASM bridge, and the interactive frontend are all present and wired together.
