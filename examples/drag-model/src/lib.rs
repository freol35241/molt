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
