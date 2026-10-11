use burn::prelude::*;
use burn::tensor::TensorData;
use rand::RngExt;

use crate::tokenizer::{EOS, FIM_MIDDLE, FIM_PREFIX, FIM_SUFFIX};


pub fn get_fim_batch<B: Backend>(
    data: &[i32],
    batch_size: usize,
    block_size: usize,
    fim_rate: f32,
    device: &B::Device,
) -> (Tensor<B, 2, Int>, Tensor<B, 2, Int>) {
    let n = block_size + 1; // x and y are shifted by one
    assert!(data.len() > n + 1, "dataset to small");

    let mut rng = rand::rng();
    let mut x = Vec::with_capacity(batch_size * block_size);
    let mut y = Vec::with_capacity(batch_size * block_size);

    for _ in 0..batch_size {
        let i = rng.random_range(0..data.len() - n);
        let raw = &data[i..i + n];

        let seq: Vec<i32> = if rng.random::<f32>() < fim_rate {

            // random total length, so <fim_middle> appears ate evary poss
            let total = rng.random_range(24..=n);
            let body = &raw[..total -1]

            let mid = rng.random_range(1..=48.min(len)); // 1 to 48 tokens, like a real completation
            let a = rng.random_range(0..=len - mid);
            let b = a + mid;

            let mut s = Vec::with_capacity(n);
            s.push(FIM_PREFIX as i32);
            s.extend_from_slice(&body[..a]);
            s.push(FIM_SUFFIX as i32);
            s.extend_from_slice(&body[b..]);
            s.push(FIM_MIDDLE as i32);
            s.extend_from_slice(&body[a..b]);
            s.push(EOS as i32);

            while s.len() < n {
                s.push(PAD as i32); // padding, ignored by the loss
            }
            s
        } else {
            raw.to_vec()
        };

        x.extend_from_slice(&seq[..block_size]);
        y.extend_from_slice(&seq[1..]);

    }

    let x = Tensor::<B, 2, Int>::from_ints(TensorData::new(x, [batch_size, block_size]), device);
    let y = Tensor::<B, 2, Int>::from_ints(TensorData::new(y, [batch_size, block_size]), device);

    (x, y)
}