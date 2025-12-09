//! Scale model implementation.
//!
//! This is pure Rust - no wit-bindgen macros needed.
//! molt generates the glue code in molt-gen/lib.rs.

/// A simple scaling model that multiplies input by a factor.
pub struct ScaleModel {
    factor: f64,
}

impl ScaleModel {
    /// Create a new scale model with the given factor.
    pub fn new(factor: f64) -> Result<Self, String> {
        if factor.is_nan() {
            return Err("factor cannot be NaN".to_string());
        }
        Ok(Self { factor })
    }

    /// Predict the scaled value.
    pub fn predict(&self, value: f64) -> Result<ScaleOutputs, String> {
        if value.is_nan() {
            return Err("input value cannot be NaN".to_string());
        }
        Ok(ScaleOutputs {
            scaled: value * self.factor,
        })
    }
}

/// Output from scale prediction.
pub struct ScaleOutputs {
    pub scaled: f64,
}
