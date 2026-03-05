use crate::traits::{SecuritySandbox, TabOrchestrator};
use std::sync::Arc;
use tauri::{AppHandle, Manager, Window, WindowBuilder};

pub struct WindowController {
    pub window: Window,
    pub security: Arc<dyn SecuritySandbox>,
}

impl WindowController {
    pub fn new(app: &AppHandle, security: Arc<dyn SecuritySandbox>) -> tauri::Result<Self> {
        let window = WindowBuilder::new(app, "main")
            .title("Lotion (Notion Engine)")
            .inner_size(1200.0, 768.0)
            .decorations(false) // Custom injected Mac-like titlebar handles this now
            .build()?;

        // Ensure window state exists in AppState
        let app_state_lock = app.state::<Arc<tokio::sync::Mutex<crate::state::AppState>>>();
        let mut app_state = app_state_lock.blocking_lock();
        if !app_state.windows.contains_key("main") {
            app_state.windows.insert(
                "main".to_string(),
                crate::state::WindowState {
                    id: "main".to_string(),
                    bounds: crate::state::Bounds {
                        x: None,
                        y: None,
                        width: 1200.0,
                        height: 768.0,
                    },
                    is_focused: true,
                    is_maximized: false,
                    is_minimized: false,
                    is_full_screen: false,
                    tab_ids: Vec::new(),
                    active_tab_id: None,
                },
            );
            let _ = app_state.save_to_disk();
        }

        Ok(Self { window, security })
    }

    pub fn setup_listeners(&self, app_handle: AppHandle) {
        let window_label = self.window.label().to_string();

        self.window.on_window_event(move |event| match event {
            tauri::WindowEvent::CloseRequested { .. } => {
                log::info!("Window {} close requested", window_label);
                app_handle.exit(0);
            }
            tauri::WindowEvent::Focused(focused) => {
                log::debug!("Window {} focused: {}", window_label, focused);
                let app_state_lock =
                    app_handle.state::<Arc<tokio::sync::Mutex<crate::state::AppState>>>();
                let mut app_state = app_state_lock.blocking_lock();
                if *focused {
                    app_state.focused_window_id = Some(window_label.clone());
                }
                if let Some(w_state) = app_state.windows.get_mut(&window_label) {
                    w_state.is_focused = *focused;
                }
                let _ = app_state.save_to_disk();
            }
            tauri::WindowEvent::Resized(size) => {
                log::debug!("Window {} resized to {:?}", window_label, size);
                if let Some(w) = app_handle.get_window(&window_label) {
                    let webviews = w.webviews();
                    for webview in webviews {
                        let _ = webview.set_size(*size);
                    }
                }
                let app_state_lock =
                    app_handle.state::<Arc<tokio::sync::Mutex<crate::state::AppState>>>();
                let mut app_state = app_state_lock.blocking_lock();
                if let Some(w_state) = app_state.windows.get_mut(&window_label) {
                    w_state.bounds.width = size.width as f64;
                    w_state.bounds.height = size.height as f64;
                }
                let _ = app_state.save_to_disk();
            }
            tauri::WindowEvent::Moved(position) => {
                log::debug!("Window {} moved to {:?}", window_label, position);
                let app_state_lock =
                    app_handle.state::<Arc<tokio::sync::Mutex<crate::state::AppState>>>();
                let mut app_state = app_state_lock.blocking_lock();
                if let Some(w_state) = app_state.windows.get_mut(&window_label) {
                    w_state.bounds.x = Some(position.x as f64);
                    w_state.bounds.y = Some(position.y as f64);
                }
                let _ = app_state.save_to_disk();
            }
            _ => {}
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
                    return Err(tauri::Error::AssetNotFound(
                        "TabOrchestrator state timeout".into(),
                    ));
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        };

        let config = app.state::<crate::config::LotionConfig>();
        let mut tabs_restored = false;

        if config.restore_tabs {
            let app_state_lock = app.state::<Arc<tokio::sync::Mutex<crate::state::AppState>>>();
            let mut app_state = app_state_lock.blocking_lock();

            // Find state for THIS window
            let window_label = self.window.label();
            if let Some(window_state) = app_state.windows.get_mut(window_label) {
                log::info!(
                    "WindowController: Restoring {} tabs from saved state.",
                    window_state.tab_ids.len()
                );
                let old_tab_ids = window_state.tab_ids.clone();
                window_state.tab_ids.clear();

                for old_id in &old_tab_ids {
                    if let Some(tab_state) = app_state.tabs.get(old_id) {
                        let url = tab_state.url.clone();
                        // Drop the borrow of app_state so we can call create_tab (which might use it)
                        // and re-borrow window_state to update it.
                        // Actually, create_tab doesn't need a lock on app_state, but we need to update window_state.
                        let new_tab_id = tab_manager.create_tab(app, window_label, &url)?;

                        // Re-fetch window_state to avoid borrow conflict
                        if let Some(ws) = app_state.windows.get_mut(window_label) {
                            ws.tab_ids.push(new_tab_id.clone());
                        }
                        let _ = tab_manager.show_tab(&new_tab_id);
                        tabs_restored = true;
                    }
                }
            }
        }

        if !tabs_restored {
            let notion_url = "https://www.notion.so";
            log::info!(
                "WindowController: Creating initial tab for Notion: {}",
                notion_url
            );
            let tab_id = tab_manager.create_tab(app, self.window.label(), notion_url)?;

            let app_state_lock = app.state::<Arc<tokio::sync::Mutex<crate::state::AppState>>>();
            let mut app_state = app_state_lock.blocking_lock();
            if let Some(window_state) = app_state.windows.get_mut(self.window.label()) {
                window_state.tab_ids.push(tab_id.clone());
            }

            let _ = tab_manager.show_tab(&tab_id);
        }

        Ok(())
    }
}
