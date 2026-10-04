
mod data;
mod model;

use std::fs;

fn main() {

    if fs::metadata("dataset").is_ok() {
        println!("Already downloaded");
    } else {
        data::get_data_set();
    }

    data::data_set_preparing();
}