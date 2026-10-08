use burn::module::Module;
use burn::nn::{Linear, LinearConfig};
use burn::prelude::*;
use burn::tensor::activation::gelu;

use super::ModelConfig;

#[derive(Module, Debug)]
pub struct FeedForward<B: Backend> {
    pub input: Linear<B>,
    pub output: Linear<B>,
}

impl<B: Backend> FeedForward<B> {

    pub fn new(
        config: &ModelConfig,
        device: &B::Device,
    ) -> Self {
        let hidden_dim = config.d_model * 4;

        Self {
            input: LinearConfig::new(
                config.d_model,
                hidden_dim,
            ).init(device),

            output: LinearConfig::new(
                hidden_dim,
                config.d_model,
            ).init(device),
        }
    }

    pub fn forward(
        &self,
        x: Tensor<B, 3>,
    ) -> Tensor<B, 3> {

        let x = self.input.forward(x);
        let x = gelu(x);
        self.output.forward(x)
    }
}