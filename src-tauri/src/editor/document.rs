use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Document {
    pub path: PathBuf,
    pub text: String,
    pub language: String,
    pub version: usize,
}


impl Document {

    pub fn new(path: PathBuf, text: String) -> Self {

        let language = detect_language(&path);

        Self {

            path,
            text,
            language,
            version: 1,
        }
    }
}

fn detect_language(path: &PathBuf) -> String {

    match path.extension().and_then(|ext| ext.to_str()) {
        Some("rs") => "rust".to_string(),
        Some("js") => "javascript".to_string(),
        Some("css") => "css".to_string(),
        Some("html") => "html".to_string(),
        Some("py") => "python".to_string(),
        Some("go") => "go".to_string(),
        Some("c") => "c".to_string(),
        Some("cpp") => "cpp".to_string(),
        Some("java") => "java".to_string(),
        Some("ts") => "typescript".to_string(),
        Some("tsx") => "typescript".to_string(),
        Some("jsx") => "javascript".to_string(),
        Some("json") => "json".to_string(),
        Some("md") => "markdown".to_string(),
        _ => "text".to_string(),

        
    }
}