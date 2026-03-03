use crate::traits::ThemingEngine;
use tauri::Webview;

pub struct ThemeManager {
    // We could store theme definitions here
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {}
    }

    fn generate_notion_variables(&self, theme_name: &str) -> String {
        // Ported from legacy NOTION_ENHANCER_INTEGRATION.md
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
            "#.to_string(),
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
            "#.to_string(),
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
        // Placeholder for user's custom.css
        "".to_string()
    }

    fn inject_theme(&self, webview: &Webview, theme_name: &str) {
        let js = self.get_theme_css(theme_name);
        if !js.is_empty() {
            let _ = webview.eval(&js);
        }
    }
}
