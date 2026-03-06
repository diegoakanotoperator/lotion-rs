use crate::litebox::LiteBox;
use crate::traits::{PolicyEnforcer, ThemingEngine};
use std::sync::Arc;
use tauri::webview::Webview;
use tauri::{AppHandle, Manager, Url, WebviewBuilder, WebviewUrl};

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
        _litebox: Arc<LiteBox>,
    ) -> tauri::Result<Self> {
        let policy = app.state::<Arc<dyn PolicyEnforcer>>().inner().clone();

        // Zero-Trust Enforcement: Validate URL before creation
        if !policy.validate_url(url_str) {
            return Err(tauri::Error::AssetNotFound(format!(
                "Zero-Trust Policy Blocked: {}",
                url_str
            )));
        }

        let window = app
            .get_window(window_id)
            .ok_or(tauri::Error::AssetNotFound(format!(
                "Window {} not found",
                window_id
            )))?;

        let url = url_str
            .parse::<Url>()
            .map_err(|e| tauri::Error::AssetNotFound(e.to_string()))?;

        // Create a new webview for this tab
        let mut webview_builder = WebviewBuilder::new(&tab_id, WebviewUrl::External(url.clone()));

        let nav_app = app.clone();
        let nav_policy = policy.clone();
        let popup_app = app.clone();
        let popup_policy = policy.clone();

        let window_id_clone = window_id.to_string();
        webview_builder = webview_builder
            .on_navigation(move |url| {
                let window_id = &window_id_clone;
                let url_str = url.as_str();

                // Intercept custom window control actions
                if url_str.starts_with("lotion-action://") {
                    let action = url_str.strip_prefix("lotion-action://").unwrap_or("");
                    log::info!("Intercepted Lotion action: {}", action);

                    if let Some(w) = nav_app.get_window(window_id) {
                        match action {
                            "window:close" => {
                                let _ = w.close();
                            }
                            "window:minimize" => {
                                let _ = w.minimize();
                            }
                            "window:maximize" => {
                                if let Ok(true) = w.is_maximized() {
                                    let _ = w.unmaximize();
                                } else {
                                    let _ = w.maximize();
                                }
                            }
                            "tab:new" => {
                                let notion_url = "https://www.notion.so";
                                if let Some(orchestrator) = nav_app.try_state::<Arc<dyn crate::traits::TabOrchestrator>>() {
                                    if let Ok(new_id) = orchestrator.inner().create_tab(&nav_app, window_id, notion_url) {
                                        let _ = orchestrator.inner().show_tab(&new_id);

                                        // Update AppState
                                        if let Some(state_lock) = nav_app.try_state::<Arc<tokio::sync::Mutex<crate::state::AppState>>>() {
                                            let mut app_state = state_lock.blocking_lock();
                                            if let Some(w_state) = app_state.windows.get_mut(window_id) {
                                                w_state.tab_ids.push(new_id);
                                                let _ = app_state.save_to_disk();
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {
                                log::warn!("Unknown Lotion action: {}", action);
                            }
                        }
                    }
                    return false; // Prevent actual navigation
                }

                if nav_policy.validate_url(url_str) {
                    true // Allow internal navigation to Notion
                } else if nav_policy.validate_external_link(url_str) {
                    log::info!("Zero-Trust: Opening validated external link in default browser: {}", url_str);
                    use tauri_plugin_shell::ShellExt;
                    #[allow(deprecated)]
                    let _ = nav_app.shell().open(url_str.to_string(), None);
                    false // Block navigation in the webview
                } else {
                    log::warn!("Zero-Trust: BLOCKED unauthorized navigation attempt to: {}", url_str);
                    false // Block everything else
                }
            })
            .on_new_window(move |url, _| {
                use tauri_plugin_shell::ShellExt;
                let url_str = url.as_str();

                if popup_policy.should_route_popup_to_system_browser(url_str) {
                    log::info!("Routing new window request (popup) to system browser: {}", url_str);
                    #[allow(deprecated)]
                    let _ = popup_app.shell().open(url_str.to_string(), None);
                    tauri::webview::NewWindowResponse::Deny
                } else {
                    // Spawn a controlled popup using the recursive secure factory
                    spawn_secure_popup(&popup_app, popup_policy.clone(), url.clone());
                    tauri::webview::NewWindowResponse::Deny
                }
            });

        let inner_size = window.inner_size()?;
        let webview = window.add_child(
            webview_builder,
            tauri::LogicalPosition::new(0.0, 0.0),
            tauri::LogicalSize::new(inner_size.width as f64, inner_size.height as f64),
        )?;

        log::info!("Created tab webview: {} in window: {}", tab_id, window_id);

        // Inject theme from config (not hardcoded)
        let theming = app.state::<Arc<dyn ThemingEngine>>();
        let active_theme = theming.get_active_theme();
        theming.inject_theme(&webview, &active_theme);

        // Inject title observer and custom Mac-style Window Controls
        let title_observer_js = format!(
            r#"
            (function() {{
                const tabId = '{}';

                // 1. Title Observer
                let lastTitle = document.title;
                const observer = new MutationObserver(function() {{
                    if (document.title !== lastTitle) {{
                        lastTitle = document.title;
                        if (window.__TAURI__) {{
                            window.__TAURI__.invoke('update_tab_state', {{
                                tabId: tabId,
                                title: lastTitle,
                                url: window.location.href
                            }});
                        }}
                    }}
                }});
                observer.observe(document.querySelector('title') || document.head, {{
                    subtree: true, characterData: true, childList: true
                }});

                // 2. Inject Native-feeling Window Controls (Titlebar)
                window.addEventListener('DOMContentLoaded', () => {{
                    const titlebar = document.createElement('div');
                    titlebar.id = 'lotion-custom-titlebar';
                    titlebar.setAttribute('data-tauri-drag-region', '');
                    titlebar.style.cssText = `
                        position: fixed;
                        top: 0;
                        left: 0;
                        width: 100%;
                        height: 38px;
                        z-index: 999999;
                        display: flex;
                        align-items: center;
                        padding-left: 12px;
                        pointer-events: none;
                        background: inherit;
                        border-bottom: 1px solid rgba(0,0,0,0.05);
                    `;

                    // Push the Notion sidebar down slightly so it doesn't overlap the buttons
                    const style = document.createElement('style');
                    style.textContent = `
                        .notion-sidebar-container {{ margin-top: 38px !important; }}
                        .notion-topbar {{ padding-left: 80px !important; }}
                        .lotion-tab {{
                            padding: 4px 12px;
                            font-size: 12px;
                            border-radius: 6px 6px 0 0;
                            cursor: pointer;
                            display: flex;
                            align-items: center;
                            gap: 8px;
                            max-width: 150px;
                            overflow: hidden;
                            white-space: nowrap;
                            text-overflow: ellipsis;
                            background: rgba(0,0,0,0.05);
                            border: 1px solid rgba(0,0,0,0.1);
                            border-bottom: none;
                            pointer-events: auto;
                        }}
                        .lotion-tab.active {{
                            background: white;
                            font-weight: 500;
                        }}
                        .lotion-tab-close {{
                            font-size: 14px;
                            opacity: 0.5;
                            transition: opacity 0.2s;
                        }}
                        .lotion-tab-close:hover {{
                            opacity: 1;
                        }}
                    `;
                    document.head.appendChild(style);

                    const btnContainer = document.createElement('div');
                    btnContainer.style.cssText = `
                        display: flex;
                        gap: 8px;
                        align-items: center;
                        pointer-events: auto;
                    `;

                    const createBtn = (color, clickHandler, label = "") => {{
                        const btn = document.createElement('div');
                        btn.style.cssText = `
                            width: 12px; height: 12px;
                            border-radius: 50%;
                            background-color: ${{color}};
                            cursor: pointer;
                            border: 1px solid rgba(0,0,0,0.1);
                            display: flex; align-items: center; justify-content: center;
                            font-size: 8px; font-family: sans-serif;
                        `;
                        if (label) btn.innerText = label;
                        btn.addEventListener('click', (e) => {{
                            e.stopPropagation();
                            clickHandler();
                        }});
                        return btn;
                    }};

                    const closeBtn = createBtn('#ff5f56', () => {{
                        if (window.__TAURI__) {{
                            window.__TAURI__.invoke('close_window', {{ windowId: '${window_id}' }});
                        }}
                    }});

                    const minBtn = createBtn('#ffbd2e', () => {{
                        if (window.__TAURI__) {{
                            window.__TAURI__.invoke('minimize_window', {{ windowId: '${window_id}' }});
                        }}
                    }});

                    const maxBtn = createBtn('#27c93f', () => {{
                        if (window.__TAURI__) {{
                            window.__TAURI__.invoke('maximize_window', {{ windowId: '${window_id}' }});
                        }}
                    }});


                    btnContainer.appendChild(closeBtn);
                    btnContainer.appendChild(minBtn);
                    btnContainer.appendChild(maxBtn);

                    const spacer = document.createElement('div');
                    spacer.style.width = '24px';
                    btnContainer.appendChild(spacer);

                    const tabList = document.createElement('div');
                    tabList.style.cssText = `
                        display: flex;
                        gap: 4px;
                        align-items: flex-end;
                        height: 100%;
                        padding-top: 8px;
                        pointer-events: auto;
                    `;

                    const renderTabs = async () => {{
                        if (!window.__TAURI__) return;
                        const tabs = await window.__TAURI__.invoke('get_window_tabs', {{ windowId: '{}' }});
                        tabList.innerHTML = '';
                        tabs.forEach(t => {{
                            const tabEl = document.createElement('div');
                            tabEl.className = 'lotion-tab' + (t.id === tabId ? ' active' : '');
                            tabEl.innerText = t.title || 'Notion';

                            const closeX = document.createElement('span');
                            closeX.className = 'lotion-tab-close';
                            closeX.innerText = ' ×';
                            closeX.onclick = (e) => {{
                                e.stopPropagation();
                                window.__TAURI__.invoke('close_tab', {{ tabId: t.id }});
                            }};

                            tabEl.appendChild(closeX);
                            tabEl.onclick = () => {{
                                if (t.id !== tabId) {{
                                    window.__TAURI__.invoke('switch_tab', {{ tabId: t.id }});
                                }}
                            }};
                            tabList.appendChild(tabEl);
                        }});

                        const newTab = createBtn('#27c93f', () => {{
                            window.location.href = 'lotion-action://tab:new';
                        }}, '+');
                        newTab.style.marginLeft = '8px';
                        newTab.style.marginBottom = '6px';
                        tabList.appendChild(newTab);
                    }};

                    btnContainer.appendChild(tabList);
                    titlebar.appendChild(btnContainer);
                    document.body.appendChild(titlebar);

                    renderTabs();
                    // Poll for tab changes (simple for now)
                    setInterval(renderTabs, 5000);
                }});
            }})();
        "#,
            tab_id, window_id
        );
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
        let url = url
            .parse::<Url>()
            .map_err(|e| tauri::Error::AssetNotFound(e.to_string()))?;
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

    pub fn destroy(&self) -> tauri::Result<()> {
        log::info!("Destroying tab: {}", self.tab_id);
        self.webview.close()?;
        Ok(())
    }
}

/// Routes secure popup requests into the application's internal TabManager.
/// Guaranteeing that any nested popups (e.g. nested OAuth flows) inherit
/// the exact same zero-trust `on_navigation` and `on_new_window` policies
/// as their parent window via the TabController factory.
pub fn spawn_secure_popup(app: &AppHandle, _policy: Arc<dyn PolicyEnforcer>, url: Url) {
    log::info!(
        "Intercepted popup request. Routing into a secure in-app tab: {}",
        url.as_str()
    );

    // Instead of spawning a completely disconnected OS window, dispatch the popup
    // into our managed TabOrchestrator. This keeps the application bounded strictly
    // to a single window and enforces all Zero-Trust policies recursively since
    // create_tab() uses the TabController factory.
    if let Some(orchestrator) = app.try_state::<Arc<dyn crate::traits::TabOrchestrator>>() {
        if let Err(e) = orchestrator.inner().create_tab(app, "main", url.as_str()) {
            log::error!("Zero-Trust: Failed to route popup into managed tab: {}", e);
        }
    } else {
        log::error!("Zero-Trust: Cannot spawn tab securely. TabOrchestrator missing from state.");
    }
}
