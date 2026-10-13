use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};

use burn::prelude::*;
use burn::tensor::TensorData;
use rand::RngExt;

use crate::tokenizer::{Tokenizer, EOS, FIM_MIDDLE, FIM_PREFIX, FIM_SUFFIX, PAD};

pub const FIM_FILE: &str = "dataset/fim.bin";
pub const ROW: usize = 513; // 512 context + 1 for the shifted target
pub const VAL_ROWS: usize = 2000;

fn flush(block: &mut String, seen: &mut HashSet<u64>, out: &mut String) {
    if block.is_empty() { return; }

    let mut h = DefaultHasher::new();
    block.hash(&mut h);
    if seen.insert(h.finish()) {
        out.push_str(block);

    }
    block.clear();
}

// drop doc commaents, #[cfg(text)] modules, and duplicate 40-line blocks this is amzing :)))
fn clean(text: &str) -> String {
    let mut out = String::new();
    let mut seen: HashSet<u64> = HashSet::new();
    let mut block = String::new();
    let mut n = 0;
    let mut skipping = false;
    let mut depth: i32 = 0;
    let mut pending_cfg = false;

    for line in text.lines() {
        let t = line.trim_start();
        let opens = line.matches('{').count() as i32;
        let closes = line.matches('}').count() as i32;

        if skipping {
            depth += opens - closes;
            if depth <= 0 {
                skipping = false;
            }
            continue;
        }

        if t.starts_with("///") || t.starts_with("//!") {
            continue;
        }
        if t.starts_with("#[cfg(test)]") {
            pending_cfg = true;
            continue;
        }

        if pending_cfg {
            pending_cfg = false;
            if t.starts_with("mod ") {
                depth = opens - closes;
                if depth > 0 {
                    skipping = true;
                }
                continue;
            }
        }

        block.push_str(line);
        block.push('\n');
        
        n += 1;
        if n == 40 {
            flush(&mut block, &mut seen, &mut out);
            n = 0;
        }
    }

    flush(&mut block, &mut seen, &mut out);
    out
}

fn pick_cut(win: &str) -> Option<(usize, usize)> {
    let mut rng = rand::rng();
    let b = win.as_bytes();
    let len = win.len();
    let mut a: usize;
    let mut e: usize;

    if rng.random::<f32>() < 0.6 {

        // cute where a cursor really site: after ( { = , . : or at a line start
        let cands: Vec<usize> = win.char_indices().filter(|&(i, c)| i > 0 && matches!(c, '(' | '{' | '=' | ',' | '\n' | '.' | ':')).map(|(i, c)| i + c.len_utf8()).collect();
        if cands.is_empty() {
            return None;
        }
        a = cands[rng.random_range(0..cands.len())];
        
        if win[..a].ends_with('\n') {
            while a < len && b[a] == b' ' {
                a += 1; // cursor after the indentation
            }
        } else if a < len && b[a] == b' ' && rng.random::<f32>() < 0.5 {
            a += 1; // sometimes the user alrdy typed the space 
        }

        if a >= len {
            return None;
        }

        e = match win[a..].find('\n') {
            Some(p) => a + p,
            None => len,
        };

        if rng.random::<f32>() < 0.3 {
            // multi-line completion
            for _ in 0..rng.random_range(1..6usize) {
                if e < len {
                    match win[e + 1..].find('\n') {
                        Some(p) => e = e + 1 + p,
                        None => e = len,
                    }
                }
            }
        }
    } else {
        a = rng.random_range(0..len);
        while !win.is_char_boundary(a) {
            a += 1;
        }

        e = (a + rng.random_range(1..=150usize)).min(len);

        while !win.is_char_boundary(e) {
            e -= 1;
        }


    }

    if e <= a {
        return None;
    }

    Some((a, e))
}

fn window(text: &str) -> &str {
    let mut rng = rand::rng();
    let want = rng.random_range(300..2500usize);
    if text.len() <= want + 10 {
        return text;
    }

    let mut s = rng.random_range(0..text.len() - want);
    while !text.is_char_boundary(s) {
        s += 1;
    }

    if let Some(p) = text[s..].find('\n') {
        s += p + 1;
    }

    let mut e = (s + want).min(text.len());
    while !text.is_char_boundary(e) {
        e -= 1;
    }

    if e <= s {
        return &text[0..0];
    }
    &text[s..e]
}

fn fim_row(tok: &Tokenizer, cache: &mut HashMap<Vec<u8>, Vec<u32>>, win: &str) -> Option<Vec<u16>> {
    if win.len() < 40 {
        return None;
    }

    let (a, e) = pick_cut(win)?;
    let p = tok.encode_cached(&win[..a], cache);
    let s = tok.encode_cached(&win[e..], cache);
    let mut m = tok.encode_cached(&win[a..e], cache);
    m.push(EOS);
    if m.len() > 200 {
        return None;
    }


    // keep the text nearest the cursor: trim preifix from the left, suffix from the right
    let room = ROW - 3 - m.len();
    let s_keep = s.len().min(room / 3);
    let p_keep = p.len().min(room - s_keep);

    let mut row: Vec<u16> = Vec::with_capacity(ROW);
    row.push(FIM_PREFIX as u16);
    row.extend(p[p.len() - p_keep..].iter().map(|&t| t as u16));
    row.push(FIM_SUFFIX as u16);
    row.extend(s[..s_keep].iter().map(|&t| t as u16));
    row.push(FIM_MIDDLE as u16);
    row.extend(m.iter().map(|&t| t as u16));
    row.resize(ROW, PAD as u16);
    Some(row)
    
}


fn plain_row(tok: &Tokenizer, cache: &mut HashMap<Vec<u8>, Vec<u32>>, win: &str) -> Option<Vec<u16>> {

    if win.len() < 40 {
        return None;
    }

    let mut t = tok.encode_cached(win, cache);
    t.truncate(ROW - 1);
    t.push(EOS);
    let mut row: Vec<u16> = t.iter().map(|&x| x as u16).collect();
    row.resize(ROW, PAD as u16);
    Some(row)
}

fn write_rows(
    tok: &Tokenizer,
    cache: &mut HashMap<Vec<u8>, Vec<u32>>,
    text: &str,
    count: usize,
    plain_rate: f32,
    out: &mut Vec<u8>,
) {
    let mut rng = rand::rng();
    let mut done = 0;
    while done < count {
        let w = window(text);
        let row = if rng.random::<f32>() < plain_rate {
            plain_row(tok, cache, w)
        } else {
            fim_row(tok, cache, w)
        };
        if let Some(r) = row {
            for t in r {
                out.extend_from_slice(&t.to_le_bytes());
            }
            done += 1;
            if done % 20000 == 0 {
                println!("   {} / {} rows", done, count);
            }
        }
    }
}

pub fn build(tok: &Tokenizer, corpus_file: &str, train_rows: usize) -> Result<(), String> {
    let raw = fs::read(corpus_file).map_err(|e| format!("Could not read {}: {}", corpus_file, e))?;
    let text = clean(&String::from_utf8_lossy(&raw));
    println!("corpus {:.1} MB -> cleaned {:.1} MB", raw.len() as f64 / 1e6, text.len() as f64 / 1e6);


    // the last 2% of the text is only used for validation
    let mut cut = text.len() * 98 / 100;
    while !text.is_char_boundary(cut) {
        cut += 1;
    }
    let (train_text, val_text) = text.split_at(cut);

    let mut cache: HashMap<Vec<u8>, Vec<u32>> = HashMap::new();
    let mut out: Vec<u8> = Vec::with_capacity((train_rows + VAL_ROWS) * ROW * 2);
    println!("Building {} train rows (20% plain, 80% FIM)...", train_rows);
    write_rows(tok, &mut cache, train_text, train_rows, 0.2, &mut out);
    println!("building {} validation rows (FIM only)...", VAL_ROWS);
    write_rows(tok, &mut cache, val_text, VAL_ROWS, 0.0, &mut out);

    fs::write(FIM_FILE, &out).map_err(|e| format!("Could not write {}: {}", FIM_FILE, e))?;
    println!("saved {} ({:.0} MB)", FIM_FILE, out.len() as f64 / 1e6);

    // show 3 rows so you can check the format by eye
    for i in 0..3 {
        let ids: Vec<u32> = (0..ROW).map(|k| u16::from_le_bytes([out[(i * ROW + k) * 2], out[(i * ROW + k) * 2 + 1]]) as u32).filter(|&t| t != PAD).collect();

        println!("\n==== sample row {} ====\n{}", i, tok.decode(&ids));
    }
    Ok(())
}

// returns (all rows, number of train rows, total rows)
pub fn load() -> Result<(Vec<u16>, usize, usize), String> {
    let raw = fs::read(FIM_FILE).map_err(|e| format!("Could not read {}: {}", FIM_FILE, e))?;
    let rows: Vec<u16> = raw.chunks_exact(2).map(|b| u16::from_le_bytes([b[0], b[1]])).collect();
    let total = rows.len() / ROW;
    if total <= VAL_ROWS {
        return Err("fim.bin is too small".to_string());
    }

    Ok((rows,  total - VAL_ROWS, total))
}

// random batch from rows [lo, hi]. val=true: only FIM rows, loss only on the middle part
pub fn batch<B: Backend>(
    rows: &[u16],
    lo: usize,
    hi: usize,
    batch_size: usize,
    val: bool,
    device: &B::Device,
) -> (Tensor<B, 2, Int>, Tensor<B, 2, Int>) {
    let mut rng = rand::rng();
    let mut x: Vec<i32> = Vec::with_capacity(batch_size * (ROW - 1));
    let mut y: Vec<i32> = Vec::with_capacity(batch_size * (ROW - 1));

    let mut got = 0;
    while got < batch_size {
       
        let r = rng.random_range(lo..hi);
        let row = &rows[r * ROW..(r + 1) * ROW];
        let mid = row.iter().position(|&t| t as u32 == FIM_MIDDLE);
        if val && mid.is_none() {
            continue;
        }

        x.extend(row[..ROW - 1].iter().map(|&t| t as i32));
        let mut yi: Vec<i32> = row[1..].iter().map(|&t| t as i32).collect();
        if val {
            if let Some(p) = mid {
                for k in 0..p {
                    yi[k] = PAD as i32; // ignore the  prefix/suffix part
                }
            }
        }

        y.extend(yi);
        got += 1;
    }

    let x = Tensor::<B, 2, Int>::from_ints(TensorData::new(x, [batch_size, ROW - 1]), device);
    let y = Tensor::<B, 2, Int>::from_ints(TensorData::new(y, [batch_size, ROW - 1]), device);
    (x, y)
}
