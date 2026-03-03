use lotion_rs::security::SecurityModule;
use lotion_rs::tab_manager::TabManager;
use lotion_rs::policy::PolicyManager;
use lotion_rs::ui::theming::ThemeManager;
use lotion_rs::traits::{SecuritySandbox, TabOrchestrator, PolicyEnforcer, ThemingEngine};
use std::sync::Arc;
use tauri::Manager;

fn main() -> iced::Result {
    env_logger::init();
    log::info!("Starting Lotion-rs with Iced Native Frontend & Zero-Trust Enforcement...");

    // Initialize Concrete Modules
    let security = Arc::new(SecurityModule::new());
    let policy = Arc::new(PolicyManager::new());
    let theming = Arc::new(ThemeManager::new());
    
    // In a real application, we would pass these to the Iced state
    // For now, ensure they are registered in the global state if needed
    
    // Initialize Native Menu
    // Note: In Tauri v2, we need an AppHandle. ui::run() could be modified 
    // to initialize the menu after Tauri starts, or we can use a plugin.
    
    // Start Iced application
    ui::run()
}
