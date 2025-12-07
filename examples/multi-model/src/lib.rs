wit_bindgen::generate!({
    world: "math-models",
});

// Adder model implementation
mod adder_impl {
    use crate::exports::example::math::adder::{Guest, GuestModel, Inputs, Outputs, Params};

    pub struct AdderModel {
        params: Params,
    }

    impl GuestModel for AdderModel {
        fn new(p: Params) -> Self {
            Self { params: p }
        }

        fn step(&self, i: Inputs) -> Outputs {
            Outputs {
                sum: i.value + self.params.addend,
            }
        }
    }

    pub struct Adder;

    impl Guest for Adder {
        type Model = AdderModel;
    }
}

// Multiplier model implementation
mod multiplier_impl {
    use crate::exports::example::math::multiplier::{Guest, GuestModel, Inputs, Outputs, Params};

    pub struct MultiplierModel {
        params: Params,
    }

    impl GuestModel for MultiplierModel {
        fn new(p: Params) -> Self {
            Self { params: p }
        }

        fn step(&self, i: Inputs) -> Outputs {
            Outputs {
                product: i.value * self.params.factor,
            }
        }
    }

    pub struct Multiplier;

    impl Guest for Multiplier {
        type Model = MultiplierModel;
    }
}

// Export both models
export!(adder_impl::Adder, multiplier_impl::Multiplier);
