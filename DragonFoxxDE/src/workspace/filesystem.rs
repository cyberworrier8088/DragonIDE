use std::fs;
use std::path::Path;

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
        b.is_directory.cmp(&a.is_directory).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(files)
}

pub fn read_file(path: &Path) -> Result<String, String> {

    fs::read_to_string(path).map_err(|error| error.to_string())
}

pub fn create_file(path: &Path) -> Result<(), String> {

    fs::File::create(path).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn create_directory(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn rename_entry(old_path: &Path, new_path: &Path) -> Result<(), String> {
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