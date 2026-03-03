use lotion_rs::security::SecurityModule;
use lotion_rs::tab_manager::TabManager;
use lotion_rs::policy::PolicyManager;
use lotion_rs::ui::theming::ThemeManager;
use lotion_rs::traits::{SecuritySandbox, TabOrchestrator, PolicyEnforcer, ThemingEngine};
use lotion_rs::ui::{self, Message};
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::mpsc;
use lotion_rs::state::AppState;

fn main() -> iced::Result {
    env_logger::init();
    log::info!("Starting Lotion-rs with Iced Native Frontend & Zero-Trust Enforcement...");

    // Initialize Concrete Modules
    let security = Arc::new(SecurityModule::new());
    let policy = Arc::new(PolicyManager::new());
    let theming = Arc::new(ThemeManager::new());
    let app_state = Arc::new(tokio::sync::Mutex::new(AppState::new()));
    
    // Create a channel for Tauri to send messages to Iced
    let (tx, mut rx) = mpsc::channel(100);

    // Initial Iced settings with Flags
    let mut settings = iced::Settings::with_flags(ui::Flags { rx });
    settings.window = iced::window::Settings {
        size: iced::Size::new(1200.0, 800.0),
        decorations: false,
        transparent: true,
        ..Default::default()
    };

    // Spawn Tauri in a separate thread
    let tx_clone = tx.clone();
    std::thread::spawn(move || {
        let app = tauri::Builder::default()
            .plugin(tauri_plugin_shell::init())
            .setup(move |app| {
                let handle = app.handle().clone();
                // Notify Iced that Tauri is ready
                let _ = tx_clone.blocking_send(Message::TauriReady(handle));
                
                // Initialize modules in Tauri state
                app.manage(security);
                app.manage(policy);
                app.manage(theming);
                app.manage(app_state);
                
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
