use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fs;
use anyhow::Result;

/* 
 * Configuration Module
 * 
 * Manages the application's persistent settings, such as user-defined 
 * search paths and the global activation hotkey.
 * Settings are stored in JSON format within the standard OS configuration directory.
 */

/** Application settings structure. */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub extra_paths: Vec<PathBuf>, // Additional directories to index for search
    pub hotkey: String,           // Activation hotkey (e.g., "Alt+Space")
}

impl Default for Config {
    /** Provides fallback settings if no configuration file is found. */
    fn default() -> Self {
        Self {
            extra_paths: Vec::new(),
            hotkey: "Alt+Space".to_string(), // Modern default for search tools
        }
    }
}

/** 
 * Resolves the absolute path to the 'config.json' file.
 * Automatically creates the parent directories if they do not exist.
 * Uses standard OS-specific directories (e.g., AppData/Roaming on Windows).
 */
pub fn get_config_path() -> Result<PathBuf> {
    let mut path = directories::ProjectDirs::from("com", "simulquest", "appsearch")
        .map(|proj| proj.config_dir().to_path_buf())
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
    
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    
    path.push("config.json");
    Ok(path)
}

/** 
 * Loads settings from disk.
 * Returns the stored configuration or the default values if loading fails.
 */
pub fn load_config() -> Config {
    if let Ok(path) = get_config_path() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
    }
    Config::default()
}
