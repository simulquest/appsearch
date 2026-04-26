use tray_icon::{
    menu::{Menu, MenuItem, MenuEvent, PredefinedMenuItem},
    TrayIcon, TrayIconBuilder, TrayIconEvent,
};
use std::path::Path;

/* 
 * System Tray Module
 * 
 * Manages the application's presence in the Windows System Tray (Taskbar notification area).
 * Allows the user to interact with the application when the main window is hidden.
 */

// Aliases for easier event handling in the main loop
pub type TrayMenuEvent = MenuEvent;
pub type TrayEvent = TrayIconEvent;

/** 
 * Initializes the tray icon and its associated context menu.
 * 
 * Menu Items:
 * - Check for Updates: Triggers the update check logic.
 * - Exit: Completely shuts down the application.
 */
pub fn setup_tray() -> TrayIcon {
    let tray_menu = Menu::new();
    let update_item = MenuItem::with_id("check_update", "Check for Updates", true, None);
    let exit_item = MenuItem::with_id("exit", "Exit", true, None);
    
    // Construct the context menu
    let _ = tray_menu.append_items(&[
        &update_item,
        &PredefinedMenuItem::separator(),
        &exit_item,
    ]);

    // Load the visual icon (assets/logo.ico)
    let icon = load_icon(Path::new("assets/logo.ico"));

    let mut builder = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("APPSearch");

    if let Some(i) = icon {
        builder = builder.with_icon(i);
    }

    builder.build().expect("Failed to initialize System Tray Icon")
}

/** 
 * Loads an .ico or image file and converts it into the format 
 * required by the tray_icon library (Raw RGBA).
 */
fn load_icon(path: &Path) -> Option<tray_icon::Icon> {
    if !path.exists() {
        eprintln!("Tray icon not found at {:?}", path);
        return None;
    }

    // Decode image using the 'image' crate
    let image = match image::open(path) {
        Ok(img) => img.into_rgba8(),
        Err(e) => {
            eprintln!("Failed to open tray icon: {}", e);
            return None;
        }
    };

    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    // Create the tray_icon compatible object
    match tray_icon::Icon::from_rgba(rgba, width, height) {
        Ok(icon) => Some(icon),
        Err(e) => {
            eprintln!("Failed to create tray icon from RGBA: {}", e);
            None
        }
    }
}
