use tauri::AppHandle;

/// Interface for security sandboxing
pub trait SecuritySandbox: Send + Sync {
    fn initialize(&self);
    fn get_fd_count(&self) -> usize;
}

/// Interface for tab navigation and orchestration
pub trait TabOrchestrator: Send + Sync {
    fn create_tab(&self, app: &AppHandle, window_id: &str, url: &str) -> tauri::Result<String>;
    fn destroy_tab(&self, tab_id: &str) -> tauri::Result<()>;
}

/// Interface for window management
pub trait WindowProvider: Send + Sync {
    fn create_main_window(&self, app: &AppHandle) -> tauri::Result<()>;
}

/// Interface for enforcing Zero-Trust Manifesto policies
pub trait PolicyEnforcer: Send + Sync {
    /// Returns true if navigation to the URL is allowed by the manifesto
    fn validate_url(&self, url: &str) -> bool;
    /// Policy on whether telemetry is allowed (defaults to false per Manifesto)
    fn telemetry_allowed(&self) -> bool;
    /// Returns true if an external link is safe to open in the system browser
    fn validate_external_link(&self, url: &str) -> bool;
}

/// Interface for the CSS/JS Theming Engine
pub trait ThemingEngine: Send + Sync {
    /// Returns the CSS string to be injected for a given theme name
    fn get_theme_css(&self, theme_name: &str) -> String;
    /// Returns the CSS to apply custom user overrides
    fn get_custom_css(&self) -> String;
    /// Injects the current theme into a webview
    fn inject_theme(&self, webview: &tauri::Webview, theme_name: &str);
}
