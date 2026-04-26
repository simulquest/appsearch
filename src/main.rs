#![windows_subsystem = "windows"]

/* 
 * APPSearch - Main Entry Point
 * 
 * This application is a high-performance desktop search tool for Windows.
 * It uses Slint for the UI and raw Win32 APIs for window management (transparency, 
 * taskbar hiding, global hotkeys, etc.).
 *
 * Architecture:
 * - Slint Event Loop: Runs continuously to handle UI interactions.
 * - Win32 Integration: Manages window visibility and "always-on-top" behavior without killing the Slint loop.
 * - Multi-threading: Background threads for periodic app/file indexing and async tasks.
 */

mod scraper;
mod searcher;
mod hotkey;
mod config;
mod history;
mod converter;
mod math;
mod color;
mod date;
mod system;
mod currency;
mod timer;
mod tray;
mod updater;

use slint::{ComponentHandle, ModelRc, VecModel, SharedString};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}, mpsc};
use std::time::{Duration, Instant};
use crate::searcher::Searcher;
use crate::history::History;

// Win32 API imports for advanced window management
use windows::{
    core::w,
    Win32::{
        Foundation::{HWND, WPARAM, LPARAM},
        UI::WindowsAndMessaging::{
            FindWindowW, GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos,
            ShowWindow, SetForegroundWindow, SendMessageW, LoadImageW,
            GWL_EXSTYLE, HWND_TOPMOST,
            WS_EX_TOOLWINDOW, WS_EX_APPWINDOW,
            SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_FRAMECHANGED,
            SW_HIDE, SW_SHOW,
            GetCursorPos, WM_SETICON, ICON_BIG, ICON_SMALL,
            IMAGE_ICON, LR_LOADFROMFILE, LR_DEFAULTSIZE, HICON,
        },
        Graphics::Gdi::{
            GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTOPRIMARY,
        },
    },
};

// Include the generated Slint code
slint::include_modules!();

// ─── WIN32 WINDOW MANAGEMENT UTILITIES ───────────────────────────────

/** Retrieves the native Windows handle (HWND) for the APPSearch window. */
fn get_hwnd() -> Option<HWND> {
    unsafe { FindWindowW(None, w!("APPSearch")) }.ok()
}

/** 
 * Removes the window from the Windows Taskbar while keeping it active.
 * Uses WS_EX_TOOLWINDOW style to achieve the "background utility" look.
 */
fn hide_from_taskbar(hwnd: HWND) {
    unsafe {
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        let new_ex = (ex | WS_EX_TOOLWINDOW.0) & !WS_EX_APPWINDOW.0;
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex as isize);
        
        // Force Windows to refresh the taskbar state for this window
        let _ = SetWindowPos(
            hwnd,
            None,
            0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
    }
}

/** 
 * Centers the window on the monitor where the mouse cursor is currently located.
 * This ensures the search bar always appears on the user's active screen.
 */
fn center_window(hwnd: HWND) {
    unsafe {
        let mut rect = windows::Win32::Foundation::RECT::default();
        let _ = windows::Win32::UI::WindowsAndMessaging::GetWindowRect(hwnd, &mut rect);
        let win_w = rect.right - rect.left;
        let win_h = rect.bottom - rect.top;

        let mut cursor_pos = windows::Win32::Foundation::POINT::default();
        let _ = GetCursorPos(&mut cursor_pos);
        
        // Find monitor from cursor position
        let h_monitor = MonitorFromPoint(cursor_pos, MONITOR_DEFAULTTOPRIMARY);
        let mut monitor_info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        
        if GetMonitorInfoW(h_monitor, &mut monitor_info).as_bool() {
            let work_area = monitor_info.rcWork;
            let sw = work_area.right - work_area.left;
            let sh = work_area.bottom - work_area.top;
            
            // Calculate centered coordinates
            let x = work_area.left + ((sw - win_w) / 2);
            let y = work_area.top + ((sh - win_h) / 2);
            
            // Move window and ensure it is top-most
            let _ = SetWindowPos(
                hwnd, 
                Some(HWND_TOPMOST), 
                x, y, 0, 0,
                SWP_NOACTIVATE | SWP_NOSIZE
            );
        }
    }
}

/** 
 * Hides the window using the Win32 API.
 * This is crucial because it keeps the Slint event loop running in the background.
 */
fn win32_hide(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_HIDE);
    }
}

/** 
 * Shows the window, brings it to focus, and applies custom styling (icon, top-most).
 */
fn win32_show(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
        hide_from_taskbar(hwnd);
        
        // Load and set the application icon via Win32 for the Alt-Tab menu
        let icon_path = w!("assets\\logo.ico");
        let h_icon_res = LoadImageW(
            None, 
            icon_path, 
            IMAGE_ICON, 
            0, 0, 
            LR_LOADFROMFILE | LR_DEFAULTSIZE
        );
        
        if let Ok(h_icon_handle) = h_icon_res {
            let h_icon = HICON(h_icon_handle.0);
            let _ = SendMessageW(hwnd, WM_SETICON, Some(WPARAM(ICON_BIG as usize)), Some(LPARAM(h_icon.0 as isize)));
            let _ = SendMessageW(hwnd, WM_SETICON, Some(WPARAM(ICON_SMALL as usize)), Some(LPARAM(h_icon.0 as isize)));
        }

        // Re-enforce TOPMOST status
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}

// ─── STRING UTILITIES ────────────────────────────────────────────────

/** Simple heuristic to detect if the input string looks like a URL. */
fn is_url(text: &str) -> bool {
    let t = text.trim();
    t.starts_with("http://") || t.starts_with("https://") || (t.contains('.') && !t.contains(' ') && t.contains('.'))
}

/** Prepends 'https://' if the protocol is missing. */
fn prepare_url(text: &str) -> String {
    let t = text.trim();
    if t.starts_with("http://") || t.starts_with("https://") {
        t.to_string()
    } else {
        format!("https://{}", t)
    }
}

// ─── MAIN ────────────────────────────────────────────────────────────

fn main() -> anyhow::Result<()> {
    // Initialize UI
    let ui = AppWindow::new()?;
    let ui_handle = ui.as_weak();

    // Core application state
    let apps = Arc::new(Mutex::new(scraper::scan_apps()));
    let searcher = Arc::new(Searcher::new());
    let history = Arc::new(Mutex::new(History::load()));

    // ─── BACKGROUND THREADS ──────────────────────────────────────────
    
    // Channel to trigger rescans from the main thread
    let (rescan_tx, rescan_rx) = mpsc::channel::<()>();
    let apps_bg = apps.clone();
    
    println!("Background rescan thread starting...");
    std::thread::spawn(move || {
        println!("Background rescan thread is alive and waiting for triggers.");
        while rescan_rx.recv().is_ok() {
            println!(">>> TRIGGER RECEIVED: Starting full rescan of apps and drives...");
            let new_apps = scraper::scan_apps();
            if let Ok(mut lock) = apps_bg.lock() {
                *lock = new_apps;
                println!(">>> SUCCESS: Index updated with {} items.", lock.len());
            }
            println!(">>> Rescan complete.");
            while rescan_rx.try_recv().is_ok() {}
        }
        println!("Background rescan thread is shutting down.");
    });

    // Periodic rescan every 5 minutes to keep the index fresh
    let tx_periodic = rescan_tx.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(300));
            let _ = tx_periodic.send(());
        }
    });

    let last_rescan = Arc::new(Mutex::new(Instant::now()));

    // Initial UI state setup
    ui.set_results(ModelRc::new(VecModel::from(Vec::new())));

    // TRICK: Show window once to create the HWND, then hide it immediately.
    // This allows us to use Win32 Show/Hide while keeping the Slint process alive.
    ui.show()?;
    if let Some(hwnd) = get_hwnd() {
        hide_from_taskbar(hwnd);
        win32_hide(hwnd);
    }

    // ─── UI CALLBACKS & EVENT HANDLERS ───────────────────────────────

    let s2 = searcher.clone();
    let a2 = apps.clone();
    let h2 = ui_handle.clone();
    let hist_search = history.clone();

    // Helper for color formatting
    ui.on_color_to_string(|c| {
        format!("#{:02X}{:02X}{:02X}", c.red(), c.green(), c.blue()).into()
    });

    // HSVA to Slint Color conversion logic (used by the picker)
    ui.on_hsva_to_color(|h, s, v, a| {
        let c = v * s;
        let x = c * (1.0 - (((h * 6.0) % 2.0) - 1.0).abs());
        let m = v - c;
        let (r, g, b) = if h < 1.0 / 6.0 { (c, x, 0.0) } 
        else if h < 2.0 / 6.0 { (x, c, 0.0) } 
        else if h < 3.0 / 6.0 { (0.0, c, x) } 
        else if h < 4.0 / 6.0 { (0.0, x, c) } 
        else if h < 5.0 / 6.0 { (x, 0.0, c) } 
        else { (c, 0.0, x) };

        slint::Color::from_argb_u8(
            (a * 255.0) as u8,
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
        )
    });

    // MAIN SEARCH LOGIC: Triggered on every keystroke
    ui.on_search_text_changed(move |text| {
        if let Some(ui) = h2.upgrade() {
            let engine_mode = ui.get_search_engine_mode().to_string();
            
            // Handle Specialized Modes (YouTube/Spotify)
            if engine_mode == "YouTube" || engine_mode == "Spotify" {
                let mut entries = Vec::new();
                if !text.is_empty() {
                    let (name, url) = if engine_mode == "YouTube" {
                        (format!("Search YouTube for: {}", text), format!("https://www.youtube.com/results?search_query={}", text))
                    } else {
                        (format!("Search Spotify for: {}", text), format!("https://open.spotify.com/search/{}", text))
                    };
                    entries.push(AppEntry { name: name.into(), path: url.into(), is_url: true });
                }
                ui.set_results(ModelRc::new(VecModel::from(entries)));
                ui.set_selected_index(0);
                return;
            }

            ui.set_is_url_input(is_url(&text));
            
            // Auto-close picker if the text is no longer a valid color
            if color::try_parse_color(&text).is_none() {
                ui.set_is_color_picker_open(false);
            }
            
            // PIPELINE: Check for utility results (Math -> Converter -> Color -> Date -> System -> Currency -> Timer)
            if let Some(result) = converter::try_convert(&text) {
                ui.set_result_title("CONVERSION".into());
                ui.set_result_text(result.into());
                ui.set_show_color_preview(false);
            } else if let Some(result) = math::try_evaluate(&text) {
                ui.set_result_title("CALCULATION".into());
                ui.set_result_text(result.into());
                ui.set_show_color_preview(false);
            } else if let Some(color) = color::try_parse_color(&text) {
                ui.set_result_title("COLOR".into());
                ui.set_result_text(format!("{} - {}", color.hex, color.rgb).into());
                ui.set_result_color(color.slint_color);
                ui.set_show_color_preview(true);
            } else if let Some(result) = date::try_date_info(&text) {
                ui.set_result_title("DATE/TIME".into());
                ui.set_result_text(result.into());
                ui.set_show_color_preview(false);
            } else if let Some((title, result)) = system::try_parse_system_command(&text) {
                ui.set_result_title(title.into());
                ui.set_result_text(result.into());
                ui.set_show_color_preview(false);
            } else if let Some((title, result)) = currency::try_convert_currency(&text) {
                ui.set_result_title(title.into());
                ui.set_result_text(result.into());
                ui.set_show_color_preview(false);
            } else if let Some((title, result)) = timer::try_parse_timer(&text) {
                ui.set_result_title(title.into());
                ui.set_result_text(result.into());
                ui.set_show_color_preview(false);
            } else {
                ui.set_result_text("".into());
                ui.set_show_color_preview(false);
            }
            
            // Auto-complete suggestion based on history
            let suggestion = hist_search.lock().unwrap().autocomplete(&text);
            ui.set_autocomplete_text(suggestion.unwrap_or_else(|| "".to_string()).into());

            // App/File Search Indexing
            let apps_lock = a2.lock().unwrap();
            let hist_lock = hist_search.lock().unwrap();
            let filtered = s2.search(&text, &apps_lock, &hist_lock);
            let mut entries: Vec<AppEntry> = filtered.iter().map(|a| AppEntry {
                name: a.name.clone().into(),
                path: a.path.to_string_lossy().to_string().into(),
                is_url: false,
            }).collect();

            // Inject special entries (Open AppData, Open URL, Search Google)
            if !text.is_empty() {
                if text.to_lowercase() == "appdata" {
                    if let Ok(path) = std::env::var("APPDATA") {
                        entries.insert(0, AppEntry { name: "Open AppData Folder".into(), path: path.into(), is_url: false });
                    }
                }
                
                if is_url(&text) {
                    entries.insert(0, AppEntry { name: format!("Open URL: {}", text).into(), path: text.clone().into(), is_url: true });
                }
                
                entries.push(AppEntry {
                    name: format!("Search Google for: {}", text).into(),
                    path: format!("https://www.google.com/search?q={}", text).into(),
                    is_url: true,
                });

                // Add priority entries for system commands and timers
                if system::try_parse_system_command(&text).is_some() {
                    entries.insert(0, AppEntry { name: "SYSTEM: Execute command".into(), path: "".into(), is_url: false });
                }
                if timer::try_parse_timer(&text).is_some() {
                    entries.insert(0, AppEntry { name: "TIMER: Start timer".into(), path: "".into(), is_url: false });
                }
            }

            ui.set_results(ModelRc::new(VecModel::from(entries)));
            ui.set_selected_index(0);
        }
    });

    // Manual auto-complete request (e.g. via Tab key)
    let hist_auto = history.clone();
    ui.on_autocomplete_requested(move |text| {
        if let Some(auto) = hist_auto.lock().unwrap().autocomplete(&text) {
            SharedString::from(auto)
        } else {
            SharedString::from(text)
        }
    });

    // SELECTION LOGIC: Triggered when user presses Enter or clicks an item
    let is_visible = Arc::new(AtomicBool::new(false));
    let iv_select = is_visible.clone();
    let hist_launch = history.clone();
    let ui_handle_select = ui_handle.clone();
    ui.on_entry_selected(move |entry| {
        if entry.is_url {
            let url = prepare_url(entry.path.as_str());
            let domain = url.replace("https://", "").replace("http://", "").replace("www.", "");
            let domain = domain.split('/').next().unwrap_or(&domain).to_string();
            
            if let Err(e) = open::that(&url) {
                eprintln!("Launch failed for URL {}: {}", url, e);
            } else {
                hist_launch.lock().unwrap().add_url(domain);
            }
        } else {
            let path = entry.path.as_str().to_string();
            let name = entry.name.as_str().to_string();
            
            // Check for custom internal commands (System/Timer)
            if name.starts_with("SYSTEM: ") || name.starts_with("TIMER: ") {
                let cmd_text = ui_handle_select.upgrade().unwrap().get_search_text().to_string();
                if !system::execute_system_command(&cmd_text) {
                    timer::start_timer(&cmd_text, ui_handle_select.clone());
                }
            } else {
                // Launch file or app
                hist_launch.lock().unwrap().add_item(name, path.clone());
                std::thread::spawn(move || {
                    let _ = std::process::Command::new("cmd").args(["/C", "start", "", &path]).spawn();
                });
            }
        }
        
        // Reset UI state for next activation
        if let Some(ui) = ui_handle_select.upgrade() {
            ui.set_search_text("".into());
            ui.set_results(ModelRc::new(VecModel::from(Vec::new())));
            ui.set_search_engine_mode("".into());
            ui.set_autocomplete_text("".into());
            ui.set_result_text("".into());
            ui.set_show_color_preview(false);
        }

        // Hide window using Win32
        if let Some(hwnd) = get_hwnd() { win32_hide(hwnd); }
        iv_select.store(false, Ordering::Relaxed);
    });

    // Close requested via ESC or Close Button
    let iv_close = is_visible.clone();
    let ui_handle_close = ui_handle.clone();
    ui.on_close_requested(move || {
        if let Some(ui) = ui_handle_close.upgrade() { ui.set_search_engine_mode("".into()); }
        if let Some(hwnd) = get_hwnd() { win32_hide(hwnd); }
        iv_close.store(false, Ordering::Relaxed);
    });

    // ─── EXTERNAL INTEGRATIONS ───────────────────────────────────────

    let _hotkey_manager = hotkey::setup_hotkey();
    let _tray_handle = tray::setup_tray();
    let tray_menu_rx = tray::TrayMenuEvent::receiver();
    let tray_rx = tray::TrayEvent::receiver();

    // ─── MAIN POLLING LOOP (50ms interval) ───────────────────────────
    
    let timer = slint::Timer::default();
    let h5 = ui_handle.clone();
    let iv5 = is_visible.clone();
    let tx_trigger = rescan_tx.clone();
    let last_r = last_rescan.clone();
    
    timer.start(slint::TimerMode::Repeated, Duration::from_millis(50), move || {
        // 1. Handle Global Hotkey (Alt+Space)
        if hotkey::check_hotkey_events() {
            if let Some(ui) = h5.upgrade() {
                if let Some(hwnd) = get_hwnd() {
                    if iv5.load(Ordering::Relaxed) {
                        win32_hide(hwnd);
                        iv5.store(false, Ordering::Relaxed);
                    } else {
                        // Reset and show search bar
                        ui.set_search_text("".into());
                        ui.set_results(ModelRc::new(VecModel::from(Vec::new())));
                        ui.set_selected_index(0);
                        ui.set_is_url_input(false);
                        ui.set_autocomplete_text("".into());
                        ui.set_result_text("".into());
                        ui.set_show_color_preview(false);

                        center_window(hwnd);
                        win32_show(hwnd);
                        ui.invoke_focus_search_input();
                        iv5.store(true, Ordering::Relaxed);

                        // Proactive background index refresh (cooldown: 10s)
                        if let Ok(mut last) = last_r.lock() {
                            if last.elapsed().as_secs() > 10 {
                                let _ = tx_trigger.send(());
                                *last = Instant::now();
                            }
                        }
                    }
                }
            }
        }

        // 2. Handle Tray Context Menu Events (Update, Exit)
        if let Ok(event) = tray_menu_rx.try_recv() {
            if event.id == "check_update" { updater::check_and_update(); } 
            else if event.id == "exit" { let _ = slint::quit_event_loop(); }
        }

        // 3. Handle Tray Icon Click (Toggle window visibility)
        static mut LAST_TRAY_CLICK: Option<Instant> = None;
        if let Ok(_event) = tray_rx.try_recv() {
            let now = Instant::now();
            let should_toggle = unsafe {
                match LAST_TRAY_CLICK {
                    Some(last) => now.duration_since(last) > Duration::from_millis(500),
                    None => true,
                }
            };

            if should_toggle {
                unsafe { LAST_TRAY_CLICK = Some(now); }
                if let Some(ui) = h5.upgrade() {
                    if let Some(hwnd) = get_hwnd() {
                        if !iv5.load(Ordering::Relaxed) {
                            ui.set_search_text("".into());
                            ui.set_results(ModelRc::new(VecModel::from(Vec::new())));
                            ui.set_selected_index(0);
                            center_window(hwnd);
                            win32_show(hwnd);
                            ui.invoke_focus_search_input();
                            iv5.store(true, Ordering::Relaxed);
                        } else {
                            win32_hide(hwnd);
                            iv5.store(false, Ordering::Relaxed);
                        }
                    }
                }
            }
        }
    });

    // Enter the Slint event loop
    slint::run_event_loop()?;
    Ok(())
}
