# WASM Calculus Engine

This repository is being scaffolded as a Rust-first calculus visualization engine with a TypeScript frontend.

Current status:
- Repository folders created
- Architecture baseline documented
- Detailed implementation plan documented

Primary planning docs:
- docs/implementation-plan.md
- docs/folder-structure.md
- docs/adr/0001-architecture-baseline.md

Planned top-level structure:
- crates/calc-core: pure Rust math engine
- crates/calc-wasm: wasm-bindgen bridge
- frontend: Vite + TypeScript UI and renderers
- docs: architecture, validation, and milestone docs
- scripts: local build and packaging helpers
- examples: future sample functions and demo presets

Immediate next implementation order:
1. Create the Rust workspace manifests
2. Implement shared engine types and error model
3. Build the expression parser and evaluator
4. Add Riemann geometry generation and tests
5. Add WASM buffer exports and frontend integration
