use global_hotkey::{
    hotkey::{HotKey, Modifiers, Code},
    GlobalHotKeyManager, GlobalHotKeyEvent, HotKeyState,
};
use std::fs;

/* 
 * Hotkey Module
 * 
 * Manages the registration and detection of global system-wide hotkeys.
 * This allows the user to summon APPSearch from any application.
 * 
 * Configuration:
 * - Reads the hotkey from a 'shortcut.txt' file located next to the executable.
 * - Defaults to 'Ctrl+Shift+K' if the file is missing or invalid.
 */

/** 
 * Initializes the global hotkey manager and registers the configured shortcut.
 * Searches for 'shortcut.txt' in the executable directory or current working directory.
 */
pub fn setup_hotkey() -> GlobalHotKeyManager {
    let manager = GlobalHotKeyManager::new().expect("Failed to create GlobalHotKeyManager");
    
    // Resolve path to shortcut.txt (Priority: 1. Next to EXE, 2. CWD)
    let mut shortcut_path = std::env::current_exe()
        .map(|mut path| {
            path.pop();
            path.join("shortcut.txt")
        })
        .unwrap_or_else(|_| std::path::PathBuf::from("shortcut.txt"));

    if !shortcut_path.exists() {
        let dev_path = std::path::PathBuf::from("shortcut.txt");
        if dev_path.exists() { shortcut_path = dev_path; }
    }

    println!("Searching for shortcut file at: {:?}", shortcut_path);
    
    // Load shortcut string
    let shortcut_str = match fs::read_to_string(&shortcut_path) {
        Ok(s) => {
            let content = s.trim().to_string();
            println!("Successfully read shortcut.txt: '{}'", content);
            content
        },
        Err(_) => {
            println!("No shortcut.txt found. Using default: 'Ctrl+Shift+K'");
            "Ctrl+Shift+K".to_string()
        }
    };

    // Attempt registration
    if let Some(hotkey) = parse_hotkey(&shortcut_str) {
        match manager.register(hotkey) {
            Ok(_) => println!("Successfully registered hotkey: '{}'", shortcut_str),
            Err(e) => println!("Failed to register hotkey '{}': {}", shortcut_str, e),
        }
    } else {
        println!("Invalid hotkey format in shortcut.txt: '{}'. Falling back to default.", shortcut_str);
        let default_hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyK);
        let _ = manager.register(default_hotkey);
    }
    
    manager
}

/** 
 * Parses a string representation of a hotkey (e.g., "Ctrl+Alt+S") 
 * into a structured HotKey object.
 */
fn parse_hotkey(s: &str) -> Option<HotKey> {
    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
    let mut modifiers = Modifiers::empty();
    let mut key_code = None;

    for part in parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "shift" => modifiers |= Modifiers::SHIFT,
            "alt" => modifiers |= Modifiers::ALT,
            "meta" | "win" | "command" => modifiers |= Modifiers::META,
            k => {
                key_code = parse_key_code(k);
            }
        }
    }

    key_code.map(|code| HotKey::new(Some(modifiers), code))
}

/** Maps a key name (e.g., "Space", "Enter", "A") to its corresponding Code. */
fn parse_key_code(s: &str) -> Option<Code> {
    match s.to_uppercase().as_str() {
        "A" => Some(Code::KeyA),
        "B" => Some(Code::KeyB),
        "C" => Some(Code::KeyC),
        "D" => Some(Code::KeyD),
        "E" => Some(Code::KeyE),
        "F" => Some(Code::KeyF),
        "G" => Some(Code::KeyG),
        "H" => Some(Code::KeyH),
        "I" => Some(Code::KeyI),
        "J" => Some(Code::KeyJ),
        "K" => Some(Code::KeyK),
        "L" => Some(Code::KeyL),
        "M" => Some(Code::KeyM),
        "N" => Some(Code::KeyN),
        "O" => Some(Code::KeyO),
        "P" => Some(Code::KeyP),
        "Q" => Some(Code::KeyQ),
        "R" => Some(Code::KeyR),
        "S" => Some(Code::KeyS),
        "T" => Some(Code::KeyT),
        "U" => Some(Code::KeyU),
        "V" => Some(Code::KeyV),
        "W" => Some(Code::KeyW),
        "X" => Some(Code::KeyX),
        "Y" => Some(Code::KeyY),
        "Z" => Some(Code::KeyZ),
        "SPACE" => Some(Code::Space),
        "ENTER" => Some(Code::Enter),
        _ => None,
    }
}

/** 
 * Polls the hotkey event receiver to check if the shortcut was pressed.
 * Non-blocking: returns true if at least one press event is in the queue.
 */
pub fn check_hotkey_events() -> bool {
    let mut triggered = false;
    while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
        if event.state == HotKeyState::Pressed {
            triggered = true;
        }
    }
    triggered
}
