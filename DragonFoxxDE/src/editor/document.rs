use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Document {
    pub path: PathBuf,
    pub text: String,
    pub version: usize,
    pub modified: bool,
}

impl Document {
    pub fn new(path: PathBuf, text: String) -> Self {
        Self {
            path,
            text,
            version: 1,
            modified: false,
        }
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.version += 1;
        self.modified = true;
    }

    pub fn mark_saved(&mut self) {
        self.modified = false;
    }
}
