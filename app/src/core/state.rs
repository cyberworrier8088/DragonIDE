use std::sync::Mutex;

use crate::editor::manager::DocumentManager;

pub struct IdeState {
    pub documents: Mutex<DocumentManager>,
}

impl Default for IdeState {
    fn default() -> Self {
        Self {
            documents: Mutex::new(DocumentManager::new()),
        }
    }
}
