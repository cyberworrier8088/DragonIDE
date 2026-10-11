use burn::prelude::*;
use burn::tensor::TensorData;
use rand::RngExt;

use crate::model::FoxAiModel;
use crate::tokenizer::{Tokenizer, EOS, FIM_MIDDLE, FIM_PREFIX, FIM_SUFFIX};


pub fn generate_ids<B: Backend>(
    model: &FoxAiModel<B>,
    mut tokens: Vec<i32>,
    max_new_tokens: usize,
    context_length: usize,
    temperature: f32,
    top_k: usize,
    device: &B::Device,
) -> Vec<i32> {

    let mut new_tokens = Vec::new();
    let mut rng = rand::rng();

    for _ in 0..max_new_tokens {

        let start = tokens.len().saturating_sub(context_length);
        let window = tokens[start..].to_vec();
        let len = window.len();

        let input = Tensor::<B, 2, Int>::from_ints(TensorData::new(window, [1, len]), device);
        let logits = model.forward(input);
        let [_, _, vocab] = logits.dims();

        let last = logits.slice([0..1, len -1..len, 0..vocab]).reshape([vocab]);
        let mut scores: Vec<f32> = last.into_data().to_vec::<f32>().unwrap();

        for s in scores.iter_mut() {
            *s /= temperature;
        }

        let mut idx: Vec<usize> = (0..vocab).collect();
        idx.sort_by(|&a, &b| scores[b].partial_cmp(&scores[a]).unwrap());
        idx.truncate(top_k.min(vocab));

        let max = scores[idx[0]];
        let exps: Vec<f32> = idx.iter().map(|&i| (scores[i] - max).exp()).collect();
        let sum: f32 = exps.iter().sum();

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
        new_tokens.push(chosen as i32);

        if chosen as u32 == EOS {
            break;
        }
    }

    new_tokens
}

// plain left to right genertaion (prompt + continuation)
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

    let start_time = std::time::Instant::now();
    let new = generate_ids(model, tokens.clone(), max_new_tokens, context_length, temperature, top_k, device);
    let elapsed = start_time.elapsed().as_secs_f64();
    let count = new.len();
    let tok_per_sec = count as f64 / elapsed.max(1e-4);
    eprintln!("[Generate] {} tokens in {:.3}s ({:.1} tok/s)", count, elapsed, tok_per_sec);

    tokens.extend(new);

    let ids: Vec<u32> = tokens.iter().map(|&t| t as u32).collect();
    tok.decode(&ids)
}


// autocomplete: code before the cursor + code after it -> text for the cursor
pub fn complete<B: Backend>(

    model: &FoxAiModel<B>,
    tok: &Tokenizer,
    prefix: &str,
    suffix: &str,
    max_new_tokens: usize,
    context_length: usize,
    temperature: f32,
    top_k: usize,
    device: &B::Device,
) -> String {

    let mut ids: Vec<i32> = vec![FIM_PREFIX as i32];
    ids.extend(tok.encode(prefix).into_iter().map(|t| t as i32));
    ids.push(FIM_SUFFIX as i32);
    ids.extend(tok.encode(suffix).into_iter().map(|t| t as i32));
    ids.push(FIM_MIDDLE as i32);

    let start_time = std::time::Instant::now();
    let new = generate_ids(model, ids, max_new_tokens, context_length, temperature, top_k, device);
    let elapsed = start_time.elapsed().as_secs_f64();
    let count = new.len();
    let tok_per_sec = count as f64 / elapsed.max(1e-4);
    eprintln!("[FIM Complete] {} tokens in {:.3}s ({:.1} tok/s)", count, elapsed, tok_per_sec);

    let out: Vec<u32> = new
        .into_iter()
        .filter(|&t| t as u32 != EOS)
        .map(|t| t as u32)
        .collect();
    tok.decode(&out)
}