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

    pub fn setup_listeners(&self, app_handle: AppHandle) {
        let window_ = self.window.clone();
        let window_id = self.window.label().to_string();
        window_.on_window_event(move |event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    log::info!("Window {} close requested", window_id);
                    // Prevent immediate close, allow for custom logic (e.g., saving state)
                    api.prevent_close();
                    // In a real app, you might emit an event or call a method to handle cleanup
                    // For now, we'll just close it directly for demonstration
                    let _ = window_.close();
                }
                tauri::WindowEvent::Focused(focused) => {
                    log::debug!("Window {} focused: {}", window_id, focused);
                    // TODO: Dispatch action to update Redux-like store
                }
                tauri::WindowEvent::Resized(size) => {
                    log::debug!("Window {} resized to {:?}", window_id, size);
                    // TODO: Dispatch action to update Redux-like store and update view bounds
                }
                tauri::WindowEvent::Moved(position) => {
                    log::debug!("Window {} moved to {:?}", window_id, position);
                    // TODO: Dispatch action to update Redux-like store
                }
                _ => {}
            }
        });
    }

    pub fn setup_tabs(&self, app: &AppHandle) -> tauri::Result<()> {
        let tab_manager = app.state::<Arc<dyn TabOrchestrator>>();
        let notion_url = "https://www.notion.so";
        
        log::info!("WindowController: Creating initial tab for Notion: {}", notion_url);
        tab_manager.create_tab(app, self.window.label(), notion_url)?;
        
        Ok(())
    }
}
