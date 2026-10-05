
mod data;
mod model;

use std::fs;
use model::ModelConfig;

fn main() {

    if fs::metadata("dataset").is_ok() {
        println!("Already downloaded");
    } else {
        data::get_data_set();
    }

    let config = ModelConfig::tiny(65);


    println!("FoxAi configuration:");
    println!("{:#?}", config);
}