use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};
use crate::tab_controller::TabController;
use crate::litebox::LiteBox;
use crate::litebox::host::HostPlatform;
use crate::traits::TabOrchestrator;

impl TabOrchestrator for TabManager {
    fn create_tab(&self, app: &AppHandle, window_id: &str, url: &str) -> tauri::Result<String> {
        self.create_tab(app, window_id, url)
    }

    fn destroy_tab(&self, tab_id: &str) -> tauri::Result<()> {
        self.destroy_tab(tab_id)
    }

    fn get_tab_ids(&self) -> Vec<String> {
        let tabs = self.tabs.lock().unwrap();
        tabs.keys().cloned().collect()
    }

    fn inject_theme_into_tab(&self, app: &AppHandle, tab_id: &str, theme_name: &str) -> tauri::Result<()> {
        let tabs = self.tabs.lock().unwrap();
        if let Some(tab) = tabs.get(tab_id) {
            let theming = app.state::<Arc<dyn crate::traits::ThemingEngine>>();
            theming.inject_theme(&tab.webview, theme_name);
        }
        Ok(())
    }
}

pub struct TabManager {
    pub tabs: Mutex<HashMap<String, TabController>>,
    pub litebox: Arc<LiteBox<HostPlatform>>,
}

impl TabManager {
    pub fn new(litebox: Arc<LiteBox<HostPlatform>>) -> Self {
        Self {
            tabs: Mutex::new(HashMap::new()),
            litebox,
        }
    }

    pub fn create_tab(
        &self,
        app: &AppHandle,
        window_id: &str,
        url: &str,
    ) -> tauri::Result<String> {
        let tab_id = uuid::Uuid::new_v4().to_string();
        
        let tab_controller = TabController::new(
            app,
            window_id,
            tab_id.clone(),
            url,
            self.litebox.clone(),
        )?;

        let mut tabs = self.tabs.lock().unwrap();
        tabs.insert(tab_id.clone(), tab_controller);

        log::info!("TabManager: Created tab {}", tab_id);
        Ok(tab_id)
    }

    pub fn get_tab(&self, tab_id: &str) -> Option<String> {
        let tabs = self.tabs.lock().unwrap();
        tabs.get(tab_id).map(|t| t.tab_id.clone())
    }

    pub fn destroy_tab(&self, tab_id: &str) -> tauri::Result<()> {
        let mut tabs = self.tabs.lock().unwrap();
        if let Some(tab) = tabs.remove(tab_id) {
            tab.destroy()?;
        }
        Ok(())
    }
}
