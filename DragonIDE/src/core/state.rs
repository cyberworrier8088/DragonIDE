use std::sync::Mutex;

use crate::editor::manager::DocumentManager;

pub struct IdeState {
    pub workspace: Mutex<Option<String>>,
    pub documents: Mutex<DocumentManager>,
}

impl Default for IdeState {
    fn default() -> Self {
        Self {
            workspace: Mutex::new(None),
            documents: Mutex::new(DocumentManager::new()),
        }
    }
}