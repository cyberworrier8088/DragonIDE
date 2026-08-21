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