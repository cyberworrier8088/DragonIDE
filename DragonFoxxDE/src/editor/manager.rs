use std::collections::HashMap;
use std::path::PathBuf;

use super::document::Document;

pub const CHANGED_ON_DISK: &str = "changed on disk";

#[derive(Debug, Default)]
pub struct DocumentManager {
    documents: HashMap<PathBuf, Document>,
}

impl DocumentManager {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    pub fn open(&mut self, path: PathBuf, text: String) -> &Document {
        let document = self
            .documents
            .entry(path.clone())
            .or_insert_with(|| Document::new(path, text.clone()));

        if !document.modified && document.disk_text != text {
            document.text = text.clone();
            document.disk_text = text;
        }
        document
    }

    #[allow(dead_code)] // used by tests; not called by the app yet
    pub fn get(&self, path: &PathBuf) -> Option<&Document> {
        self.documents.get(path)
    }

    pub fn get_mut(&mut self, path: &PathBuf) -> Option<&mut Document> {
        self.documents.get_mut(path)
    }

    pub fn close(&mut self, path: &PathBuf) -> Option<Document> {
        self.documents.remove(path)
    }

    pub fn count(&self) -> usize {
        self.documents.len()
    }

    pub fn save(&mut self, path: &PathBuf) -> Result<(), String> {
        let document = self
            .documents
            .get_mut(path)
            .ok_or_else(|| "Document is not open".to_string())?;

        if !force {
            if let Ok(disk) = std::fs::read_to_string(&document.path) {
                if disk != document.disk_text {
                    return Err(format!(
                        "{}: anthor prgram changed this file",
                        CHANGED_ON_DISK
                    ));
                }
            }
        }

        std::fs::write(&document.path, &document.text).map_err(|error| error.to_string())?;

        document.mark_saved();

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]

    fn opens_document() {
        let mut manager = DocumentManager::new();

        let path = PathBuf::from("main.rs");

        manager.open(path.clone(), "fn main() {}".to_string());

        assert_eq!(manager.count(), 1);
        assert!(manager.get(&path).is_some());
    }

    #[test]
    fn reopening_after_close_uses_the_new_text() {
        let mut manager = DocumentManager::new();
        let path = PathBuf::from("main.rs");

        manager.open(path.clone(), "old".to_string());
        manager.close(&path);

        assert_eq!(manager.open(path, "new".to_string()).text, "new");
    }
}
