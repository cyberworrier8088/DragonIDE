use burn::module::Module;
use burn::nn::{Linear, LinearConfig};
use burn::prelude::*;

use super::ModelConfig;

#[derive(Module, Debug)]
pub struct CausalSelfAttention<B: Backend> {
    pub query: Linear<B>,
    pub key: Linear<B>,
    pub value: Linear<B>,
    pub output: Linear<B>,

    pub num_heads: usize,
    pub head_dim: usize,
}


impl<B: Backend> CausalSelfAttention<B> {
    pub fn new(config: &ModelConfig, device: &B::Device) -> Self {

        assert!(config.d_model % config.num_heads == 0, "d_model must be divisible by num_heads");

        let head_dim = config.d_model / config.num_heads;

        Self {
            query: LinearConfig::new(config.d_model, config.d_model).init(device),

            key: LinearConfig::new(config.d_model, config.d_model).init(device),

            value: LinearConfig::new(config.d_model, config.d_model).init(device),

            output: LinearConfig::new(config.d_model, config.d_model).init(device),

            num_heads: config.num_heads,
            head_dim,
        }
    }


    pub fn project(
        &self,
        input: Tensor<B, 3>
    ) -> (Tensor<B, 3>, Tensor<B, 3>, Tensor<B, 3>) {

        let query = self.query.forward(input.clone());
        let key = self.key.forward(input.clone());
        let value = self.value.forward(input);

        (query, key, value)
    }
}