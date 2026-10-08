
mod data;
mod model;


use burn::tensor::Tensor;

use std::fs;
use model::{ModelConfig, FoxAiModel};

fn main() {

    if fs::metadata("dataset").is_ok() {
        println!("Already downloaded");
    } else {
        let _ = data::get_data_set();
    }

    let config = ModelConfig::tiny(65);


    let device = Default::default();

    let model = FoxAiModel::<burn::backend::Wgpu>::new(
        &config,
        &device,
    );

    let output = model.forward(
        Tensor::from_ints(
            [[1, 2, 3, 4, 5, 6, 7, 8]],
            &device,
        )
    );

    println!("Model output shape: {:?}", output.dims());
}