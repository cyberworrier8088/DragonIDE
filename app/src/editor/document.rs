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
        // the editor is a textarea, which turns CRLF into LF: keep the file's own line endings
        let crlf = self
            .disk_text
            .split_once('\n')
            .is_some_and(|(line, _)| line.ends_with('\r'));
        self.text = if crlf && !text.contains('\r') {
            text.replace('\n', "\r\n")
        } else {
            text
        };
        self.version += 1;
        self.modified = self.text != self.disk_text;
    }

    pub fn mark_saved(&mut self) {
        self.disk_text = self.text.clone();
        self.modified = false;
    }
}
