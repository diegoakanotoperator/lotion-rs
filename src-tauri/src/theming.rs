use crate::traits::ThemingEngine;
use std::path::PathBuf;
use std::sync::RwLock;
use tauri::Webview;

pub struct ThemeManager {
    active_theme: RwLock<String>,
    custom_css_path: RwLock<Option<PathBuf>>,
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            active_theme: RwLock::new("dracula".to_string()),
            custom_css_path: RwLock::new(None),
        }
    }

    pub fn with_config(theme: &str, css_path: Option<PathBuf>) -> Self {
        Self {
            active_theme: RwLock::new(theme.to_string()),
            custom_css_path: RwLock::new(css_path),
        }
    }

    fn generate_notion_variables(&self, theme_name: &str) -> String {
        match theme_name {
            "dracula" => r#"
                :root.dark {
                    --theme--bg: #282a36;
                    --theme--fg: #44475a;
                    --theme--text: #f8f8f2;
                    --theme--text_ui: #6272a4;
                    --theme--text_ui_info: #8be9fd;
                    --theme--interactive: #bd93f9;
                    --theme--interactive_hover: #ff79c6;
                    --theme--divider: #6272a4;
                }
                .notion-sidebar, .notion-topbar { background: #21222c !important; }
            "#
            .to_string(),
            "nord" => r#"
                :root.dark {
                    --theme--bg: #2e3440;
                    --theme--fg: #3b4252;
                    --theme--text: #eceff4;
                    --theme--text_ui: #4c566a;
                    --theme--text_ui_info: #88c0d0;
                    --theme--interactive: #81a1c1;
                    --theme--interactive_hover: #88c0d0;
                    --theme--divider: #4c566a;
                }
            "#
            .to_string(),
            "default" | "light" => "".to_string(),
            _ => "".to_string(),
        }
    }
}

impl ThemingEngine for ThemeManager {
    fn get_theme_css(&self, theme_name: &str) -> String {
        let vars = self.generate_notion_variables(theme_name);
        if vars.is_empty() {
            return "".to_string();
        }
        format!("(function() {{
            const style = document.getElementById('lotion-theme-style') || document.createElement('style');
            style.id = 'lotion-theme-style';
            style.textContent = `{}`;
            if (!style.parentElement) document.head.appendChild(style);
        }})();", vars)
    }

    fn get_custom_css(&self) -> String {
        let path_guard = self
            .custom_css_path
            .read()
            .expect("ThemeManager: custom_css_path read lock poisoned");
        if let Some(path) = path_guard.as_ref() {
            if path.exists() {
                match std::fs::read_to_string(path) {
                    Ok(css) => {
                        log::info!("Loaded custom CSS from {}", path.display());
                        return format!("(function() {{
                            const style = document.getElementById('lotion-custom-style') || document.createElement('style');
                            style.id = 'lotion-custom-style';
                            style.textContent = `{}`;
                            if (!style.parentElement) document.head.appendChild(style);
                        }})();", css);
                    }
                    Err(e) => log::warn!("Failed to read custom CSS: {}", e),
                }
            }
        }
        "".to_string()
    }

    fn inject_theme(&self, webview: &Webview, theme_name: &str) {
        let js = self.get_theme_css(theme_name);
        if !js.is_empty() {
            let _ = webview.eval(&js);
        }
        // Also inject custom CSS if present
        let custom = self.get_custom_css();
        if !custom.is_empty() {
            let _ = webview.eval(&custom);
        }
    }

    fn set_active_theme(&self, name: &str) {
        let mut theme = self
            .active_theme
            .write()
            .expect("ThemeManager: active_theme write lock poisoned");
        *theme = name.to_string();
        log::info!("Active theme changed to: {}", name);
    }

    fn get_active_theme(&self) -> String {
        self.active_theme
            .read()
            .expect("ThemeManager: active_theme read lock poisoned")
            .clone()
    }
}
