//! Aggregate model implementation - demonstrates list and option types.
//!
//! This is pure Rust - no wit-bindgen macros needed.
//! molt generates the glue code in molt-gen/lib.rs.

/// Model demonstrating list and option types.
pub struct AggregateModel {
    weights: Vec<f64>,
    default_value: Option<f64>,
}

impl AggregateModel {
    /// Create a new aggregate model with the given parameters.
    pub fn new(weights: Vec<f64>, default_value: Option<f64>) -> Result<Self, String> {
        Ok(Self {
            weights,
            default_value,
        })
    }

    /// Predict with aggregation operations.
    pub fn predict(
        &self,
        values: Vec<f64>,
        select_index: Option<u32>,
    ) -> Result<AggregateOutputs, String> {
        // Sum all values
        let sum: f64 = values.iter().sum();

        // Weighted sum (zip truncates to shorter length)
        let weighted_sum: f64 = values
            .iter()
            .zip(self.weights.iter())
            .map(|(v, w)| v * w)
            .sum();

        // Select value by index or use default
        let selected = match select_index {
            Some(idx) => values
                .get(idx as usize)
                .copied()
                .or(self.default_value)
                .unwrap_or(0.0),
            None => self.default_value.unwrap_or(0.0),
        };

        Ok(AggregateOutputs {
            sum,
            weighted_sum,
            selected,
            count: values.len() as u32,
        })
    }
}

/// Output from aggregate prediction.
pub struct AggregateOutputs {
    pub sum: f64,
    pub weighted_sum: f64,
    pub selected: f64,
    pub count: u32,
}
