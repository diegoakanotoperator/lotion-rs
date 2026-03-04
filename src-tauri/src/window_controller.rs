use tauri::{AppHandle, Manager, Window, WindowBuilder};
use std::sync::Arc;
use crate::traits::{SecuritySandbox, TabOrchestrator};

pub struct WindowController {
    pub window: Window,
    pub security: Arc<dyn SecuritySandbox>,
}

impl WindowController {
    pub fn new(
        app: &AppHandle,
        security: Arc<dyn SecuritySandbox>,
    ) -> tauri::Result<Self> {
        let window = WindowBuilder::new(app, "main")
            .title("Lotion-Engine")
            .inner_size(1200.0, 768.0)
            .decorations(false) // Frameless to allow Iced to provide the chrome
            .transparent(true)  // Allow transparency for a unified look
            .build()?;

        Ok(Self { window, security })
    }

    pub fn setup_listeners(&self, app_handle: AppHandle) {
        let window_label = self.window.label().to_string();
        
        self.window.on_window_event(move |event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    log::info!("Window {} close requested", window_label);
                    api.prevent_close();
                    if let Some(w) = app_handle.get_window(&window_label) {
                        let _ = w.close();
                    }
                }
                tauri::WindowEvent::Focused(focused) => {
                    log::debug!("Window {} focused: {}", window_label, focused);
                    // TODO: Dispatch action to update Redux-like store
                }
                tauri::WindowEvent::Resized(size) => {
                    log::debug!("Window {} resized to {:?}", window_label, size);
                    // TODO: Dispatch action to update Redux-like store and update view bounds
                }
                tauri::WindowEvent::Moved(position) => {
                    log::debug!("Window {} moved to {:?}", window_label, position);
                    // TODO: Dispatch action to update Redux-like store
                }
                _ => {}
            }
        });
    }

    pub fn setup_tabs(&self, app: &AppHandle) -> tauri::Result<()> {
        let tab_manager = {
            let mut attempts = 0;
            loop {
                if let Some(state) = app.try_state::<Arc<dyn TabOrchestrator>>() {
                    break state;
                }
                attempts += 1;
                if attempts > 60 {
                    log::error!("WindowController: TabOrchestrator state not available after 3s");
                    return Err(tauri::Error::AssetNotFound("TabOrchestrator state timeout".into()));
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        };

        let config = app.state::<crate::config::LotionConfig>();
        let mut tabs_restored = false;

        if config.restore_tabs {
            let app_state_lock = app.state::<Arc<tokio::sync::Mutex<crate::state::AppState>>>();
            let app_state = app_state_lock.blocking_lock();

            // Find state for THIS window
            if let Some(window_state) = app_state.windows.get(self.window.label()) {
                log::info!("WindowController: Restoring {} tabs from saved state.", window_state.tab_ids.len());
                for tab_id in &window_state.tab_ids {
                    if let Some(tab_state) = app_state.tabs.get(tab_id) {
                        tab_manager.create_tab(app, self.window.label(), &tab_state.url)?;
                        tabs_restored = true;
                    }
                }
            }
        }

        if !tabs_restored {
            let notion_url = "https://www.notion.so";
            log::info!("WindowController: Creating initial tab for Notion: {}", notion_url);
            tab_manager.create_tab(app, self.window.label(), notion_url)?;
        }

        Ok(())
    }
}
