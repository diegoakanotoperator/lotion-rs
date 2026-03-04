use tauri::{AppHandle, Manager, WebviewUrl, Url, WebviewBuilder};
use tauri::webview::Webview;
use std::sync::Arc;
use crate::litebox::LiteBox;
use crate::litebox::host::HostPlatform;
use crate::traits::{PolicyEnforcer, ThemingEngine};

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
        url_str: &str,
        _litebox: Arc<LiteBox<HostPlatform>>,
    ) -> tauri::Result<Self> {
        let policy = app.state::<Arc<dyn PolicyEnforcer>>().inner().clone();
        
        // Zero-Trust Enforcement: Validate URL before creation
        if !policy.validate_url(url_str) {
            return Err(tauri::Error::AssetNotFound(format!("Zero-Trust Policy Blocked: {}", url_str)));
        }

        let window = app.get_window(window_id).ok_or(
            tauri::Error::AssetNotFound(format!("Window {} not found", window_id))
        )?;

        let url = url_str.parse::<Url>().map_err(|e| tauri::Error::AssetNotFound(e.to_string()))?;

        // Create a new webview for this tab
        let mut webview_builder = WebviewBuilder::new(&tab_id, WebviewUrl::External(url.clone()));
        
        // Zero-Trust: Intercept all navigation requests
        let policy_cloned = policy.clone();
        let app_handle = app.clone();
        webview_builder = webview_builder
            .on_navigation({
                let app_handle = app_handle.clone();
                move |url| {
                    let url_str = url.as_str();
                    if policy_cloned.validate_url(url_str) {
                        true // Allow internal navigation to Notion
                    } else if policy_cloned.validate_external_link(url_str) {
                        log::info!("Zero-Trust: Opening validated external link in default browser: {}", url_str);
                        use tauri_plugin_shell::ShellExt;
                        #[allow(deprecated)]
                        let _ = app_handle.shell().open(url_str.to_string(), None);
                        false // Block navigation in the webview
                    } else {
                        log::warn!("Zero-Trust: BLOCKED unauthorized navigation attempt to: {}", url_str);
                        false // Block everything else
                    }
                }
            })
            .on_new_window({
                let app_handle = app_handle.clone();
                let policy_cloned = policy.clone();
                move |url, _| {
                    use tauri_plugin_shell::ShellExt;
                    let url_str = url.as_str();

                    if policy_cloned.should_route_popup_to_system_browser(url_str) {
                        log::info!("Routing new window request (popup) to system browser: {}", url_str);
                        #[allow(deprecated)]
                        let _ = app_handle.shell().open(url_str.to_string(), None);
                        tauri::webview::NewWindowResponse::Deny
                    } else {
                        // Manually create a controlled window for OAuth/Notion popups
                        // This ensures they are resizable, decorated, and handle window.close() correctly
                        let label = format!("popup-{}", uuid::Uuid::new_v4());
                        log::info!("Creating controlled in-app popup for: {}", url_str);
                        
                        let _ = tauri::WebviewWindowBuilder::new(&app_handle, label, tauri::WebviewUrl::External(url.clone()))
                            .title("Lotion Login")
                            .inner_size(800.0, 600.0)
                            .resizable(true)
                            .decorations(true)
                            .always_on_top(true)
                            .build();
                            
                        tauri::webview::NewWindowResponse::Deny
                    }
                }
            });

        let webview = window.add_child(
            webview_builder,
            tauri::LogicalPosition::new(0.0, 32.0),
            tauri::LogicalSize::new(
                window.inner_size().unwrap().width as f64, 
                (window.inner_size().unwrap().height as f64) - 32.0
            )
        )?;

        log::info!("Created tab webview: {} in window: {}", tab_id, window_id);

        // Inject theme from config (not hardcoded)
        let theming = app.state::<Arc<dyn ThemingEngine>>();
        let active_theme = theming.get_active_theme();
        theming.inject_theme(&webview, &active_theme);

        // Inject title observer — watches for document.title changes and logs them
        let title_observer_js = format!(r#"
            (function() {{
                const tabId = '{}';
                let lastTitle = document.title;
                const observer = new MutationObserver(function() {{
                    if (document.title !== lastTitle) {{
                        lastTitle = document.title;
                        console.log('[lotion-title-sync] tab=' + tabId + ' title=' + lastTitle);
                        window.__TAURI__.invoke('update_tab_title', {{ tabId: tabId, title: lastTitle }});
                    }}
                }});
                observer.observe(document.querySelector('title') || document.head, {{
                    subtree: true,
                    characterData: true,
                    childList: true
                }});
            }})();
        "#, tab_id);
        let _ = webview.eval(&title_observer_js);

        // Inject network monitor — intercepts fetch and XHR to log status/errors
        let network_monitor_js = r#"
            (function() {
                const log = (msg) => {
                    console.log(msg);
                    if (window.__TAURI__) {
                        window.__TAURI__.invoke('log_network_event', { event: msg });
                    }
                };

                // Monitor Fetch
                const originalFetch = window.fetch;
                window.fetch = async (...args) => {
                    const url = args[0] instanceof Request ? args[0].url : args[0];
                    try {
                        const response = await originalFetch(...args);
                        log(`FETCH SUCCESS: ${response.status} ${url}`);
                        return response;
                    } catch (error) {
                        log(`FETCH ERROR: ${url} - ${error.message}`);
                        throw error;
                    }
                };

                // Monitor XHR
                const originalOpen = XMLHttpRequest.prototype.open;
                XMLHttpRequest.prototype.open = function(method, url) {
                    this._url = url;
                    this.addEventListener('load', function() {
                        log(`XHR SUCCESS: ${this.status} ${this._url}`);
                    });
                    this.addEventListener('error', function() {
                        log(`XHR ERROR: ${this._url}`);
                    });
                    return originalOpen.apply(this, arguments);
                };
                log('Network monitoring active.');
            })();
        "#;
        let _ = webview.eval(network_monitor_js);

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
        let url = url.parse::<Url>().map_err(|e| tauri::Error::AssetNotFound(e.to_string()))?;
        self.webview.navigate(url)?;
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
