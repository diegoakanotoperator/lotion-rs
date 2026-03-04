use lotion_rs::security::SecurityModule;
use lotion_rs::policy::PolicyManager;
use lotion_rs::theming::ThemeManager;

use lotion_rs::config::LotionConfig;
use lotion_rs::state::AppState;
use lotion_rs::i18n::I18nManager;
use lotion_rs::spellcheck::SpellcheckManager;
use std::sync::Arc;
use tauri::Manager;

#[tauri::command]
fn get_window_tabs(
    window_id: String,
    state: tauri::State<'_, Arc<tokio::sync::Mutex<AppState>>>,
) -> Vec<lotion_rs::state::TabState> {
    let app_state = state.blocking_lock();
    if let Some(w_state) = app_state.windows.get(&window_id) {
        w_state.tab_ids.iter()
            .filter_map(|id| app_state.tabs.get(id))
            .cloned()
            .collect()
    } else {
        Vec::new()
    }
}

#[tauri::command]
fn switch_tab(
    tab_id: String,
    orchestrator: tauri::State<'_, Arc<dyn lotion_rs::traits::TabOrchestrator>>,
) {
    let _ = orchestrator.show_tab(&tab_id);
}

#[tauri::command]
fn close_tab(
    tab_id: String,
    _app: tauri::AppHandle,
    orchestrator: tauri::State<'_, Arc<dyn lotion_rs::traits::TabOrchestrator>>,
    state: tauri::State<'_, Arc<tokio::sync::Mutex<AppState>>>,
) {
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
    let _ = app_state.save_to_disk();
}

#[tauri::command]
fn update_tab_state(
    tab_id: String,
    title: String,
    url: String,
    state: tauri::State<'_, Arc<tokio::sync::Mutex<AppState>>>,
) {
    let mut app_state = state.blocking_lock();
    
    // Update or Insert TabState
    app_state.tabs.insert(tab_id.clone(), lotion_rs::state::TabState {
        id: tab_id.clone(),
        title: title.clone(),
        url: url.clone(),
        is_active: true, // If it's sending updates, it's presumably the active one in its window
        is_pinned: false,
    });

    // Find which window this tab belongs to and update active_tab_id
    for window_state in app_state.windows.values_mut() {
        if window_state.tab_ids.contains(&tab_id) {
            window_state.active_tab_id = Some(tab_id.clone());
        }
    }

    let _ = app_state.save_to_disk();
    log::debug!("[lotion-state] Updated tab {} (title: {}, url: {})", tab_id, title, url);
}

#[tauri::command]
fn log_network_event(event: String) {
    log::info!("[lotion-net] {}", event);
}

fn main() {
    std::env::set_var("NO_AT_BRIDGE", "1");
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    std::env::set_var("WEBKIT_USE_SINGLE_WEB_PROCESS", "1");
    std::env::set_var("WEBKIT_DISABLE_ACCESSIBILITY", "1");
    std::env::set_var("GTK_A11Y", "none");
    std::env::set_var("GIO_USE_VFS", "local");

    // Set RUST_LOG only if not already set by the user
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    log::info!("Starting Lotion-rs...");

    // Load user config
    let config = LotionConfig::load();
    log::info!("Config: theme={}, restore_tabs={}", config.active_theme, config.restore_tabs);

    // Load saved state (if any)
    let app_state = AppState::load_from_disk().unwrap_or_default();
    let app_state = Arc::new(tokio::sync::Mutex::new(app_state));

    // Initialize Concrete Modules
    let security = Arc::new(SecurityModule::new());
    let policy = Arc::new(PolicyManager::new());
    let theming = Arc::new(ThemeManager::with_config(
        &config.active_theme,
        config.custom_css_path.clone(),
    ));
    let tab_manager = Arc::new(lotion_rs::tab_manager::TabManager::new(security.litebox.clone()));
    
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
            log_network_event
        ])
        .setup(move |app| {
            // Initialize modules in Tauri state FIRST as trait objects where expected
            app.manage::<Arc<dyn lotion_rs::traits::SecuritySandbox>>(security.litebox.clone());
            app.manage::<Arc<dyn lotion_rs::traits::PolicyEnforcer>>(policy);
            app.manage::<Arc<dyn lotion_rs::traits::ThemingEngine>>(theming);
            app.manage::<Arc<dyn lotion_rs::traits::TabOrchestrator>>(tab_manager);
            app.manage(config);
            app.manage(app_state);
            app.manage(I18nManager::new());
            app.manage(SpellcheckManager::new());

            let handle = app.handle().clone();
            
            // Native Menu Setup
            let _ = lotion_rs::menu::create_main_menu(&handle);

            let security_state = handle.state::<Arc<dyn lotion_rs::traits::SecuritySandbox>>().inner().clone();
            
            // Spawn the main window directly via Tauri WindowController
            match lotion_rs::window_controller::WindowController::new(&handle, security_state) {
                Ok(wc) => {
                    wc.setup_listeners(handle.clone());
                    if let Err(e) = wc.setup_tabs(&handle) {
                        log::error!("Failed to set up tabs: {}", e);
                    }
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
