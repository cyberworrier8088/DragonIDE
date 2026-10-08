#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::process::Command;

use burn::prelude::*;
use burn::tensor::TensorData;
use rand::RngExt;

pub const DATASET_DIR: &str = "dataset";
pub const DATASET_FILE: &str = "dataset/test-training-data.txt";

/// download tiny shakesperare
pub fn get_data_set() -> Result<(), String> {

    let url = "https://raw.githubusercontent.com/karpathy/char-rnn/master/data/tinyshakespeare/input.txt";

    println!("Creating folder: {}", DATASET_DIR);

    fs::create_dir_all(DATASET_DIR).map_err(|e| format!("Failed creating dir '{}': {}", DATASET_DIR, e))?;

    println!("downloading...");

    let status = Command::new("curl").arg("-L").arg("-f").arg("-o").arg(DATASET_FILE).arg(url).status().map_err(|e| format!("Could not run curl: {}", e))?;

    if status.success() {
        println!("Downloaded");
        Ok(())
    } else {

        // Remove a half - writern file so the next run download again
        let _ = fs::remove_file(DATASET_FILE);
        Err("Dowload failed".to_string())
    }
}

// charcter-level dataset: evary charcter is on a token

pub struct CharDataset {
    pub chars: Vec<char>,
    stoi: HashMap<char, i32>,

    pub train: Vec<i32>,
    pub val: Vec<i32>,
}

impl CharDataset {

    pub fn load(path: &str) -> Result<Self, String> {

        let text = fs::read_to_string(path).map_err(|e| format!("Coukd not read {}: {}", path, e))?;

        // creaye the vacabulary
        let mut chars: Vec<char> = text.chars().collect();

        chars.sort();
        chars.dedup();

        let mut stoi: HashMap<char, i32> = HashMap::new();

        for (i, &ch) in chars.iter().enumerate() {
            stoi.insert(ch, i as i32);

        }

        // encode the entire dataset
        let data: Vec<i32> = text.chars().map(|ch| stoi[&ch]).collect();
        // 90& train, 10% validation
        let n = (0.9 * data.len() as f64) as usize;

        let train = data[..n].to_vec();
        let val = data[n..].to_vec();

        Ok(
            Self {
                chars,
                stoi,
                train,
                val,
            }
        )
    }

    pub fn vocab_size(&self) -> usize {
        self.chars.len()
    }

    // Text -> tokens
    pub fn encode(&self, text: &str) -> Vec<i32> {
        text.chars().filter_map(|ch| self.stoi.get(&ch).copied()).collect()
    }

    // Tokens -> text
    pub fn decode(&self, tokens: &[i32]) -> String {

        tokens.iter().map(|&t| self.chars.get(t as usize).copied().unwrap_or('?')).collect()
    }
}


pub fn get_batch<B: Backend>(
    data: &[i32],
    batch_size: usize,
    block_size: usize,
    device: &B::Device,
) -> (Tensor<B, 2, Int>, Tensor<B, 2, Int>) {

    assert!(
        data.len() > block_size + 1,
        "dataset is too small for block_size {}",
        block_size
    );

    let mut rng = rand::rng();

    let mut x = Vec::with_capacity(batch_size * block_size);
    let mut y = Vec::with_capacity(batch_size * block_size);

    for _ in 0..batch_size {

        let i = rng.random_range(0..data.len() - block_size);

        x.extend_from_slice(&data[i..i + block_size]);
        y.extend_from_slice(&data[i + 1..i + block_size + 1]);
    }

    let x = Tensor::<B, 2, Int>::from_ints(TensorData::new(x, [batch_size, block_size]), device);

    let y = Tensor::<B, 2, Int>::from_ints(TensorData::new(y, [batch_size, block_size]), device);

    (x, y)

}