use burn::module::Module;
use burn::nn::{Linear, LinearConfig};
use burn::prelude::*;
use burn::tensor::activation::softmax;

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

    pub fn split_heads(
        &self,
        x: Tensor<B, 3>,
    ) -> Tensor<B, 4> {

        let [batch_size, sequence_length, _] = x.dims();

        x.reshape([
            batch_size,
            sequence_length,
            self.num_heads,
            self.head_dim,
        ]).swap_dims(1, 2)
    }

    pub fn attention_scores(
        &self,
        query: Tensor<B, 4>,
        key: Tensor<B, 4>,
    ) -> Tensor<B, 4> {

        // [B, H, T, Dh] x [B, H, Dh, T]
        let key_transposed = key.swap_dims(2, 3);

        let scores = query.matmul(key_transposed);

        let scale = (self.head_dim as f32).sqrt();

        scores / scale
    }

    pub fn apply_causal_mask(
        &self,
        scores: Tensor<B, 4>,
    ) -> Tensor<B, 4> {

        let [batch_size, num_heads, sequence_length, _] = scores.dims();

        let device = scores.device();

        let mut mask_data = Vec::with_capacity(sequence_length * sequence_length);

        for i in 0..sequence_length {
            for j in 0..sequence_length {
                if j <= i {
                    mask_data.push(0.0f32);
                } else {
                    mask_data.push(f32::NEG_INFINITY);
                }
            }
        }

        let mask = Tensor::<B, 1>::from_floats(
            mask_data.as_slice(),
            &device,
        ).reshape([sequence_length, sequence_length]).unsqueeze::<3>().unsqueeze::<4>().repeat_dim(0, batch_size).repeat_dim(1, num_heads);

        scores + mask
    }

    pub fn attention_weights(
        &self,
        masked_scores: Tensor<B, 4>,
    ) -> Tensor<B, 4> {
        softmax(masked_scores, 3)
    }

    pub fn weighted_values(
        &self,
        attention_weights: Tensor<B, 4>,
        value: Tensor<B, 4>,
    ) -> Tensor<B, 4> {
        attention_weights.matmul(value)
    }


    pub fn merge_heads(
        &self,
        x: Tensor<B, 4>,
    ) -> Tensor<B, 3> {

        let [batch_size, num_heads, sequence_length, head_dim] = x.dims();

        assert_eq!(num_heads, self.num_heads);
        assert_eq!(head_dim, self.head_dim);

        x.swap_dims(1, 2).reshape(
            [
                batch_size,
                sequence_length,
                self.num_heads * self.head_dim,
            ]
        )
    }

    pub fn output_projection(
        &self,
        x: Tensor<B, 3>,
    ) -> Tensor<B, 3> {
        self.output.forward(x)
    }

    pub fn forward(
        &self,
        input: Tensor<B, 3>,
    ) -> Tensor<B, 3> {


        let (query, key, value) = self.project(input);

        let query = self.split_heads(query);
        let key = self.split_heads(key);
        let value = self.split_heads(value);

        let scores = self.attention_scores(query, key);

        let scores = self.apply_causal_mask(scores);

        let weights = self.attention_weights(scores);

        let output = self.weighted_values(weights, value);

        let output = self.merge_heads(output);

        self.output_projection(output)
    }
}