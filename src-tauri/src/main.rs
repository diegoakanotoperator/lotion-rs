use lotion_rs::policy::PolicyManager;
use lotion_rs::security::SecurityModule;
use lotion_rs::theming::ThemeManager;

use lotion_rs::config::LotionConfig;
use lotion_rs::i18n::I18nManager;
use lotion_rs::spellcheck::SpellcheckManager;
use lotion_rs::state::AppState;
use std::sync::Arc;
use tauri::Manager;
use std::fs::{self, File};
use std::io::{Read, Write};
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt; // Specific import for unix permissions
use rand::RngCore;

const SECRET_FILE_NAME: &str = "secret_key";

// Helper function to get or create the application secret
fn get_or_create_app_secret() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let secret_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("lotion-rs");
    let secret_path = secret_dir.join(SECRET_FILE_NAME);

    if secret_path.exists() {
        let mut file = File::open(&secret_path)?;
        let mut secret = vec![0u8; 32];
        file.read_exact(&mut secret)?;
        log::info!("Application secret loaded from {}", secret_path.display());
        Ok(secret)
    } else {
        log::info!("Generating new application secret at {}", secret_path.display());
        fs::create_dir_all(&secret_dir)?;
        
        let mut secret = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut secret);

        let mut file = File::create(&secret_path)?;
        #[cfg(target_family = "unix")]
        {
            // Set permissions to 0o600 (read/write only for owner)
            file.set_permissions(fs::Permissions::from_mode(0o600))?;
        }
        file.write_all(&secret)?;
        Ok(secret)
    }
}

// Helper function to check if the command invocation origin is trusted
fn is_trusted_origin<R: tauri::Runtime>(webview: &tauri::Webview<R>) -> bool {
    // In a real application, this list of trusted origins would be configurable
    // and potentially loaded from a secure source.
    let trusted_origins = vec![
        "https://www.notion.so",
        "https://lotion.app", // Example of a self-controlled origin
        "tauri://localhost", // For local development/devtools
    ];

    if let Ok(url) = webview.url() {
        let origin = format!("{}://{}", url.scheme(), url.host_str().unwrap_or_default());
        let is_trusted = trusted_origins.iter().any(|o| origin.starts_with(o));
        if !is_trusted {
            log::warn!("Untrusted origin: {} attempted to invoke command.", origin);
        }
        is_trusted
    } else {
        log::warn!("Could not determine origin for command invocation (webview URL not found).");
        false
    }
} // <--- Added this closing brace

#[tauri::command]
fn get_window_tabs(
    webview: tauri::Webview<tauri::Wry>,
    window_id: String,
    state: tauri::State<'_, Arc<tokio::sync::Mutex<AppState>>>,
) -> Vec<lotion_rs::state::TabState> {
    if !is_trusted_origin(&webview) {
        return Vec::new(); // Deny access for untrusted origins
    }
    log::info!("get_window_tabs called from origin: {:?}", webview.url());
    let app_state = state.blocking_lock();
    if let Some(w_state) = app_state.windows.get(&window_id) {
        w_state
            .tab_ids
            .iter()
            .filter_map(|id| app_state.tabs.get(id))
            .cloned()
            .collect()
    } else {
        Vec::new()
    }
}

#[tauri::command]
fn switch_tab(
    webview: tauri::Webview<tauri::Wry>,
    tab_id: String,
    orchestrator: tauri::State<'_, Arc<dyn lotion_rs::traits::TabOrchestrator<tauri::Wry>>>,
) {
    if !is_trusted_origin(&webview) {
        return; // Deny access for untrusted origins
    }
    let _ = orchestrator.show_tab(&tab_id);
}

#[tauri::command]
fn close_tab(
    webview: tauri::Webview<tauri::Wry>,
    tab_id: String,
    _app: tauri::AppHandle<tauri::Wry>,
    orchestrator: tauri::State<'_, Arc<dyn lotion_rs::traits::TabOrchestrator<tauri::Wry>>>,
    state: tauri::State<'_, Arc<tokio::sync::Mutex<AppState>>>,
    app_secret_state: tauri::State<'_, Arc<Vec<u8>>>,
) {
    if !is_trusted_origin(&webview) {
        return; // Deny access for untrusted origins
    }
    let _ = orchestrator.destroy_tab(&tab_id);

    let mut app_state = state.blocking_lock();
    app_state.tabs.remove(&tab_id);
    for window_state in app_state.windows.values_mut() {
        window_state.tab_ids.retain(|id| id != &tab_id);
        if window_state.active_tab_id.as_ref() == Some(&tab_id) {
            window_state.active_tab_id = window_state.tab_ids.last().cloned();
            if let Some(ref next_id) = window_state.active_tab_id {
                let _ = orchestrator.show_tab(next_id);
            }
        }
    }
    let _ = app_state.save_to_disk(app_secret_state.inner().as_slice());
}

#[tauri::command]
fn new_tab(
    webview: tauri::Webview<tauri::Wry>,
    window_id: String,
    app: tauri::AppHandle<tauri::Wry>,
    orchestrator: tauri::State<'_, Arc<dyn lotion_rs::traits::TabOrchestrator<tauri::Wry>>>,
    state: tauri::State<'_, Arc<tokio::sync::Mutex<AppState>>>,
    app_secret_state: tauri::State<'_, Arc<Vec<u8>>>,
) {
    if !is_trusted_origin(&webview) {
        return; // Deny access for untrusted origins
    }
    let notion_url = "https://www.notion.so";
    if let Ok(new_id) = orchestrator.create_tab(&app, &window_id, notion_url) {
        let _ = orchestrator.show_tab(&new_id);

        let mut app_state = state.blocking_lock();
        if let Some(w_state) = app_state.windows.get_mut(&window_id) {
            w_state.tab_ids.push(new_id);
            let _ = app_state.save_to_disk(app_secret_state.inner().as_slice());
        }
    }
}

#[tauri::command]
fn update_tab_state(
    webview: tauri::Webview<tauri::Wry>,
    tab_id: String,
    title: String,
    url: String,
    state: tauri::State<'_, Arc<tokio::sync::Mutex<AppState>>>,
    app_secret_state: tauri::State<'_, Arc<Vec<u8>>>,
) {
    if !is_trusted_origin(&webview) {
        return; // Deny access for untrusted origins
    }

    // Additional validation: Ensure the URL provided by the webview matches the actual webview URL.
    if let Ok(webview_url) = webview.url() {
        if webview_url.as_str() != url {
            log::warn!(
                "Origin {} attempted to update tab state with mismatched URL. Provided: {}, Actual: {}",
                webview_url,
                url,
                webview_url.as_str()
            );
            return;
        }
    } else {
        log::warn!("Could not determine webview URL for origin validation in update_tab_state.");
        return;
    }

    let mut app_state = state.blocking_lock();

    // Update or Insert TabState
    app_state.tabs.insert(
        tab_id.clone(),
        lotion_rs::state::TabState {
            id: tab_id.clone(),
            title: title.clone(),
            url: url.clone(),
            is_active: true, // If it's sending updates, it's presumably the active one in its window
            is_pinned: false,
        },
    );

    // Find which window this tab belongs to and update active_tab_id
    for window_state in app_state.windows.values_mut() {
        if window_state.tab_ids.contains(&tab_id) {
            window_state.active_tab_id = Some(tab_id.clone());
        }
    }

    let _ = app_state.save_to_disk(app_secret_state.inner().as_slice());
    log::debug!(
        "[lotion-state] Updated tab {} (title: {}, url: {})",
        tab_id,
        title,
        url
    );
}

#[tauri::command]
fn minimize_window(webview: tauri::Webview<tauri::Wry>, window_id: String, app: tauri::AppHandle<tauri::Wry>) {
    if !is_trusted_origin(&webview) {
        return; // Deny access for untrusted origins
    }
    if let Some(window) = app.get_window(&window_id) {
        let _ = window.minimize();
    }
}

#[tauri::command]
fn maximize_window(webview: tauri::Webview<tauri::Wry>, window_id: String, app: tauri::AppHandle<tauri::Wry>) {
    if !is_trusted_origin(&webview) {
        return; // Deny access for untrusted origins
    }
    if let Some(window) = app.get_window(&window_id) {
        if let Ok(true) = window.is_maximized() {
            let _ = window.unmaximize();
        } else {
            let _ = window.maximize();
        }
    }
}

#[tauri::command]
fn close_window(webview: tauri::Webview<tauri::Wry>, window_id: String, app: tauri::AppHandle<tauri::Wry>) {
    if !is_trusted_origin(&webview) {
        return; // Deny access for untrusted origins
    }
    if let Some(window) = app.get_window(&window_id) {
        let _ = window.close();
    }
}

#[tauri::command]
fn log_network_event(webview: tauri::Webview<tauri::Wry>, event: String) {
    if !is_trusted_origin(&webview) {
        return; // Deny access for untrusted origins
    }
    // Truncate event to prevent log spamming or excessive memory usage
    let truncated_event = if event.len() > 512 {
        format!("{}...", &event[..512])
    } else {
        event
    };
    log::info!("[lotion-net] {}", truncated_event);
}

fn main() {
    #[cfg(target_os = "linux")]
    {
        std::env::set_var("NO_AT_BRIDGE", "1");
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        std::env::set_var("WEBKIT_USE_SINGLE_WEB_PROCESS", "1");
        std::env::set_var("WEBKIT_DISABLE_ACCESSIBILITY", "1");
        std::env::set_var("GTK_A11Y", "none");
        std::env::set_var("GIO_USE_VFS", "local");
    }

    // Set RUST_LOG only if not already set by the user
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    log::info!("Starting Lotion-rs...");

    // Get or create application secret
    let app_secret = get_or_create_app_secret()
        .expect("Failed to get or create application secret");
    let app_secret_arc = Arc::new(app_secret);

    // Load user config
    let config = LotionConfig::load();
    log::info!(
        "Config: theme={}, restore_tabs={}",
        config.active_theme,
        config.restore_tabs
    );

    // Load saved state (if any)
    let app_state = AppState::load_from_disk(&app_secret_arc).unwrap_or_default();
    let app_state = Arc::new(tokio::sync::Mutex::new(app_state));

    // Initialize Concrete Modules
    let security = Arc::new(SecurityModule::new());
    let policy = Arc::new(PolicyManager::new());
    let theming = Arc::new(ThemeManager::with_config(
        &config.active_theme,
        config.custom_css_path.clone(),
    ));
    let tab_manager = Arc::new(lotion_rs::tab_manager::TabManager::<tauri::Wry>::new(
        security.litebox.clone(),
    ));

    // Tauri Application Context
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            lotion_rs::i18n::get_translation,
            lotion_rs::i18n::set_locale,
            lotion_rs::spellcheck::check_spelling,
            lotion_rs::spellcheck::get_spelling_suggestions,
            update_tab_state,
            get_window_tabs,
            switch_tab,
            close_tab,
            new_tab,
            minimize_window,
            maximize_window,
            close_window,
            log_network_event
        ])
        .setup(move |app| {
            // Initialize modules in Tauri state FIRST as trait objects where expected
            app.manage::<Arc<dyn lotion_rs::traits::SecuritySandbox>>(security.litebox.clone());
            app.manage::<Arc<dyn lotion_rs::traits::PolicyEnforcer>>(policy);
            app.manage::<Arc<dyn lotion_rs::traits::ThemingEngine<tauri::Wry>>>(theming);
            app.manage::<Arc<dyn lotion_rs::traits::TabOrchestrator<tauri::Wry>>>(tab_manager);
            app.manage(config);
            app.manage(app_state);
            app.manage(I18nManager::new());
            app.manage(SpellcheckManager::new());
            app.manage(app_secret_arc.clone()); // Manage the app_secret_arc

            let handle = app.handle().clone();

            // Native Menu Setup
            let _ = lotion_rs::menu::create_main_menu(&handle);

            let security_state = handle
                .state::<Arc<dyn lotion_rs::traits::SecuritySandbox>>()
                .inner()
                .clone();

            // Spawn the main window directly via Tauri WindowController
            match lotion_rs::window_controller::WindowController::<tauri::Wry>::new(&handle, security_state) {
                Ok(wc) => {
                    wc.setup_listeners(handle.clone());
                    let setup_handle = handle.clone();
                    std::thread::spawn(move || {
                        if let Err(e) = wc.setup_tabs(&setup_handle) {
                            log::error!("Failed to set up tabs: {}", e);
                        }
                    });
                    log::info!("WindowController initialized and set up.");
                }
                Err(e) => {
                    log::error!("Failed to create WindowController: {}", e);
                }
            }

            log::info!("Tauri background layer initialized.");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
