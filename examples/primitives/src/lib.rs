wit_bindgen::generate!({
    world: "types-model",
});

use exports::example::primitives::types::{Guest, GuestModel, Inputs, Outputs, Params};

pub struct TypesModel {
    params: Params,
}

impl GuestModel for TypesModel {
    fn new(p: Params) -> Self {
        Self { params: p }
    }

    fn step(&self, i: Inputs) -> Outputs {
        let was_processed = self.params.enabled && i.condition;

        Outputs {
            scaled_value: i.value_f64 * self.params.scale_f64,
            offset_value: i.value_f32 + self.params.offset_f32,
            multiplied_counter: (i.counter_u32 as i64) * (self.params.multiplier_s32 as i64),
            processed_id: if self.params.enabled { i.id_u64 } else { 0 },
            combined: (i.adjustment_s8 as i32) + (i.condition as i32),
            was_processed,
        }
    }
}

pub struct Types;

impl Guest for Types {
    type Model = TypesModel;
}

export!(Types);
