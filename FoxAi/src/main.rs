
mod data;
mod model;
mod train;


use burn::tensor::Tensor;

use std::fs;
use model::{ModelConfig, FoxAiModel};

fn main() {

    if fs::metadata(data::DATASET_FILE).is_err() {
        data::get_data_set().expect("dataset download failed");
    }

    let ds = data::CharDataset::load(data::DATASET_FILE).unwrap();
    let config = ModelConfig::tiny(ds.vocab_size());

    train::train(&ds, &config);
}