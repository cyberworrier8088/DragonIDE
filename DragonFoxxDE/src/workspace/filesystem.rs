use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
}

pub fn read_directory(root: &Path) -> Result<Vec<FileEntry>, String> {
    let entries = fs::read_dir(root).map_err(|error| error.to_string())?;

    let mut files = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;

        let path = entry.path();

        let name = entry.file_name().to_string_lossy().to_string();

        files.push(FileEntry {
            name,
            path: path.to_string_lossy().to_string(),
            is_directory: path.is_dir(),
        })
    }

    files.sort_by(|a, b| {
        b.is_directory
            .cmp(&a.is_directory)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(files)
}

const MAX_TEXT_FILE: u64 = 50 * 1024 * 1024; // same limit as the image preview

pub fn read_file(path: &Path) -> Result<String, String> {
    // read_to_string loads all of a huge log or binary into memory before it can fail
    if fs::metadata(path).is_ok_and(|m| m.len() > MAX_TEXT_FILE) {
        return Err("This file is too big to open (over 50MB)".to_string());
    }

    fs::read_to_string(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::InvalidData {
            "This is not a text file, so DragonIDE can't open it".to_string()
        } else {
            error.to_string()
        }
    })
}

pub fn create_file(path: &Path) -> Result<(), String> {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn create_directory(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn rename_entry(old_path: &Path, new_path: &Path) -> Result<(), String> {
    // on Windows and macOS "README.md" finds "readme.md" itself, so let a case-only rename through
    if new_path.exists() && new_path.canonicalize().ok() != old_path.canonicalize().ok() {
        return Err("A file or folder with that name already exists".to_string());
    }

    fs::rename(old_path, new_path).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_entry(path: &Path, is_directory: bool) -> Result<(), String> {
    if is_directory {
        fs::remove_dir_all(path).map_err(|e| e.to_string())?;
    } else {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn paste_entry(from: &Path, to_dir: &Path, cut: bool) -> Result<PathBuf, String> {
    if !to_dir.is_dir() {
        return Err("Paste target is not a folder".to_string());
    }
    if from.is_dir() && to_dir.starts_with(from) {
        return Err("Can't paste a folder inside itself".to_string());
    }

    // cut and paste in the same folder: nothing to do
    if cut && from.parent() == Some(to_dir) {
        return Ok(from.to_path_buf());
    }

    let target = free_name(from, to_dir)?;

    if cut {
        fs::rename(from, &target).map_err(|e| e.to_string())?;
    } else if from.is_dir() {
        copy_dir(from, &target)?;
    } else {
        fs::copy(from, &target).map_err(|e| e.to_string())?;
    }
    Ok(target)
}

fn free_name(from: &Path, to_dir: &Path) -> Result<PathBuf, String> {
    let name = from
        .file_name()
        .ok_or("Nothing to paste")?
        .to_string_lossy();

    let first = to_dir.join(name.as_ref());
    if !first.exists() {
        return Ok(first);
    }

    let (stem, ext) = match (from.is_dir(), name.rsplit_once('.')) {
        (false, Some((stem, ext))) if !stem.is_empty() => (stem.to_string(), format!(".{}", ext)),
        _ => (name.to_string(), String::new()),
    };

    for number in 1.. {
        let copy = if number == 1 {
            " copy".to_string()
        } else {
            format!(" copy {}", number)
        };

        let candidate = to_dir.join(format!("{}{}{}", stem, copy, ext));

        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    unreachable!()
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir(to).map_err(|e| e.to_string())?;

    for entry in fs::read_dir(from).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let target = to.join(entry.file_name());

        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rename_changes_case_but_never_overwrites() {
        let dir = std::env::temp_dir().join(format!("dragonide_rename_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("readme.md"), "a").unwrap();
        fs::write(dir.join("other.md"), "b").unwrap();

        rename_entry(&dir.join("readme.md"), &dir.join("README.md")).unwrap();
        let names: Vec<String> = read_directory(&dir)
            .unwrap()
            .into_iter()
            .map(|f| f.name)
            .collect();
        assert!(names.contains(&"README.md".to_string()), "{:?}", names);

        assert!(rename_entry(&dir.join("README.md"), &dir.join("other.md")).is_err());
        assert_eq!(fs::read_to_string(dir.join("other.md")).unwrap(), "b");

        let _ = fs::remove_dir_all(&dir);
    }
}
