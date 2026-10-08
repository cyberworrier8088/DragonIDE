use burn::module::Module;
use burn::nn::{Linear, LinearConfig};
use burn::prelude::*;

use super::ModelConfig;

#[derive(Module, Debug)]
pub struct LmHead<B: Backend> {
    pub projection: Linear<B>,
}

impl<B: Backend> LmHead<B> {
    pub fn new(
        config: &ModelConfig,
        device: &B::Device,
    ) -> Self {
        Self {
            projection: LinearConfig::new(
                config.d_model,
                config.vocab_size,
            )
            .init(device),
        }
    }

    pub fn forward(
        &self,
        x: Tensor<B, 3>,
    ) -> Tensor<B, 3> {
        self.projection.forward(x)
    }
}