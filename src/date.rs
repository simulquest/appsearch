use chrono::{Local, Datelike, Duration, NaiveDate};
use regex::Regex;

/* 
 * Date & Time Module
 * 
 * Provides utility information related to dates, times, and countdowns.
 * Supports natural language queries like "days until Christmas" or "today + 90 days".
 */

/** 
 * Attempts to provide date-related information based on the user's query.
 * 
 * Supported Queries:
 * - Holidays: "Christmas", "Easter", "New Year"
 * - Current Status: "today", "now", "week", "unix"
 * - Relative Dates: "yesterday", "tomorrow"
 * - Date Math: "today + 90 days"
 * - Countdowns: "days until 25/12/2026"
 * - Metadata: "day of 25/12/2026" (get week day)
 */
pub fn try_date_info(query: &str) -> Option<String> {
    let query = query.trim().to_lowercase();
    
    // ─── 1. SPECIAL HOLIDAYS ────────────────────────────────────────
    
    if query == "christmas" || query == "xmas" || query == "noel" {
        let now = Local::now().naive_local().date();
        let year = now.year();
        let mut target = NaiveDate::from_ymd_opt(year, 12, 25).unwrap();
        if now > target {
            target = NaiveDate::from_ymd_opt(year + 1, 12, 25).unwrap();
        }
        let diff = target.signed_duration_since(now).num_days();
        return Some(format!("{} days until Christmas!", diff));
    }

    if query == "pâques" || query == "paques" || query == "easter" {
        let now = Local::now().naive_local().date();
        // Easter date calculation is complex; using hardcoded dates for near future
        let target = if now <= NaiveDate::from_ymd_opt(2026, 4, 5).unwrap() {
            NaiveDate::from_ymd_opt(2026, 4, 5).unwrap()
        } else {
            NaiveDate::from_ymd_opt(2027, 3, 28).unwrap()
        };
        let diff = target.signed_duration_since(now).num_days();
        return Some(format!("{} days until Easter!", diff));
    }

    if query == "new year" || query == "nouvel an" {
        let now = Local::now().naive_local().date();
        let target = NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).unwrap();
        let diff = target.signed_duration_since(now).num_days();
        return Some(format!("{} days until the New Year!", diff));
    }

    // ─── 2. CURRENT STATUS & TIMESTAMPS ─────────────────────────────
    
    if query == "date" || query == "today" || query == "now" {
        let now = Local::now();
        return Some(format!("{} (Week {})", now.format("%d/%m/%Y %H:%M:%S"), now.iso_week().week()));
    }

    if query == "week" {
        let now = Local::now();
        return Some(format!("Current Week: {}", now.iso_week().week()));
    }

    if query == "unix" || query == "timestamp" {
        let now = Local::now();
        return Some(format!("Unix Timestamp: {}", now.timestamp()));
    }

    // ─── 3. RELATIVE DATES ──────────────────────────────────────────
    
    if query == "tomorrow" {
        let tomorrow = Local::now() + Duration::days(1);
        return Some(format!("Tomorrow: {}", tomorrow.format("%d/%m/%Y")));
    }

    if query == "yesterday" {
        let yesterday = Local::now() - Duration::days(1);
        return Some(format!("Yesterday: {}", yesterday.format("%d/%m/%Y")));
    }

    // ─── 4. DATE MATH (e.g., "today + 90 days") ─────────────────────
    let re_math = Regex::new(r"^(?:today|now)\s*([+-])\s*(\d+)\s*days?$").ok()?;
    if let Some(caps) = re_math.captures(&query) {
        let sign = caps.get(1)?.as_str();
        let days: i64 = caps.get(2)?.as_str().parse().ok()?;
        let target = if sign == "+" {
            Local::now() + Duration::days(days)
        } else {
            Local::now() - Duration::days(days)
        };
        return Some(format!("Result: {}", target.format("%d/%m/%Y")));
    }

    // ─── 5. COUNTDOWNS (e.g., "days until 25/12/2026") ──────────────
    let re_until = Regex::new(r"^days\s+(?:until|to)\s+(\d{1,2}/\d{1,2}/\d{4})$").ok()?;
    if let Some(caps) = re_until.captures(&query) {
        let date_str = caps.get(1)?.as_str();
        if let Ok(target_date) = NaiveDate::parse_from_str(date_str, "%d/%m/%Y") {
            let today = Local::now().naive_local().date();
            let diff = target_date.signed_duration_since(today).num_days();
            if diff > 0 {
                return Some(format!("{} days until {}", diff, date_str));
            } else if diff < 0 {
                return Some(format!("{} days since {}", diff.abs(), date_str));
            } else {
                return Some("That's today!".to_string());
            }
        }
    }

    // ─── 6. DAY OF WEEK (e.g., "day of 25/12/2026") ──────────────────
    let re_day_of = Regex::new(r"^day\s+of\s+(\d{1,2}/\d{1,2}/\d{4})$").ok()?;
    if let Some(caps) = re_day_of.captures(&query) {
        let date_str = caps.get(1)?.as_str();
        if let Ok(target_date) = NaiveDate::parse_from_str(date_str, "%d/%m/%Y") {
            return Some(format!("{} was/will be a {}", date_str, target_date.format("%A")));
        }
    }

    None
}
