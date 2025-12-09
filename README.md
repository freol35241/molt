# molt — Model Once, Load Trivially

A build-time toolkit that lets library maintainers write physical/mathematical models once and distribute them as idiomatic, typed packages for multiple languages.

## Vision

**Maintainer experience:** Write models in pure Rust, define interfaces in WIT, run `molt build`, get publishable packages for Python/Rust/JS.

**Consumer experience:** `pip install physics-models`, then `from physics_models import DragModel`. No WASM knowledge required.

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
├── molt.toml           # MOLT project manifest
├── Cargo.toml          # Rust package config (points to molt-gen/lib.rs)
├── wit/
│   └── example.wit     # Model interface definition
├── src/
│   └── example.rs      # Pure Rust model implementation
└── molt-gen/           # Generated glue code (git-ignored)
    └── lib.rs
```

### 2. Define your model interface (WIT)

Edit `wit/drag.wit`:

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
        predict: func(i: inputs) -> result<outputs, string>;
    }
}

world drag-model {
    export drag;
}
```

### 3. Implement the model (Pure Rust)

Write your model in `src/drag.rs` - **no wit-bindgen macros needed!**

```rust
//! Drag model implementation.
//! This is pure Rust - molt generates the glue code.

pub struct DragModel {
    area: f64,
    cd: f64,
}

impl DragModel {
    /// Create a new drag model.
    pub fn new(area: f64, cd: f64) -> Result<Self, String> {
        if area <= 0.0 {
            return Err("area must be positive".to_string());
        }
        Ok(Self { area, cd })
    }

    /// Compute drag force and dynamic pressure.
    pub fn predict(&self, rho: f64, velocity: f64) -> Result<DragOutputs, String> {
        let q = 0.5 * rho * velocity * velocity;
        Ok(DragOutputs {
            drag_force: q * self.cd * self.area,
            dynamic_pressure: q,
        })
    }
}

pub struct DragOutputs {
    pub drag_force: f64,
    pub dynamic_pressure: f64,
}
```

### 4. Configure the build

Edit `molt.toml`:

```toml
[package]
name = "physics-models"
version = "0.1.0"
description = "Aerodynamic models"

[models.drag]
wit = "wit/drag.wit"
world = "drag-model"
source = "src/drag.rs"
struct = "DragModel"
outputs = "DragOutputs"

[targets.python]
package_name = "physics_models"
output_dir = "dist/python"
```

### 5. Build

```bash
molt build
```

This runs three steps:
1. **Generate** - Creates `molt-gen/lib.rs` with WIT bindings and adapter code
2. **Compile** - Builds WASM component via `cargo component`
3. **Pack** - Generates target language packages

You can also run these steps separately:
```bash
molt generate    # Only generate glue code
molt pack        # Only generate packages (requires existing WASM)
```

### 6. Use from Python

```python
from physics_models import DragModel

# Create model with parameters
model = DragModel(area=10.0, cd=0.3)

# Run prediction
result = model.predict(rho=1.225, velocity=20.0)
print(f"Drag force: {result.drag_force} N")
print(f"Dynamic pressure: {result.dynamic_pressure} Pa")

# Use with scipy
from scipy.integrate import solve_ivp

def dynamics(t, state):
    v = state[0]
    result = model.predict(rho=1.225, velocity=v)
    return [-result.drag_force / mass]

solve_ivp(dynamics, [0, 10], [100.0])
```

## The Class-Based Pattern

Models follow a class-based pattern with a `predict` method:

```python
model = ModelClass(param1, param2, ...)  # Construction
result = model.predict(input1, input2, ...)  # Prediction
```

This pattern:
- Separates one-time parameters from per-call inputs
- Composes naturally with ODE solvers and optimizers
- Provides clear error handling via exceptions

## CLI Commands

```bash
# Full build (generate + compile + pack)
molt build

# Build specific target only
molt build --target python

# Skip WASM compilation (use existing .wasm)
molt build --skip-compile

# Generate glue code only
molt generate

# Generate packages from existing WASM
molt pack

# Validate without building
molt check

# Initialize new project
molt init my-project
```

## Project Structure

After `molt build`, you get:

```
my-models/
├── molt.toml
├── Cargo.toml
├── wit/
│   └── drag.wit
├── src/
│   └── drag.rs           # Your pure Rust implementation
├── molt-gen/
│   └── lib.rs            # Generated glue code (git-ignored)
├── target/
│   └── wasm32-wasip1/
│       └── release/
│           └── *.wasm    # Compiled WASM
└── dist/
    └── python/
        ├── pyproject.toml
        ├── README.md
        └── physics_models/
            ├── __init__.py
            ├── drag.py       # DragModel, DragOutputs
            ├── _runtime.py
            ├── _wasm.py
            └── py.typed
```

## Generated Python Package Features

- **Class-based API** with `predict()` method
- **Full type hints** with dataclasses
- **IDE autocompletion** works out of the box
- **mypy compatible**
- **Single runtime dependency** (`wasmtime>=21.0.0`)
- **Error handling** via custom exception classes (e.g., `DragModelError`)

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
| `result<T, string>` | Returns `T`, raises exception on error |

## Requirements

- Rust 1.70+
- `cargo-component` (for WASM compilation)
- Python 3.10+ (for generated packages)

## Examples

See the `examples/` directory for complete working examples:

- **minimal** - Simple scaling model
- **primitives** - All WIT primitive types
- **collections** - Lists and options
- **multi-model** - Multiple models in one package

## Project Status

Current features:
- [x] Parse `molt.toml` manifest
- [x] Parse WIT interfaces with `predict` method and error handling
- [x] Generate glue code (`molt-gen/lib.rs`) - users write pure Rust
- [x] Compile to WASM via cargo-component
- [x] Generate Python packages with class-based API
- [x] Multiple models per package

Planned:
- [ ] Rust consumer target
- [ ] JavaScript/npm target
- [ ] Python model sources (via componentize-py)
- [ ] Batched interface for performance

## License

Apache-2.0
