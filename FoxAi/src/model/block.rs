use burn::module::Module;
use burn::prelude::*;
use burn::nn::{
    LayerNorm,
    LayerNormConfig,
};

use super::{
    CausalSelfAttention,
    FeedForward,
    ModelConfig,
};

#[derive(Module, Debug)]
pub struct TransformerBlock<B: Backend> {
    pub norm1: LayerNorm<B>,
    pub attention: CausalSelfAttention<B>,
    pub norm2: LayerNorm<B>,
    pub feed_forward: FeedForward<B>,

}

impl<B: Backend> TransformerBlock<B> {

    pub fn new(
        config: &ModelConfig,
        device: &B::Device,
    ) -> Self {

        Self {

            norm1: LayerNormConfig::new(config.d_model).init(device),

            attention: CausalSelfAttention::new(
                config,
                device,
            ),

            norm2: LayerNormConfig::new(config.d_model).init(device),

            feed_forward: FeedForward::new(
                config,
                device,
            ),
        }
    }

    pub fn forward(
        &self,
        x: Tensor<B, 3>,
    ) -> Tensor<B, 3> {

        // Attention Sub-layer
        let normalized = self.norm1.forward(x.clone());

        let attention_output = self.attention.forward(normalized);

        let x = x + attention_output;

        // Feed-Forward sub-layer
        let normalized = self.norm2.forward(x.clone());

        let feed_forward_output = self.feed_forward.forward(normalized);

        x + feed_forward_output
    }
}