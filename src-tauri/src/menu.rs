use tauri::{
    menu::{MenuBuilder, MenuItem, SubmenuBuilder},
    AppHandle,
};

pub fn create_main_menu(app: &AppHandle) -> tauri::Result<()> {
    let pkg_info = app.package_info();
    
    // Lotion Menu
    let lotion_menu = SubmenuBuilder::new(app, "Lotion")
        .about(Some(tauri::menu::AboutMetadata {
            name: Some("Lotion".to_string()),
            version: Some(pkg_info.version.to_string()),
            ..Default::default()
        }))
        .separator()
        .item(&MenuItem::with_id(app, "preferences", "Preferences", true, Some("CmdOrCtrl+,"))?)
        .separator()
        .quit()
        .build()?;

    // Navigation Menu
    let nav_menu = SubmenuBuilder::new(app, "Navigation")
        .item(&MenuItem::with_id(app, "nav_back", "Back", true, Some("Alt+Left"))?)
        .item(&MenuItem::with_id(app, "nav_forward", "Forward", true, Some("Alt+Right"))?)
        .item(&MenuItem::with_id(app, "nav_refresh", "Refresh", true, Some("CmdOrCtrl+R"))?)
        .separator()
        .item(&MenuItem::with_id(app, "nav_home", "Home", true, Some("CmdOrCtrl+H"))?)
        .build()?;

    // Edit Menu
    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    // View Menu
    let view_menu = SubmenuBuilder::new(app, "View")
        .item(&MenuItem::with_id(app, "reload", "Reload", true, Some("F5"))?)
        .separator()
        .item(&MenuItem::with_id(app, "toggle_dev_tools", "Toggle Developer Tools", true, Some("F12"))?)
        .separator()
        .item(&MenuItem::with_id(app, "toggle_menu_bar", "Toggle Menu Bar", true, Some("CmdOrCtrl+Shift+M"))?)
        .build()?;

    let menu = MenuBuilder::new(app)
        .item(&lotion_menu)
        .item(&nav_menu)
        .item(&edit_menu)
        .item(&view_menu)
        .build()?;

    app.set_menu(menu)?;
    
    app.on_menu_event(move |_app, event| {
        match event.id().as_ref() {
            "nav_back" => { log::info!("Menu: Back"); }
            "nav_forward" => { log::info!("Menu: Forward"); }
            "nav_refresh" => { log::info!("Menu: Refresh"); }
            "nav_home" => { log::info!("Menu: Home"); }
            "preferences" => { log::info!("Menu: Preferences"); }
            "toggle_dev_tools" => {
                log::info!("Menu: Toggle Developer Tools (disabled in release)");
            }
            _ => {}
        }
    });

    Ok(())
}
