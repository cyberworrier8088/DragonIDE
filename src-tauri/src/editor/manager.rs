use std::collections::HashMap;
use std::path::PathBuf;

use super::document::Document;


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

    pub fn open (
        &mut self,
        path: PathBuf,
        text: String,
    ) -> &Document {

        self.documents.entry(path.clone()).or_insert_with(|| Document::new(path, text))
    }

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

        let document = self.documents.get_mut(path).ok_or_else(|| "Document is not open".to_string())?;

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

        manager.open(
            path.clone(),
            "fn main() {}".to_string(),

        );

        assert_eq!(manager.count(), 1);
        assert!(manager.get(&path).is_some());
    }
}