use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/* 
 * History Module
 * 
 * Tracks frequently accessed files, applications, and URLs.
 * This data is used to:
 * 1. Provide proactive auto-completion in the search bar.
 * 2. Boost the ranking of common items in search results.
 * 3. Persist user habits across application restarts.
 */

/** Represents a single frequently accessed item. */
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryItem {
    pub name: String,
    pub path: String,
    pub count: u32, // Number of times this item has been launched
}

/** Root structure for persisted history data. */
#[derive(Serialize, Deserialize, Default)]
pub struct History {
    pub urls: Vec<String>,      // List of visited domains/URLs
    pub items: Vec<HistoryItem>, // List of launched files/apps
}

impl History {
    /** Loads the history from the 'history.json' file. */
    pub fn load() -> Self {
        let path = Self::path();
        if let Ok(content) = fs::read_to_string(path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /** Serializes and saves the current history to disk. */
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /** Adds a URL to the history if it doesn't already exist. */
    pub fn add_url(&mut self, url: String) {
        if !self.urls.contains(&url) {
            self.urls.push(url);
            let _ = self.save();
        }
    }

    /** 
     * Registers an item launch. 
     * Increments the launch count if the item exists, otherwise creates it.
     * Keeps the list sorted by launch count and limited to 100 entries.
     */
    pub fn add_item(&mut self, name: String, path: String) {
        if let Some(item) = self.items.iter_mut().find(|i| i.path == path) {
            item.count += 1;
        } else {
            self.items.push(HistoryItem {
                name,
                path,
                count: 1,
            });
        }
        
        // Ranking Logic: Sort by most frequently used
        self.items.sort_by(|a, b| b.count.cmp(&a.count));
        
        // Performance optimization: Limit history size to prevent slow JSON parsing
        if self.items.len() > 100 {
            self.items.truncate(100);
        }
        
        let _ = self.save();
    }

    /** 
     * Searches history for a prefix match to provide auto-completion.
     * Returns the name of the most relevant item if found.
     */
    pub fn autocomplete(&self, text: &str) -> Option<String> {
        if text.is_empty() {
            return None;
        }
        
        // Priority 1: Match against frequently used apps/files
        if let Some(item) = self.items.iter()
            .find(|i| i.name.to_lowercase().starts_with(&text.to_lowercase()))
        {
            return Some(item.name.clone());
        }

        // Priority 2: Match against visited URLs/domains
        self.urls.iter()
            .find(|url| url.to_lowercase().contains(&text.to_lowercase()))
            .cloned()
    }

    /** Resolves the standard OS-specific path for the 'history.json' file. */
    fn path() -> PathBuf {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "simulquest", "appsearch") {
            proj_dirs.config_dir().join("history.json")
        } else {
            // Fallback to local directory if project directory resolution fails
            PathBuf::from("history.json")
        }
    }
}
