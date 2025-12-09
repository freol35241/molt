//! Adder model implementation.
//!
//! This is pure Rust - no wit-bindgen macros needed.
//! molt generates the glue code in molt-gen/lib.rs.

/// A simple model that adds a constant to input values.
pub struct AdderModel {
    addend: f64,
}

impl AdderModel {
    /// Create a new adder model with the given addend.
    pub fn new(addend: f64) -> Result<Self, String> {
        Ok(Self { addend })
    }

    /// Predict the sum of input and addend.
    pub fn predict(&self, value: f64) -> Result<AdderOutputs, String> {
        Ok(AdderOutputs {
            sum: value + self.addend,
        })
    }
}

/// Output from adder prediction.
pub struct AdderOutputs {
    pub sum: f64,
}
