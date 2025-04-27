# Apartment-Sim

Procedurally generated, fully interactive 2D apartment simulator in Rust, with WebAssembly and Python bindings for reinforcement-learning research, in-browser demos, and headless training.

## Features

- **Core engine**: ECS, continuous geometry, spatial hash, procedural apartment generator
- **WASM wrapper**: wasm-bindgen shim exposing Rust API to JS/TS
- **Python FFI**: PyO3/maturin extension with Gymnasium-style API
- **Examples**: Vue 3 + Pixi demo, Python headless Gym wrapper

## Repository Structure

```
apartment_sim/
├── Cargo.toml           # workspace definition
├── plan.md              # implementation guide and roadmap
├── crates/
│   ├── core/            # engine: ECS, geometry, generator, simulation
│   ├── wasm/            # WASM wrapper via wasm-bindgen
│   └── ffi_py/          # Python bindings via PyO3
├── examples/
│   ├── vue_viewer/      # browser demo (Vue 3 + Pixi)
│   └── python_headless/ # headless Gym wrapper using wasmtime-py
└── target/              # build artifacts
```

## Prerequisites

- Rust (edition 2021)
- Node.js & npm (for `vue_viewer` demo)
- [`wasm-pack`](https://github.com/rustwasm/wasm-pack) (optional for WASM builds)
- Python ≥3.7 & [`maturin`](https://github.com/PyO3/maturin) (for Python bindings)

## Building & Testing

```bash
# Build all crates
cargo build --workspace

# Run all tests
a cargo test --workspace
```

## Core Usage (Rust)

```rust
use core::{GenOpts, generate, World};

let opts = GenOpts::default();
let mut world = generate(&opts);
world.step(0.1);
```

## WASM Demo

```bash
cd crates/wasm
wasm-pack build --target web
# Serve `examples/vue_viewer` with a static HTTP server
```

## Python FFI

```bash
cd crates/ffi_py
pip install maturin
maturin develop
```

```python
import ffi_py
opts = ffi_py.PyGenOpts(42, 25.0, 3, 0.2, 0.1)
world = ffi_py.PyWorld.generate(opts)
world.step(0.1)
rooms = world.rooms
```

## Examples

- **Vue Viewer**: interactive browser demo with floorplan rendering
- **Python Headless**: Gymnasium-compatible RL environment wrapper

## Contributing

Contributions welcome! See `plan.md` for architecture and implementation details.

## License

MIT
