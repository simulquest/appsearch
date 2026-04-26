use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use crate::scraper::AppInfo;
use crate::history::History;

/* 
 * Searcher Module
 * 
 * Provides high-speed fuzzy matching and ranking for search queries.
 * It combines raw text matching with historical usage data to ensure 
 * the most relevant results appear at the top.
 */

/** The core search engine using the Skim fuzzy matching algorithm. */
pub struct Searcher {
    matcher: SkimMatcherV2,
}

impl Searcher {
    /** Initializes the searcher with default Skim matching settings. */
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    /** 
     * Performs a fuzzy search across a list of applications/files.
     * 
     * Ranking Algorithm:
     * 1. Fuzzy Match: Calculates a base score based on character proximity and sequence.
     * 2. Usage Boost: Multiplies historical launch counts by a weight factor (50) 
     *    and adds it to the base score.
     * 3. Sort & Truncate: Returns the top 10 highest-scoring results.
     */
    pub fn search(&self, query: &str, apps: &[AppInfo], history: &History) -> Vec<AppInfo> {
        if query.is_empty() {
            return Vec::new();
        }

        let mut scored_apps: Vec<(i64, AppInfo)> = apps
            .iter()
            .filter_map(|app| {
                self.matcher
                    .fuzzy_match(&app.name, query)
                    .map(|mut score| {
                        // Apply history-based ranking boost
                        if let Some(hist_item) = history.items.iter().find(|i| i.path == app.path.to_string_lossy()) {
                            // Significant boost for frequently used items to ensure they stay on top
                            score += (hist_item.count as i64) * 50; 
                        }
                        (score, app.clone())
                    })
            })
            .collect();

        // Sort by score descending (highest relevance first)
        scored_apps.sort_by(|a, b| b.0.cmp(&a.0));

        // Limit results to the top 10 to maintain UI snappiness
        scored_apps.into_iter().take(10).map(|(_, app)| app).collect()
    }
}
