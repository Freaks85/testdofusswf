use crate::core::types::SWFFile;
use std::collections::HashMap;
use std::sync::Mutex;

/// Application state
pub struct AppState {
    pub current_swf: Option<SWFFile>,
    pub cache: HashMap<String, SWFFile>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_swf: None,
            cache: HashMap::new(),
        }
    }

    pub fn set_current_swf(&mut self, swf: SWFFile) {
        let path = swf.path.clone();
        self.cache.insert(path.clone(), swf.clone());
        self.current_swf = Some(swf);
    }

    pub fn get_current_swf(&self) -> Option<&SWFFile> {
        self.current_swf.as_ref()
    }

    pub fn clear_current(&mut self) {
        self.current_swf = None;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe state wrapper
pub type SharedState = Mutex<AppState>;
