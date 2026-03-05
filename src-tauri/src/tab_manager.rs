use crate::litebox::LiteBox;
use crate::tab_controller::TabController;
use crate::traits::TabOrchestrator;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Manager};

impl TabOrchestrator for TabManager {
    fn create_tab(&self, app: &AppHandle, window_id: &str, url: &str) -> tauri::Result<String> {
        self.create_tab(app, window_id, url)
    }

    fn destroy_tab(&self, tab_id: &str) -> tauri::Result<()> {
        self.destroy_tab(tab_id)
    }

    fn show_tab(&self, tab_id: &str) -> tauri::Result<()> {
        let tabs = self
            .tabs
            .read()
            .expect("TabManager: tabs read lock poisoned");
        for (id, tab) in tabs.iter() {
            if id == tab_id {
                tab.show()?;
            } else {
                tab.hide()?;
            }
        }
        Ok(())
    }

    fn get_tab_ids(&self) -> Vec<String> {
        self.tabs
            .read()
            .expect("TabManager: tabs read lock poisoned")
            .keys()
            .cloned()
            .collect()
    }

    fn inject_theme_into_tab(
        &self,
        app: &AppHandle,
        tab_id: &str,
        theme_name: &str,
    ) -> tauri::Result<()> {
        if let Some(tab) = self
            .tabs
            .read()
            .expect("TabManager: tabs read lock poisoned")
            .get(tab_id)
        {
            let theming = app.state::<Arc<dyn crate::traits::ThemingEngine>>();
            theming.inject_theme(&tab.webview, theme_name);
        }
        Ok(())
    }
}

pub struct TabManager {
    pub tabs: RwLock<HashMap<String, Arc<TabController>>>,
    pub litebox: Arc<LiteBox>,
}

impl TabManager {
    pub fn new(litebox: Arc<LiteBox>) -> Self {
        Self {
            tabs: RwLock::new(HashMap::new()),
            litebox,
        }
    }

    pub fn create_tab(&self, app: &AppHandle, window_id: &str, url: &str) -> tauri::Result<String> {
        let tab_id = uuid::Uuid::new_v4().to_string();

        let tab_controller =
            TabController::new(app, window_id, tab_id.clone(), url, self.litebox.clone())?;

        self.tabs
            .write()
            .expect("TabManager: tabs write lock poisoned")
            .insert(tab_id.clone(), Arc::new(tab_controller));

        log::info!("TabManager: Created tab {}", tab_id);
        Ok(tab_id)
    }

    pub fn get_tab(&self, tab_id: &str) -> Option<String> {
        self.tabs
            .read()
            .expect("TabManager: tabs read lock poisoned")
            .get(tab_id)
            .map(|t| t.tab_id.clone())
    }

    pub fn destroy_tab(&self, tab_id: &str) -> tauri::Result<()> {
        if let Some(tab) = self
            .tabs
            .write()
            .expect("TabManager: tabs write lock poisoned")
            .remove(tab_id)
        {
            tab.destroy()?;
        }
        Ok(())
    }
}
