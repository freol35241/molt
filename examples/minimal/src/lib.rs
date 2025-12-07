wit_bindgen::generate!({
    world: "scale-model",
});

use exports::example::minimal::scale::{Guest, GuestModel, Inputs, Outputs, Params};

pub struct ScaleModel {
    params: Params,
}

impl GuestModel for ScaleModel {
    fn new(p: Params) -> Self {
        Self { params: p }
    }

    fn step(&self, i: Inputs) -> Outputs {
        Outputs {
            scaled: i.value * self.params.factor,
        }
    }
}

pub struct Scale;

impl Guest for Scale {
    type Model = ScaleModel;
}

export!(Scale);
