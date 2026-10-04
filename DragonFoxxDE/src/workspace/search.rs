use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    pub path: String,
    pub name: String,
    pub line: usize,
    pub text: String,
}

const SKIP_DIRS: &[&str] = &[".git", "target", "node_modules", "dist", "build"];
const MAX_FILE_SIZE: u64 = 1024 * 1024; // skip files bigger than 1MB
const MAX_RESULTS: usize = 1000;

pub fn search_workspace(root: &Path, query: &str, case_sensitive: bool) -> Result<Vec<SearchResult>, String> {
    if !root.is_dir() {
        return Err("Folder not found".to_string());
    }
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let needle = if case_sensitive { query.to_string() } else { query.to_lowercase() };
    let mut results = Vec::new();
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];

    'walk: while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let file_type = match entry.file_type() {
                Ok(t) => t,
                Err(_) => continue,
            };

            if file_type.is_dir() {
                let name_lower = name.to_lowercase();
                if !SKIP_DIRS.iter().any(|&d| d == name_lower) {
                    stack.push(path);
                }
                continue;
            }

            if !file_type.is_file() {
                continue; // skip symlinks and special files
            }

            if entry.metadata().map(|m| m.len() > MAX_FILE_SIZE).unwrap_or(true) {
                continue;
            }

            let content = match fs::read_to_string(&path) {

                Ok(c) => c,
                Err(_) => continue, // binary or unreadable file
            };

            for (index, line) in content.lines().enumerate() {

                let found = if case_sensitive {
                    line.contains(needle.as_str())
                } else {
                    line.to_lowercase().contains(needle.as_str())
                };

                if found {
                    results.push(
                        SearchResult {
                            path: path.to_string_lossy().to_string(),
                            name: name.clone(),
                            line: index + 1,
                            text: line.trim().chars().take(200).collect(),
                        }
                    );

                    if results.len() >= MAX_RESULTS {
                        break 'walk;
                    }
                }
            }
        }
    }

    results.sort_by(|a, b| a.path.cmp(&b.path).then(a.line.cmp(&b.line)));
    Ok(results)
}