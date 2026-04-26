use std::path::{Path, PathBuf};
use crate::config;
use walkdir::WalkDir;
use anyhow::{Result, anyhow};
use windows::{
    core::{PCWSTR, Interface},
    Win32::{
        System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, STGM},
        UI::Shell::{IShellLinkW, ShellLink},
        Storage::FileSystem::{
            WIN32_FIND_DATAW, GetLogicalDriveStringsW, GetDriveTypeW,
        },
    },
};

/* 
 * Scraper Module
 * 
 * Responsible for indexing applications, files, and directories across the system.
 * It scans standard Windows locations (Start Menu, Desktop), secondary drives,
 * and user-defined directories.
 * 
 * Features:
 * - Shell Link (.lnk) resolution using Windows COM API.
 * - Dynamic discovery of removable and secondary fixed drives.
 * - Localized path support for standard Windows folders (Documents, Downloads, etc.).
 * - File type filtering to prioritize executables and common document formats.
 */

const DRIVE_REMOVABLE: u32 = 2;
const DRIVE_FIXED: u32 = 3;

/** Data structure representing a discovered application or file. */
#[derive(Debug, Clone)]
pub struct AppInfo {
    pub name: String, // Display name (usually the filename without extension)
    pub path: PathBuf, // Absolute path to the executable or document
}

/** 
 * Orchestrates a full system scan.
 * Gathers paths from standard locations, connected drives, and user config.
 */
pub fn scan_apps() -> Vec<AppInfo> {
    println!("Scanning for apps and files in known directories and extra drives...");
    let mut apps = Vec::new();
    let config = config::load_config();
    
    let mut paths_to_scan = Vec::new();
    
    // ─── 1. USER DIRECTORIES ────────────────────────────────────────
    // Use 'directories' crate to resolve localized paths (e.g., 'Téléchargements' on French systems)
    if let Some(user_dirs) = directories::UserDirs::new() {
        // User-specific Start Menu
        let mut start_menu = user_dirs.home_dir().to_path_buf();
        start_menu.push("AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs");
        paths_to_scan.push(start_menu);
        
        // Standard libraries
        if let Some(path) = user_dirs.desktop_dir() { paths_to_scan.push(path.to_path_buf()); }
        if let Some(path) = user_dirs.document_dir() { paths_to_scan.push(path.to_path_buf()); }
        if let Some(path) = user_dirs.download_dir() { paths_to_scan.push(path.to_path_buf()); }
        if let Some(path) = user_dirs.picture_dir() { paths_to_scan.push(path.to_path_buf()); }
        if let Some(path) = user_dirs.video_dir() { paths_to_scan.push(path.to_path_buf()); }
        if let Some(path) = user_dirs.audio_dir() { paths_to_scan.push(path.to_path_buf()); }
    }
    
    // ─── 2. SYSTEM-WIDE LOCATIONS ───────────────────────────────────
    // Global Start Menu (C:\ProgramData)
    paths_to_scan.push(PathBuf::from("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs"));

    // ─── 3. SECONDARY DRIVES ────────────────────────────────────────
    // Automatically include USB drives and secondary fixed disks (D:, E:, etc.)
    for drive in get_extra_drives() {
        paths_to_scan.push(drive);
    }

    // ─── 4. CUSTOM USER PATHS ───────────────────────────────────────
    for path in config.extra_paths {
        paths_to_scan.push(path);
    }

    // Perform recursive scanning for each resolved path
    for path in paths_to_scan {
        if path.exists() {
            scan_directory(&path, &mut apps);
        }
    }

    apps
}

/** 
 * Recursively traverses a directory to find relevant files and subdirectories.
 * Limits depth to 5 levels to maintain performance and avoid deep system hierarchies.
 */
fn scan_directory(dir: &Path, apps: &mut Vec<AppInfo>) {
    for entry in WalkDir::new(dir)
        .max_depth(5) 
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        // Skip hidden items (dotfiles)
        if path.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.starts_with('.'))
            .unwrap_or(false) 
        {
            continue;
        }

        // Add directories themselves to the search index
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if path != dir {
                    apps.push(AppInfo {
                        name: name.to_string(),
                        path: path.to_path_buf(),
                    });
                }
            }
            continue;
        }

        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        
        match extension {
            // Shortcuts: Resolve their real target path via COM
            "lnk" => {
                if let Ok(info) = parse_lnk(path) {
                    apps.push(info);
                }
            }
            // Executables
            "exe" | "msi" | "bat" | "cmd" => {
                apps.push(AppInfo {
                    name: path.file_stem().and_then(|s| s.to_str()).unwrap_or("Unknown").to_string(),
                    path: path.to_path_buf(),
                });
            }
            // Common Document Formats
            "pdf" | "docx" | "xlsx" | "pptx" | "txt" | "png" | "jpg" | "jpeg" | "mp4" | "mp3" | "wav" => {
                apps.push(AppInfo {
                    name: path.file_name().and_then(|s| s.to_str()).unwrap_or("Unknown").to_string(),
                    path: path.to_path_buf(),
                });
            }
            // Generic fallback for direct children of search roots
            _ => {
                if path.parent() == Some(dir) {
                     apps.push(AppInfo {
                        name: path.file_name().and_then(|s| s.to_str()).unwrap_or("Unknown").to_string(),
                        path: path.to_path_buf(),
                    });
                }
            }
        }
    }
}

/** 
 * Queries the Win32 API to find all active drive letters.
 * Returns a list of paths for removable and secondary fixed drives (skipping C:).
 */
fn get_extra_drives() -> Vec<PathBuf> {
    let mut drives = Vec::new();
    unsafe {
        // Query required buffer size for the drive strings list
        let len = GetLogicalDriveStringsW(None);
        if len == 0 { return drives; }
        
        // Retrieve the null-separated list of drive strings (e.g., "C:\\0D:\\0\0")
        let mut buffer = vec![0u16; len as usize];
        let actual_len = GetLogicalDriveStringsW(Some(&mut buffer));
        
        if actual_len > 0 {
            let mut current_ptr = buffer.as_ptr();
            loop {
                if *current_ptr == 0 { break; } // Double null at end of list
                
                let mut str_len = 0;
                while *current_ptr.add(str_len) != 0 { str_len += 1; }
                
                let drive_slice = std::slice::from_raw_parts(current_ptr, str_len);
                let drive_str = String::from_utf16_lossy(drive_slice);
                
                let mut drive_with_null = drive_slice.to_vec();
                drive_with_null.push(0);
                
                // Identify the hardware type of the drive
                let drive_type = GetDriveTypeW(windows::core::PCWSTR(drive_with_null.as_ptr()));
                
                // Only include REMOVABLE (USB) or secondary FIXED drives (HDD/SSD)
                if drive_type == DRIVE_REMOVABLE || drive_type == DRIVE_FIXED {
                    if !drive_str.to_uppercase().starts_with("C:") {
                        drives.push(PathBuf::from(drive_str));
                    }
                }
                
                current_ptr = current_ptr.add(str_len + 1);
                if current_ptr >= buffer.as_ptr().add(actual_len as usize) { break; }
            }
        }
    }
    drives
}

/** 
 * Resolves a Windows Shell Link (.lnk) to its absolute file path.
 * Uses the IShellLinkW COM interface.
 */
fn parse_lnk(path: &Path) -> Result<AppInfo> {
    let name = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let target = unsafe {
        // Initialize COM for the current thread (Apartment model)
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        // Create an instance of the ShellLink COM object
        let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
            .map_err(|e| anyhow!("CoCreateInstance failed: {}", e))?;

        // Access the IPersistFile interface to load the .lnk binary data
        let persist_file: windows::Win32::System::Com::IPersistFile =
            shell_link.cast().map_err(|e| anyhow!("cast to IPersistFile failed: {}", e))?;

        let wide_path: Vec<u16> = path.to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0u16))
            .collect();

        persist_file.Load(PCWSTR(wide_path.as_ptr()), STGM(0))
            .map_err(|e| anyhow!("IPersistFile::Load failed: {}", e))?;

        // Extract the real target path from the resolved link
        let mut buf = [0u16; 260];
        let mut find_data = WIN32_FIND_DATAW::default();
        shell_link.GetPath(&mut buf, &mut find_data, 0)
            .map_err(|e| anyhow!("GetPath failed: {}", e))?;

        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        let target_str = String::from_utf16_lossy(&buf[..len]);

        if target_str.is_empty() {
            path.to_path_buf() // Fallback to the link path itself if target resolution fails
        } else {
            PathBuf::from(target_str)
        }
    };

    Ok(AppInfo { name, path: target })
}
