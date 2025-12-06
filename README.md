# molt — Model Once, Load Trivially

A build-time toolkit that lets library maintainers write physical/mathematical models once and distribute them as idiomatic, typed packages for multiple languages.

## Vision

**Maintainer experience:** Write models in Rust, define interfaces in WIT, run `molt build`, get publishable packages for Python/Rust/JS.

**Consumer experience:** `pip install physics-models`, then `from physics_models import create_drag`. No WASM knowledge required.

WASM is used internally as a portable compilation target but is never exposed to maintainers or consumers.

## Installation

```bash
# Install molt
cargo install --path .

# Install cargo-component (required for building WASM)
cargo install cargo-component
```

## Quick Start

### 1. Initialize a new project

```bash
molt init my-models
cd my-models
```

This creates:
```
my-models/
├── molt.toml           # Project manifest
├── Cargo.toml          # Rust package config
├── wit/
│   └── example.wit     # Model interface definition
└── src/
    └── lib.rs          # Model implementation
```

### 2. Define your model interface (WIT)

Edit `wit/example.wit`:

```wit
package myorg:physics;

interface drag {
    record params {
        area: f64,
        cd: f64,
    }

    record inputs {
        rho: f64,
        velocity: f64,
    }

    record outputs {
        drag-force: f64,
        dynamic-pressure: f64,
    }

    resource model {
        constructor(p: params);
        step: func(i: inputs) -> outputs;
    }
}

world drag-model {
    export drag;
}
```

### 3. Implement the model (Rust)

Edit `src/lib.rs`:

```rust
wit_bindgen::generate!({
    world: "drag-model",
    exports: {
        "myorg:physics/drag/model": DragModel,
    },
});

use exports::myorg::physics::drag::{GuestModel, Inputs, Outputs, Params};

pub struct DragModel {
    params: Params,
}

impl GuestModel for DragModel {
    fn new(p: Params) -> Self {
        Self { params: p }
    }

    fn step(&self, i: Inputs) -> Outputs {
        let q = 0.5 * i.rho * i.velocity * i.velocity;
        Outputs {
            drag_force: q * self.params.cd * self.params.area,
            dynamic_pressure: q,
        }
    }
}
```

### 4. Configure the build

Edit `molt.toml`:

```toml
[package]
name = "physics-models"
version = "0.1.0"
description = "Aerodynamic models"

[models]
drag = { wit = "wit/drag.wit", world = "drag-model" }

[targets.python]
package_name = "physics_models"
output_dir = "dist/python"
```

### 5. Build

```bash
molt build
```

This generates a complete Python package in `dist/python/`.

### 6. Use from Python

```python
from physics_models import create_drag

# Build (returns a callable)
drag = create_drag(area=10.0, cd=0.3)

# Use
result = drag(rho=1.225, velocity=20.0)
print(f"Drag force: {result.drag_force} N")

# Compose with solvers
from scipy.integrate import solve_ivp
solve_ivp(lambda t, y: [-drag(rho=1.225, velocity=y[0]).drag_force / mass], ...)
```

## The Functional Pattern

Models follow a closure pattern:

```
builder(params) → model_fn
model_fn(inputs) → outputs
```

The builder captures parameters, returning a function that maps inputs to outputs. This pattern composes naturally with ODE solvers, optimizers, and other numerical tools.

## CLI Commands

```bash
# Build all targets
molt build

# Build specific target
molt build --target python

# Skip WASM compilation (use existing .wasm)
molt build --skip-compile

# Validate without building
molt check

# Initialize new project
molt init my-project
```

## Generated Python Package

The generated Python package includes:

```
physics_models/
├── pyproject.toml
└── physics_models/
    ├── __init__.py          # Exports create_drag, etc.
    ├── _runtime.py          # Internal: wasmtime wrapper
    ├── _wasm.py             # Internal: embedded WASM bytes
    ├── drag.py              # Public: DragParams, DragOutputs, create_drag
    └── py.typed             # PEP 561 marker
```

Features:
- Full type hints with dataclasses
- IDE autocompletion works
- `mypy` compatible
- Single runtime dependency (`wasmtime`)

## Supported Types

| WIT Type | Python Type |
|----------|-------------|
| `f64`, `f32` | `float` |
| `u8`, `u16`, `u32`, `u64` | `int` |
| `s8`, `s16`, `s32`, `s64` | `int` |
| `string` | `str` |
| `bool` | `bool` |
| `list<T>` | `list[T]` |
| `option<T>` | `T \| None` |

## Requirements

- Rust 1.70+
- `cargo-component` (for WASM compilation)
- Python 3.10+ (for generated packages)

## Project Status

This is an MVP implementation. Current features:

- [x] Parse `molt.toml` manifest
- [x] Parse WIT interfaces
- [x] Compile to WASM via cargo-component
- [x] Generate Python packages with type hints
- [x] Embed WASM bytes in generated code

Planned:
- [ ] Rust consumer target
- [ ] JavaScript/npm target
- [ ] Multiple models per package
- [ ] Batched interface for performance
- [ ] Python model sources (via componentize-py)

## License

Apache-2.0
