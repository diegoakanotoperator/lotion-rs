use tauri::{AppHandle, Manager, WebviewWindow, WebviewWindowBuilder};
use std::sync::Arc;
use crate::traits::{SecuritySandbox, TabOrchestrator};

pub struct WindowController {
    pub window: WebviewWindow,
    pub security: Arc<dyn SecuritySandbox>,
}

impl WindowController {
    pub fn new(
        app: &AppHandle,
        security: Arc<dyn SecuritySandbox>,
    ) -> tauri::Result<Self> {
        let window = WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::Default)
            .title("Lotion")
            .inner_size(1200.0, 800.0)
            .decorations(false) // Frameless
            .build()?;

        Ok(Self { window, security })
    }

    pub fn setup_tabs(&self, app: &AppHandle) -> tauri::Result<()> {
        let tab_manager = app.state::<Arc<dyn TabOrchestrator>>();
        let notion_url = "https://www.notion.so";
        
        log::info!("WindowController: Creating initial tab for Notion: {}", notion_url);
        tab_manager.create_tab(app, self.window.label(), notion_url)?;
        
        Ok(())
    }
}
