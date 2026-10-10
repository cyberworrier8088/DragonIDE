use burn::prelude::*;
use burn::tensor::TensorData;
use rand::RngExt;

use crate::tokenizer::{Tokenizer, EOS};
use crate::model::FoxAiModel;

pub fn generate<B: Backend>(
    model: &FoxAiModel<B>,
    tok: &Tokenizer,
    prompt: &str,
    max_new_tokens: usize,
    context_length: usize,
    temperature: f32,
    top_k: usize,
    device: &B::Device,
) -> String {

    let mut tokens: Vec<i32> = tok.encode(prompt).into_iter().map(|t| t as i32).collect();

    if tokens.is_empty() {
        tokens.push(EOS as i32);
    }

    let mut rng = rand::rng();

    for _ in 0..max_new_tokens {
        // the model only see the last 'Context_length' tokens
        let start = tokens.len().saturating_sub(context_length);
        let window = tokens[start..].to_vec();
        let len = window.len();

        let input = Tensor::<B, 2, Int>::from_ints(TensorData::new(window, [1, len]), device);

        let logits = model.forward(input);
        let [_, _, vocab] = logits.dims();

        // logits of the LAST position only
        let last = logits.slice([0..1, len - 1..len, 0..vocab]).reshape([vocab]);

        let mut scores: Vec<f32> = last.into_data().to_vec::<f32>().unwrap();

        // temperature: lower = safer, higher = more random
        for s in scores.iter_mut() {
            *s /= temperature;
        }

        // top-k: keep only the l best candidates
        let mut idx: Vec<usize> = (0..vocab).collect();
        idx.sort_by(|&a, &b|  scores[b].partial_cmp(&scores[a]).unwrap());
        idx.truncate(top_k.min(vocab));

        // softmax over the kept candidates
        let max = scores[idx[0]];
        let exps: Vec<f32> = idx.iter().map(|&i| (scores[i] - max).exp()).collect();
        let sum: f32 = exps.iter().sum();

        // sample
        let mut r = rng.random::<f32>() * sum;
        let mut chosen = idx[0];
        for (k, &i) in idx.iter().enumerate() {
            r -= exps[k];
            if r <= 0.0 {
                chosen = i;
                break;
            }
        }

        tokens.push(chosen as i32);

        if chosen as u32 == EOS {
            break; // the model decided the code is finished
        }
    }

    let ids: Vec<u32> = tokens.iter().map(|&t| t as u32).collect();
    tok.decode(&ids)
}