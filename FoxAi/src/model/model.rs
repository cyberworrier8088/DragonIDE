use burn::module::Module;
use burn::prelude::*;

use super::ModelConfig;
use super::embedding::TokenAndPositionEmbeddings;

#[derive(Module, Debug)]
pub struct FoxAiModel<B: Backend> {
    pub embedding: TokenAndPositionEmbeddings<B>,
}

impl<B: Backend> FoxAiModel<B> {
    pub fn new(config: &ModelConfig, device: &B::Device) -> Self {
        Self {
            embedding: TokenAndPositionEmbeddings::new(config, device),
        }
    }

    pub fn forward(
        &self,
        tokens: Tensor<B, 2, Int>,
    ) -> Tensor<B, 3> {
        self.embedding.forward(tokens)
    }
}