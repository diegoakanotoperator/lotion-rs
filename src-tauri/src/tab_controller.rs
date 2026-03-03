use tauri::{AppHandle, Manager, WebviewBuilder, WebviewUrl, Webview};
use std::sync::Arc;
use crate::litebox::LiteBox;
use crate::litebox::host::HostPlatform;
use crate::traits::PolicyEnforcer;

pub struct TabController {
    pub tab_id: String,
    pub window_id: String,
    pub webview: Webview,
}

impl TabController {
    pub fn new(
        app: &AppHandle,
        window_id: &str,
        tab_id: String,
        url: &str,
        litebox: Arc<LiteBox<HostPlatform>>,
    ) -> tauri::Result<Self> {
        let policy = app.state::<Arc<dyn PolicyEnforcer>>();
        
        // Zero-Trust Enforcement: Validate URL before creation
        if !policy.validate_url(url) {
            return Err(tauri::Error::AssetNotFound(format!("Zero-Trust Policy Blocked: {}", url)));
        }

        let window = app.get_webview_window(window_id).ok_or(
            tauri::Error::AssetNotFound(format!("Window {} not found", window_id))
        )?;

        // Create a new webview for this tab
        let mut webview_builder = WebviewBuilder::new(&tab_id, WebviewUrl::App(url.parse().unwrap()));
        
        // Zero-Trust: Intercept all navigation requests
        let policy_cloned = policy.clone();
        webview_builder = webview_builder.on_navigation(move |url| {
            let url_str = url.as_str();
            if policy_cloned.validate_url(url_str) {
                true // Allow internal navigation to Notion
            } else if policy_cloned.validate_external_link(url_str) {
                log::info!("Zero-Trust: Opening validated external link in default browser: {}", url_str);
                let _ = opener::open(url_str);
                false // Block navigation in the webview
            } else {
                log::warn!("Zero-Trust: BLOCKED unauthorized navigation attempt to: {}", url_str);
                false // Block everything else
            }
        });

        let webview = window.add_child(
            webview_builder,
            tauri::LogicalPosition::new(0.0, 32.0), // Below tab bar
            tauri::LogicalSize::new(window.inner_size().unwrap().width as f64, (window.inner_size().unwrap().height as f64) - 32.0),
        )?;

        log::info!("Created tab webview: {} in window: {}", tab_id, window_id);

        // Phase 3: Inject Theme
        let theming = app.state::<Arc<dyn ThemingEngine>>();
        theming.inject_theme(&webview, "dracula"); // TODO: Load from config

        Ok(Self {
            tab_id,
            window_id: window_id.to_string(),
            webview,
        })
    }

    pub fn load_url(&self, app: &AppHandle, url: &str) -> tauri::Result<()> {
        let policy = app.state::<Arc<dyn PolicyEnforcer>>();
        
        // Zero-Trust Enforcement: Validate URL before navigation
        if !policy.validate_url(url) {
            log::warn!("Zero-Trust Policy Blocked navigation to: {}", url);
            return Ok(()); // Fail closed (don't navigate)
        }

        log::info!("Tab {}: Loading URL {}", self.tab_id, url);
        self.webview.navigate(WebviewUrl::App(url.parse().unwrap()))?;
        Ok(())
    }

    pub fn show(&self) -> tauri::Result<()> {
        self.webview.show()?;
        Ok(())
    }

    pub fn hide(&self) -> tauri::Result<()> {
        self.webview.hide()?;
        Ok(())
    }

    pub fn destroy(self) -> tauri::Result<()> {
        log::info!("Destroying tab: {}", self.tab_id);
        self.webview.close()?;
        Ok(())
    }
}
