#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;

pub const TOKENIZER_FILE: &str = "checkpoints/tokenizer.json";
pub const VOCAB_SIZE: usize = 4096;

pub const PAD: u32 = 256;
pub const EOS: u32 = 257;
pub const FIM_PREFIX: u32 = 258;
pub const FIM_SUFFIX: u32 = 259;
pub const FIM_MIDDLE: u32 = 260;
const SPECIALS: [&str; 5] = ["<pad>", "<eos>", "<fim_prefix>", "<fim_suffix>", "<fim_middle>"];
const FIRST_MERGE_ID: u32 = 261;


pub struct Tokenizer {
    merges: Vec<(u32, u32)>,
    ranks: HashMap<(u32, u32), u32>, // pair -> new token id (lower id = leaned earlier)
    vocab: Vec<Vec<u8>>, // id -> bytes
}

// pre-tok: merges never cross these boundaries

fn is_word(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b >= 128 // >=128 keeps UTF-8 sequences together
}

fn is_space(b: u8) -> bool {
    b == b' ' || b == b'\n' || b == b'\t' || b == b'\r'
}

pub fn pre_tokenize(text: &[u8]) -> Vec<&[u8]> {

    let mut out = Vec::new();
    let mut i = 0;

    while i < text.len() {
        let start = i;
        let b = text[i];

        if b == b' ' && i + 1 < text.len() && is_word(text[i + 1]) {

            i += 1;
            while i < text.len() && is_word(text[i]) {
                i += 1;
            }
        } else if is_space(b) {
            // whitespace run (indentation stays one chunk)
            while i < text.len() && is_space(text[i]) {
                i += 1;
            }
        } else if is_word(b) {
            while i < text.len() && is_word(text[i]) {
                i +=1;
            }
        } else {
            // single punctuation byte: ( ) { } ; etc.
            i += 1;
        }

        out.push(&text[start..i]);
    }
    out
}


fn merge_pair(word: &mut Vec<u32>, pair: (u32, u32), new_id: u32) {

    let len = word.len();
    let (mut r, mut w) = (0, 0);

    while r < len {

        if r + 1 < len && word[r] == pair.0 && word[r + 1] == pair.1 {
            word[w] = new_id;
            r += 2;
        } else {
            word[w] = word[r];
            r += 1;
        }
        w += 1;
    }

    word.truncate(w);
}

impl Tokenizer {
    fn from_merges(merges: Vec<(u32, u32)>) -> Self {
        let mut vocab: Vec<Vec<u8>> = (0..=255u8).map(|b| vec![b]).collect();
        for s in SPECIALS {
            vocab.push(s.as_bytes().to_vec());
        }

        let mut ranks = HashMap::new();
        for (i, &(a, b)) in merges.iter().enumerate() {
            let mut bytes = vocab[a as usize].clone();
            bytes.extend_from_slice(&vocab[b as usize]);
            vocab.push(bytes);
            ranks.insert((a, b), FIRST_MERGE_ID + i as u32);
        }

        Self { merges, ranks, vocab}
    }

    pub fn vocab_size(&self) -> usize {
        self.vocab.len()
    }

    pub fn train(text: &str, vocab_size: usize) -> Self {

        // count each unique pre-token oncee, with its frequency
        let mut counts: HashMap<&[u8], usize> = HashMap::new();

        for piece in pre_tokenize(text.as_bytes()) {
            *counts.entry(piece).or_insert(0) += 1;
        }

        let mut words: Vec<(Vec<u32>, usize)> = counts.into_iter().map(|(p, c)| (p.iter().map(|&b| b as u32).collect(), c)).collect();

        println!("unique pre-tokens: {}", words.len());

        let mut merges: Vec<(u32, u32)> = Vec::new();
        let mut next_id = FIRST_MERGE_ID;

        while (next_id as usize) < vocab_size {
            let mut pair_counts: HashMap<(u32, u32), usize> = HashMap::new();
            for (w, c) in &words {
                for p in w.windows(2) {
                    *pair_counts.entry((p[0], p[1])).or_insert(0) += *c;
                }
            }

            // most frequent pair; ties -> Smallest pair
            let best = pair_counts.iter().max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)));

            let Some((&pair, &count)) = best else { break };
            if count < 2 {
                break;
            }

            for (w, _) in words.iter_mut() {
                if w.len() >= 2 {
                    merge_pair(w, pair, next_id);
                }
            }

            merges.push(pair);
            next_id += 1;

            if merges.len() % 200 == 0 {
                println!("merges: {} / {}", merges.len(), vocab_size - FIRST_MERGE_ID as usize);
            }
        }

        Self::from_merges(merges)
    }

    // test -> token ids (no spicial tokens added)
    pub fn encode(&self, text: &str) -> Vec<u32> {
        let mut out = Vec::new();

        for piece in pre_tokenize(text.as_bytes()) {

            let mut ids: Vec<u32> = piece.iter().map(|&b| b as u32).collect();

            loop {
                // find the pair that was learned earliest
                let mut best: Option<(u32, usize)> = None;
                for i in 0..ids.len().saturating_sub(1) {
                    if let Some(&new_id) = self.ranks.get(&(ids[i], ids[i + 1])) {
                        if best.map_or(true, |(b, _)| new_id < b) {
                            best = Some((new_id, i));
                        }
                    }
                }

                let Some((new_id, pos)) = best else { break };
                let pair = (ids[pos], ids[pos + 1]);
                merge_pair(&mut ids, pair, new_id);
            }

            out.extend(ids);
        }
        out
    }

    // token ids -> textt
    pub fn decode(&self, ids: &[u32]) -> String {
        let mut bytes = Vec::new();
        for &id in ids {
            if let Some(t) = self.vocab.get(id as usize) {
                bytes.extend_from_slice(t);
            }
        }
        String::from_utf8_lossy(&bytes).into_owned()
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        fs::create_dir_all("checkpoints").map_err(|e| e.to_string())?;

        let mut s = String::from("FOXTOK1\n");
        for (a, b) in &self.merges {
            s.push_str(&format!("{} {}\n", a, b));
        }

        fs::write(path, s).map_err(|e| format!("Could not save tokenizer: {}", e))
    }

    pub fn load(path: &str) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|e| format!("Could not read {}: {}", path, e))?;
        let mut lines = text.lines();

        if lines.next() != Some("FOXTOK1") {
            return Err("not a FoxAi tokenizer file".to_string());
        }

        let mut merges = Vec::new();
        for line in lines {
            let mut parts = line.split(' ');
            let a = parts.next().and_then(|x| x.parse::<u32>().ok());
            let b = parts.next().and_then(|x| x.parse::<u32>().ok());

            match (a, b) {
                (Some(a), Some(b)) => merges.push((a, b)),
                _ => return Err(format!("bad line to tokenizer file: {}", line)),
            }
        }
        Ok(Self::from_merges(merges))
    }
}

pub fn train_and_save(corpus_file: &str, vocab_size: usize, max_bytes: usize) -> Result<Tokenizer, String> {

    let bytes = fs::read(corpus_file).map_err(|e| format!("Could not read {}: {}", corpus_file, e))?;

    const CHUNK: usize = 64 * 1024;
    let mut sample: Vec<u8> = Vec::new();

    if bytes.len() <= max_bytes {
        sample = bytes;
    } else {
        let chunks = max_bytes / CHUNK;
        let step = bytes.len() / chunks;
        for i in 0..chunks {
            let start = i * step;
            let end = (start + CHUNK).min(bytes.len());
            sample.extend_from_slice(&bytes[start..end]);

        }
    }

    let text = String::from_utf8_lossy(&sample);
    println!("training tokenizer on {:.1} MB", text.len() as f64 / 1e6);

    let tok = Tokenizer::train(&text, vocab_size);
    tok.save(TOKENIZER_FILE)?;
    println!("saved {} (vocab {})", TOKENIZER_FILE, tok.vocab_size());
    Ok(tok)
}

pub fn demo(tok: &Tokenizer) {
    let samples = [
        "fn main() {\n    println!(\"hello\");\n}",
        "let café = 5; // ✓ unicode",
    ];

    for s in samples {
        let ids = tok.encode(s);
        println!("chars: {:>3} | tokens: {:>3} | round trip ok: {}", s.chars().count(), ids.len(), tok.decode(&ids) == s);
    }
}