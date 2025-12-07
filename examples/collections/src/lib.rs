wit_bindgen::generate!({
    world: "aggregate-model",
});

use exports::example::collections::aggregate::{Guest, GuestModel, Inputs, Outputs, Params};

pub struct AggregateModel {
    params: Params,
}

impl GuestModel for AggregateModel {
    fn new(p: Params) -> Self {
        Self { params: p }
    }

    fn step(&self, i: Inputs) -> Outputs {
        // Sum all values
        let sum: f64 = i.values.iter().sum();

        // Weighted sum (zip truncates to shorter length)
        let weighted_sum: f64 = i
            .values
            .iter()
            .zip(self.params.weights.iter())
            .map(|(v, w)| v * w)
            .sum();

        // Select value by index or use default
        let selected = match i.select_index {
            Some(idx) => i
                .values
                .get(idx as usize)
                .copied()
                .or(self.params.default_value)
                .unwrap_or(0.0),
            None => self.params.default_value.unwrap_or(0.0),
        };

        Outputs {
            sum,
            weighted_sum,
            selected,
            count: i.values.len() as u32,
        }
    }
}

pub struct Aggregate;

impl Guest for Aggregate {
    type Model = AggregateModel;
}

export!(Aggregate);
