//! Multiplier model implementation.
//!
//! This is pure Rust - no wit-bindgen macros needed.
//! molt generates the glue code in molt-gen/lib.rs.

/// A simple model that multiplies input values by a constant.
pub struct MultiplierModel {
    factor: f64,
}

impl MultiplierModel {
    /// Create a new multiplier model with the given factor.
    pub fn new(factor: f64) -> Result<Self, String> {
        if factor == 0.0 {
            return Err("factor cannot be zero".to_string());
        }
        Ok(Self { factor })
    }

    /// Predict the product of input and factor.
    pub fn predict(&self, value: f64) -> Result<MultiplierOutputs, String> {
        Ok(MultiplierOutputs {
            product: value * self.factor,
        })
    }
}

/// Output from multiplier prediction.
pub struct MultiplierOutputs {
    pub product: f64,
}
