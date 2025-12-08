//! Types model implementation - demonstrates WIT primitive types.
//!
//! This is pure Rust - no wit-bindgen macros needed.
//! molt generates the glue code in molt-gen/lib.rs.

/// Model demonstrating various WIT primitive types.
pub struct TypesModel {
    scale_f64: f64,
    offset_f32: f32,
    multiplier_s32: i32,
    enabled: bool,
}

impl TypesModel {
    /// Create a new types model with the given parameters.
    pub fn new(
        scale_f64: f64,
        offset_f32: f32,
        multiplier_s32: i32,
        enabled: bool,
    ) -> Result<Self, String> {
        Ok(Self {
            scale_f64,
            offset_f32,
            multiplier_s32,
            enabled,
        })
    }

    /// Predict with various type transformations.
    pub fn predict(
        &self,
        value_f64: f64,
        value_f32: f32,
        counter_u32: u32,
        id_u64: u64,
        adjustment_s8: i8,
        condition: bool,
    ) -> Result<TypesOutputs, String> {
        let was_processed = self.enabled && condition;

        Ok(TypesOutputs {
            scaled_value: value_f64 * self.scale_f64,
            offset_value: value_f32 + self.offset_f32,
            multiplied_counter: (counter_u32 as i64) * (self.multiplier_s32 as i64),
            processed_id: if self.enabled { id_u64 } else { 0 },
            combined: (adjustment_s8 as i32) + (condition as i32),
            was_processed,
        })
    }
}

/// Output from types prediction.
pub struct TypesOutputs {
    pub scaled_value: f64,
    pub offset_value: f32,
    pub multiplied_counter: i64,
    pub processed_id: u64,
    pub combined: i32,
    pub was_processed: bool,
}
