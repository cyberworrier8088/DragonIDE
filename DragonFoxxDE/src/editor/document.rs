use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Document {
    pub path: PathBuf,
    pub text: String,
    pub version: usize,
    pub modified: bool,
    pub disk_text: String,
}

impl Document {
    pub fn new(path: PathBuf, text: String) -> Self {
        Self {
            path,
            disk_text: text.clone(),
            text,
            version: 1,
            modified: false,
        }
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.version += 1;
        self.modified = self.text != self.disk_text;
    }

    pub fn mark_saved(&mut self) {
        self.disk_text = self.text.clone();
        self.modified = false;
    }
}
