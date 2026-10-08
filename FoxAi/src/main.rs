
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

    let (query, key, value) = attention.project(embeddings);

    println!("Q shape: {:?}", query.shape());
    println!("K shape: {:?}", key.shape());
    println!("V shape: {:?}", value.shape());
}