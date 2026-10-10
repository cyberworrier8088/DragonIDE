#![allow(dead_code)]

use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub const CODE_FILE: &str = "dataset/code.txt";

const VALID_EXTENSIONS: &[&str] = &["rs"];

const IGNORE_DIRS: &[&str] = &[
    "target", ".git", "node_modules", "checkpoints", "dataset", "img",
];


const MAX_FILE_BYTES: u64 = 200_000;
const MAX_TOTAL_BYITES: usize = 150_000_000;

/// Collect source files and assemble the training corpus into `dataset/code.txt`.
pub fn build() -> Result<(), String> {
    println!("Building code corpus...");
    fs::create_dir_all("dataset").map_err(|e| format!("Failed to create dataset directory: {}", e))?;

    let mut out_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(CODE_FILE)
        .map_err(|e| format!("Failed to open {}: {}", CODE_FILE, e))?;

    let mut count = 0;
    let mut total_bytes = 0;
    let mut visited: HashSet<PathBuf> = HashSet::new();

    // Look in current directory (FoxAi) and parent directory (DragonIDE)
    let mut roots: Vec<PathBuf> = vec![PathBuf::from("src"), PathBuf::from("..")];

    let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).unwrap_or_default();
    roots.push(PathBuf::from(home).join(".cargo").join("registry").join("src"));

    for p in &roots {
        if p.exists() {
            println!("Scanning {}", p.display());
            collect_files(p, &mut out_file, &mut count, &mut total_bytes, &mut visited)?;
        }
    }

    // Fallback: If nothing was collected or total_bytes is 0, append sample training data
    if total_bytes == 0 {
        if let Ok(content) = fs::read(crate::data::DATASET_FILE) {
            out_file.write_all(&content).map_err(|e| e.to_string())?;
            total_bytes += content.len();
            count += 1;
        }
    }

    println!(
        "Corpus built: {} files, {:.2} MB written to {}",
        count,
        total_bytes as f64 / 1_000_000.0,
        CODE_FILE
    );
    Ok(())
}

fn collect_files(
    dir: &Path,
    out: &mut File,
    count: &mut usize,
    total_bytes: &mut usize,
    visited: &mut HashSet<PathBuf>,
) -> Result<(), String> {

    if *total_bytes >= MAX_TOTAL_BYITES {
        return Ok(());
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(canonical) = path.canonicalize() {
                if !visited.insert(canonical) {
                    continue;
                }
            }
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if IGNORE_DIRS.contains(&name) || name.starts_with('.') {
                        continue;
                    }
                }
                collect_files(&path, out, count, total_bytes, visited)?;
            } else if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if VALID_EXTENSIONS.contains(&ext) {

                        let too_big = entry.metadata().map(|m| m.len() > MAX_FILE_BYTES).unwrap_or(true);
                        if too_big {
                            continue;
                        }

                        
                        if let Ok(content) = fs::read(&path) {
                            if !content.is_empty() {
                                let _ = out.write_all(&content);
                                let _ = out.write_all(b"\n\n");
                                *count += 1;
                                *total_bytes += content.len() + 2;
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
