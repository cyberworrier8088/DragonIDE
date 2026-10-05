use burn::module::Module;
use burn::nn::{Embedding, EmbeddingConfig};
use burn::prelude::*;


use super::ModelConfig;

#[derive(Module, Debug)]
pub struct FoxAiModel<B: Backend> {
    pub token_embedding: Embedding<B>,
}

impl<B: Backend> FoxAiModel<B> {
    pub fn new(config: &ModelConfig, device: &B::Device) -> Self {

        let token_embedding = EmbeddingConfig::new(config.vocab_size, config.d_model).init(device);


        Self {
            token_embedding,
        }
    }


    pub fn forward(
        &self,
        tokens: Tensor<B, 2, Int>,
    ) -> Tensor<B, 3> {
        self.token_embedding.forward(tokens)
    }
}