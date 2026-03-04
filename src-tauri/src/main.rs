use lotion_rs::security::SecurityModule;
use lotion_rs::policy::PolicyManager;
use lotion_rs::ui::theming::ThemeManager;
use lotion_rs::ui::{self, Message};
use lotion_rs::config::LotionConfig;
use lotion_rs::state::AppState;
use lotion_rs::i18n::I18nManager;
use lotion_rs::spellcheck::SpellcheckManager;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::mpsc;

#[tauri::command]
fn log_network_event(event: String) {
    log::info!("[lotion-net] {}", event);
}

#[tauri::command]
fn update_tab_title(
    tab_id: String,
    title: String,
    tx: tauri::State<'_, mpsc::Sender<Message>>,
) {
    let _ = tx.blocking_send(Message::TabTitleUpdated(tab_id, title));
}

fn main() -> iced::Result {
    std::env::set_var("NO_AT_BRIDGE", "1");
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    std::env::set_var("WEBKIT_USE_SINGLE_WEB_PROCESS", "1");
    std::env::set_var("WEBKIT_DISABLE_ACCESSIBILITY", "1");
    std::env::set_var("GTK_A11Y", "none");
    std::env::set_var("GIO_USE_VFS", "local");
    std::env::set_var("WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS", "1");

    // Set RUST_LOG only if not already set by the user
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var(
            "RUST_LOG",
            "info,lotion_rs=trace,tauri=debug,wry=debug,wgpu=warn,iced=debug",
        );
    }
    env_logger::init();
    log::info!("Starting Lotion-rs with Verbose Logging...");

    // Load user config
    let config = LotionConfig::load();
    log::info!("Config: theme={}, restore_tabs={}", config.active_theme, config.restore_tabs);

    // Load saved state (if any)
    let app_state = AppState::load_from_disk().unwrap_or_else(AppState::new);
    let app_state = Arc::new(tokio::sync::Mutex::new(app_state));

    // Initialize Concrete Modules
    let security = Arc::new(SecurityModule::new());
    let policy = Arc::new(PolicyManager::new());
    let theming = Arc::new(ThemeManager::with_config(
        &config.active_theme,
        config.custom_css_path.clone(),
    ));
    let tab_manager = Arc::new(lotion_rs::tab_manager::TabManager::new(security.litebox.clone()));
    
    // Create a channel for Tauri to send messages to Iced
    let (tx, rx) = mpsc::channel(100);

    // Iced settings from config
    let mut settings = iced::Settings::with_flags(ui::Flags { rx });
    settings.window = iced::window::Settings {
        size: iced::Size::new(config.window.width as f32, config.window.height as f32),
        decorations: true,
        transparent: false,
        ..Default::default()
    };

    // Spawn Tauri in a separate thread
    let tx_clone = tx.clone();
    std::thread::spawn(move || {
        let app = tauri::Builder::default()
            .any_thread()
            .plugin(tauri_plugin_shell::init())
            .plugin(tauri_plugin_updater::Builder::new().build())
            .invoke_handler(tauri::generate_handler![
                lotion_rs::i18n::get_translation,
                lotion_rs::i18n::set_locale,
                lotion_rs::spellcheck::check_spelling,
                lotion_rs::spellcheck::get_spelling_suggestions,
                update_tab_title,
                log_network_event
            ])
            .setup(move |app| {
                // Initialize modules in Tauri state FIRST as trait objects where expected
                app.manage::<Arc<dyn lotion_rs::traits::SecuritySandbox>>(security.clone());
                app.manage::<Arc<dyn lotion_rs::traits::PolicyEnforcer>>(policy);
                app.manage::<Arc<dyn lotion_rs::traits::ThemingEngine>>(theming);
                app.manage::<Arc<dyn lotion_rs::traits::TabOrchestrator>>(tab_manager);
                app.manage(config);
                app.manage(app_state);
                app.manage(I18nManager::new());
                app.manage(SpellcheckManager::new());
                app.manage(tx_clone.clone());

                let handle = app.handle().clone();
                // Notify Iced that Tauri is ready
                let _ = tx_clone.blocking_send(Message::TauriReady(handle));
                
                log::info!("Tauri background layer initialized.");
                Ok(())
            })
            .build(tauri::generate_context!())
            .expect("error while building tauri application");

        app.run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
    });

    // Start Iced application
    ui::run(settings)
}
