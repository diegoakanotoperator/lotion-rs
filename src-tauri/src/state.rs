use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bounds {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabState {
    pub id: String,
    pub title: String,
    pub url: String,
    pub is_active: bool,
    pub is_pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub id: String,
    pub bounds: Bounds,
    pub is_focused: bool,
    pub is_maximized: bool,
    pub is_minimized: bool,
    pub is_full_screen: bool,
    pub tab_ids: Vec<String>,
    pub active_tab_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub windows: HashMap<String, WindowState>,
    pub tabs: HashMap<String, TabState>,
    pub focused_window_id: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            tabs: HashMap::new(),
            focused_window_id: None,
        }
    }

    /// Returns the state file path (~/.config/lotion/state.json)
    fn state_path() -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("lotion")
            .join("state.json")
    }

    /// Save application state to disk.
    pub fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::state_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        log::info!("AppState saved to {}", path.display());
        Ok(())
    }

    /// Load application state from disk.
    pub fn load_from_disk() -> Option<Self> {
        let path = Self::state_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(contents) => {
                    match serde_json::from_str::<AppState>(&contents) {
                        Ok(state) => {
                            log::info!("AppState loaded from {}", path.display());
                            return Some(state);
                        }
                        Err(e) => log::warn!("Failed to parse state file: {}", e),
                    }
                }
                Err(e) => log::warn!("Failed to read state file: {}", e),
            }
        }
        None
    }
}
