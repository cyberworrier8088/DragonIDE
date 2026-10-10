
mod data;
mod generate;
mod model;
mod train;
mod checkpoint;



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
    let device = Default::default();

    // useage: cargo run --release -- train  ||   cargo run --release -- gen
    let mode = std::env::args().nth(1).unwrap_or_else(|| "gen".to_string());

    if mode == "train" {
        let model = train::train(&ds, &config).valid();
        checkpoint::save(&model).expect("save failed");
        println!("saved to {}.mpk", checkpoint::MODEL_PATH);

    } else {

        let model = checkpoint::load::<Wgpu>(&config, &device).expect("no checkpoints found, run with 'train' first");

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

        println!("{}", text);
    }
}