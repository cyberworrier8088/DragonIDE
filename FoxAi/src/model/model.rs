use burn::module::Module;
use burn::nn::{LayerNorm, LayerNormConfig};
use burn::prelude::*;

use super::{
    embedding::TokenAndPositionEmbeddings,
    block::TransformerBlock,
    lm_head::LmHead,
    ModelConfig,
};


#[derive(Module, Debug)]
pub struct FoxAiModel<B: Backend> {
    pub embedding: TokenAndPositionEmbeddings<B>,
    pub blocks: Vec<TransformerBlock<B>>,
    pub final_norm: LayerNorm<B>,
    pub lm_head: LmHead<B>,
}

impl<B: Backend> FoxAiModel<B> {
    pub fn new(
        config: &ModelConfig,
        device: &B::Device,
    ) -> Self {


        let embedding = TokenAndPositionEmbeddings::new(config, device);

        let mut blocks = Vec::with_capacity(config.num_layers);

        for _ in 0..config.num_layers {
            blocks.push(
                TransformerBlock::new(config, device)
            );
        }

        Self {
            embedding,
            blocks,
            final_norm: LayerNormConfig::new(config.d_model).init(device),
            lm_head: LmHead::new(config, device),
        }
    }

    pub fn forward(
        &self,
        tokens: Tensor<B, 2, Int>,
    ) -> Tensor<B, 3> {

        let mut x = self.embedding.forward(tokens);

        for block in &self.blocks {
            x = block.forward(x);
        }

        x = self.final_norm.forward(x);
        self.lm_head.forward(x)
    }
}