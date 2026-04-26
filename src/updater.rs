use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_YESNO, IDYES};

/* 
 * Update Module
 * 
 * Handles checking for new versions of APPSearch and performing 
 * automated downloads and installations of the latest setup package.
 * Uses a simple HTTP-based version check and downloads the installer to the temp directory.
 */

/** 
 * Checks for updates in a background thread to prevent UI freezing.
 * Uses an AtomicBool to ensure only one update check is running at a time.
 */
pub fn check_and_update() {
    static IS_CHECKING: AtomicBool = AtomicBool::new(false);
    
    // Prevent concurrent checks
    if IS_CHECKING.swap(true, Ordering::SeqCst) {
        return;
    }

    std::thread::spawn(|| {
        let current_version = "1.0.0";
        // Remote URLs for version tracking and installer download
        let version_url = "https://raw.githubusercontent.com/simulquest/appsearch/refs/heads/main/version.txt";
        let download_url = "https://github.com/simulquest/appsearch/download/APPSearch_Setup.exe";
        
        println!("Checking for updates...");
        
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("APPSearch-Updater")
            .build();

        let client = match client {
            Ok(c) => c,
            Err(_) => {
                IS_CHECKING.store(false, Ordering::SeqCst);
                return;
            }
        };

        // ─── 1. Version Comparison ────────────────────────────────────
        match client.get(version_url).send() {
            Ok(resp) => {
                if let Ok(new_version) = resp.text() {
                    let new_version = new_version.trim();
                    if new_version != current_version && !new_version.is_empty() {
                        println!("New version found: {}", new_version);
                        
                        // Show native Windows message box for confirmation
                        let confirm = unsafe {
                            let msg = format!(
                                "Une nouvelle version ({}) est disponible !\nSouhaitez-vous la télécharger et l'installer maintenant ?", 
                                new_version
                            );
                            let msg_w: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
                            let title_w: Vec<u16> = "Mise à jour disponible".encode_utf16().chain(std::iter::once(0)).collect();
                            
                            MessageBoxW(
                                None,
                                windows::core::PCWSTR(msg_w.as_ptr()),
                                windows::core::PCWSTR(title_w.as_ptr()),
                                MB_YESNO | MB_ICONINFORMATION
                            )
                        };

                        if confirm == IDYES {
                            perform_update(client, download_url);
                        }
                    } else {
                        // User feedback if manually triggered
                        println!("Already up to date.");
                        unsafe {
                            let msg = format!("APPSearch est à jour (v{}).", current_version);
                            let msg_w: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
                            let title_w: Vec<u16> = "Mise à jour".encode_utf16().chain(std::iter::once(0)).collect();
                            
                            MessageBoxW(
                                None,
                                windows::core::PCWSTR(msg_w.as_ptr()),
                                windows::core::PCWSTR(title_w.as_ptr()),
                                MB_ICONINFORMATION
                            );
                        }
                    }
                }
            },
            Err(e) => {
                eprintln!("Failed to check for updates: {}", e);
            }
        }
        
        IS_CHECKING.store(false, Ordering::SeqCst);
    });
}

/** 
 * Downloads the latest installer, launches it, and terminates the current process.
 * Termination is necessary to allow the installer to overwrite the running executable.
 */
fn perform_update(client: reqwest::blocking::Client, url: &str) {
    println!("Downloading update from {}...", url);
    
    // Save to system temp directory
    let temp_dir = std::env::temp_dir();
    let setup_path = temp_dir.join("APPSearch_Setup.exe");

    match client.get(url).send() {
        Ok(mut resp) => {
            let mut file = match std::fs::File::create(&setup_path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Failed to create temp file: {}", e);
                    return;
                }
            };

            // Stream the download to the file
            if let Err(e) = std::io::copy(&mut resp, &mut file) {
                eprintln!("Failed to download update: {}", e);
                return;
            }

            println!("Download complete. Launching installer...");
            
            // Launch the installer via cmd 'start' to run it independently
            let _ = std::process::Command::new("cmd")
                .args(["/C", "start", "", &setup_path.to_string_lossy()])
                .spawn();
            
            // Exit immediately to free up file locks
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Download failed: {}", e);
        }
    }
}
