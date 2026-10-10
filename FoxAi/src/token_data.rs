use std::collections::HashMap;
use std::fs;

use crate::tokenizer::{Tokenizer, EOS};

pub const TOKENS_FILE: &str = "dataset/tokens.bin";
const CHUNK: usize = 256 * 1024;

pub fn build(tok: &Tokenizer, corpus_file: &str, max_bytes: usize) -> Result<(), String> {

    let bytes = fs::read(corpus_file).map_err(|e| format!("Could not read {}: {}", corpus_file, e))?;

    let chunks = (max_bytes / CHUNK).max(1);
    let step = (bytes.len() / chunks).max(1);

    let mut cache: HashMap<Vec<u8>, Vec<u32>> = HashMap::new();
    let mut ids: Vec<u32> = Vec::new();
    let mut text_bytes = 0usize;

    for i in 0..chunks {
        let mut start = i * step;
        if start >= bytes.len() {
            break;
        }

        let mut end = (start + CHUNK).min(bytes.len());

        // snap to line boundaries so we never cut a line or a UTF-8 character
        if start > 0 {
            if let Some(p) = bytes[start..end].iter().position(|&b| b == b'\n') {
                start += p + 1;
            }
        }

        if end < bytes.len() {
            if let Some(p) = bytes[start..end].iter().rposition(|&b| b == b'\n') {
                end = start + p + 1;
            }
        }
        if start >= end {
            continue;
        }

        let text = String::from_utf8_lossy(&bytes[start..end]);
        text_bytes += text.len();
        ids.extend(tok.encode_cached(&text, &mut cache));
        ids.push(EOS);

        if i % 20 == 0 {
            println!("chunk {} / {}", i, chunks);
        }
    }

    let mut out: Vec<u8> = Vec::with_capacity(ids.len() * 2);
    for id in &ids {
        out.extend_from_slice(&(*id as u16).to_le_bytes());
    }
    fs::write(TOKENS_FILE, out).map_err(|e| format!("Could not write {}: {}", TOKENS_FILE, e))?;

    println!(
        "{} tokens saved to {} ({:.2} bytes per token)",
        ids.len(),
        TOKENS_FILE,
        text_bytes as f64 / ids.len() as f64
    );

    Ok(())
}


pub fn load() -> Result<(Vec<i32>, Vec<i32>), String> {

    let raw = fs::read(TOKENS_FILE).map_err(|e| format!("Could not read {}: {}", TOKENS_FILE, e))?;

    let data: Vec<i32> = raw.chunks_exact(2).map(|b| u16::from_le_bytes([b[0], b[1]]) as i32).collect();

    let (mut train, mut val) = (Vec::new(), Vec::new());

    for (i, block) in data.chunks(4096).enumerate() {
        if i % 20 == 19 {
            val.extend_from_slice(block);
        } else {
            train.extend_from_slice(block);
        }
    }

    Ok((train, val))
}