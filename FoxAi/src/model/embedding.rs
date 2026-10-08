use burn::module::Module;
use burn::nn::{Embedding, EmbeddingConfig};
use burn::prelude::*;

use super::ModelConfig;

#[derive(Module, Debug)]
pub struct TokenAndPositionEmbeddings<B: Backend> {
    pub token_embedding: Embedding<B>,
    pub position_embedding: Embedding<B>,
}

impl<B: Backend> TokenAndPositionEmbeddings<B> {
    pub fn new(config: &ModelConfig, device: &B::Device) -> Self {

        let token_embedding = EmbeddingConfig::new(config.vocab_size, config.d_model).init(device);

        let position_embedding = EmbeddingConfig::new(config.context_length, config.d_model).init(device);

        Self {
            token_embedding,
            position_embedding,
        }
    }


    pub fn forward(
        &self,
        tokens: Tensor<B, 2, Int>,
    ) -> Tensor<B, 3> {

        let [batch_size, sequence_length] = tokens.dims();

        let token_embeddings = self.token_embedding.forward(tokens);

        let positions: Vec<i64> = (0..sequence_length).map(|position| position as i64).collect();

        let positions = Tensor::<B, 1, Int>::from_ints(
            positions.as_slice(),
            &token_embeddings.device(),
        );

        let position_embeddings = self.position_embedding.forward(positions);

        let position_embeddings = position_embeddings.unsqueeze::<3>().repeat_dim(0, batch_size);

        token_embeddings + position_embeddings
    }
}