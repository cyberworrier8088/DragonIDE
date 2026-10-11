use burn::prelude::*;
use burn::tensor::TensorData;
use rand::RngExt;

use crate::tokenizer::{EOS, FIM_MIDDLE, FIM_PREFIX, FIM_SUFFIX, PAD};


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

            // random total length, so <fim_middle> appears at various positions
            let total = rng.random_range(24..=n);
            let body = &raw[..total - 4];
            let len = body.len().max(1);

            // Realistic code completion distribution:
            // 60% short inline (1..16 tokens) for line/arg autocompletion
            // 40% block level (16..48 tokens) for loop/fn body completion
            let max_mid = if rng.random::<f32>() < 0.6 {
                16.min(len)
            } else {
                48.min(len)
            }.max(1);

            let mid = rng.random_range(1..=max_mid);
            let a = rng.random_range(0..=len - mid);
            let b = a + mid;

            let mut s = Vec::with_capacity(n);
            if rng.random::<f32>() < 0.5 {
                // PSM (Prefix-Suffix-Middle) mode - canonical IDE completion
                s.push(FIM_PREFIX as i32);
                s.extend_from_slice(&body[..a]);
                s.push(FIM_SUFFIX as i32);
                s.extend_from_slice(&body[b..]);
                s.push(FIM_MIDDLE as i32);
                s.extend_from_slice(&body[a..b]);
                s.push(EOS as i32);
            } else {
                // SPM (Suffix-Prefix-Middle) mode - improves bidirectional attention
                s.push(FIM_SUFFIX as i32);
                s.extend_from_slice(&body[b..]);
                s.push(FIM_PREFIX as i32);
                s.extend_from_slice(&body[..a]);
                s.push(FIM_MIDDLE as i32);
                s.extend_from_slice(&body[a..b]);
                s.push(EOS as i32);
            }

            s.resize(n, PAD as i32);
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