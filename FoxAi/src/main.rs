
mod data;
mod generate;
mod model;
mod train;



use burn::backend::Wgpu;
use burn::module::AutodiffModule;
use model::ModelConfig;
use std::fs;


fn main() {

    if fs::metadata(data::DATASET_FILE).is_err() {
        data::get_data_set().expect("dataset download failed");
    }

    let ds = data::CharDataset::load(data::DATASET_FILE).unwrap();
    let config = ModelConfig::tiny(ds.vocab_size());

    // train, then drop the autodiff part <not needed for generating>
    let model = train::train(&ds, &config).valid();

    let device = Default::default();
    let text = generate::generate::<Wgpu>(
        &model,
        &ds,
        "ROMEO:",
        300,
        config.context_length,
        0.8,
        10,
        &device,
    );

    println!("\n----- generated text -----\n{}", text);
}