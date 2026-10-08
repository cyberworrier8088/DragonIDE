
mod data;
mod model;


use burn::backend::Wgpu;
use burn::tensor::{Int, Tensor};

use std::fs;
use model::{ModelConfig, FoxAiModel, TransformerBlock};

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

    // attantion temprory test :)
    let block = TransformerBlock::<Wgpu>::new(
        &config,
        &device,
    );

    let block_output = block.forward(embeddings);

    println!(
        "Transformer block output shapeL {:?}",
        block_output.shape()
    );
}