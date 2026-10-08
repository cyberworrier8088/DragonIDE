
mod data;
mod model;


use burn::backend::Wgpu;
use burn::tensor::{Int, Tensor};

use std::fs;
use model::{ModelConfig, FoxAiModel};
use model::CausalSelfAttention;

fn main() {

    if fs::metadata("dataset").is_ok() {
        println!("Already downloaded");
    } else {
        data::get_data_set();
    }

    let config = ModelConfig::tiny(65);


    let device = Default::default();

    let model = FoxAiModel::<burn::backend::Wgpu>::new(
        &config,
        &device,
    );

    let tokens: Tensor<Wgpu, 2, Int> = Tensor::from_ints(
        [[1, 2, 3, 4, 5, 6, 7, 8]],
        &device,
    );

    let embeddings = model.forward(tokens);

    println!("Embeddings:");
    println!("{:#?}", embeddings);

    let attention = CausalSelfAttention::<Wgpu>::new(&config, &device);

    let (q, k, v) = attention.project(embeddings.clone());

    let q_heads = attention.split_heads(q);
    let k_heads = attention.split_heads(k);
    let v_heads = attention.split_heads(v);

    println!("Q_heads shape: {:?}", q_heads.shape());
    println!("K_heads shape: {:?}", k_heads.shape());
    println!("V_heads shape: {:?}", v_heads.shape());

    let scores = attention.attention_scores(
        q_heads,
        k_heads,
    );

    println!("attention scores shape: {:?}", scores.shape());

    let masked_scores = attention.apply_causal_mask(scores);

    println!("Masked attention score shape: {:?}", masked_scores.shape());

}