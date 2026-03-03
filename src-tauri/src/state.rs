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
pub struct WindowState {
    pub id: String,
    pub bounds: Bounds,
    pub is_focused: bool,
    pub is_maximized: bool,
    pub is_minimized: bool,
    pub is_full_screen: bool,
    pub title: String,
    pub url: Option<String>,
    pub tab_ids: Vec<String>,
    pub active_tab_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub windows: HashMap<String, WindowState>,
    pub focused_window_id: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            focused_window_id: None,
        }
    }
}
