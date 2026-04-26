use serde::Deserialize;
use std::collections::HashMap;
use regex::Regex;

/* 
 * Currency Conversion Module
 * 
 * Provides real-time currency conversion using the Frankfurter API.
 * Supports standard 3-letter currency codes (ISO 4217).
 */

/** Structure matching the JSON response from api.frankfurter.app. */
#[derive(Deserialize, Debug)]
struct FrankfurterResponse {
    amount: f64,
    base: String,
    date: String,
    rates: HashMap<String, f64>,
}

/** 
 * Attempts to parse a string as a currency conversion request.
 * Format: [amount] [from_code] [to|in|=] [to_code]
 * Example: "50 EUR to USD", "100 USD in GBP"
 * 
 * Returns: Option<(Title, ResultString)>
 */
pub fn try_convert_currency(text: &str) -> Option<(String, String)> {
    let t = text.trim().to_lowercase();
    
    // Regex to capture amount, source currency and target currency
    let re = Regex::new(r"([\d.]+)\s*([a-z]{3})\s*(?:to|in|=)\s*([a-z]{3})").ok()?;
    let caps = re.captures(&t)?;
    
    let amount: f64 = caps.get(1)?.as_str().parse().ok()?;
    let from = caps.get(2)?.as_str().to_uppercase();
    let to = caps.get(3)?.as_str().to_uppercase();
    
    // Handle same-currency conversion immediately
    if from == to {
        return Some(("CURRENCY".to_string(), format!("{:.2} {}", amount, to)));
    }

    // API Call (Note: Blocking call is used here for simplicity as it's triggered by UI typing)
    // The Frankfurter API is a free, open-source service for exchange rates.
    let url = format!("https://api.frankfurter.app/latest?amount={}&from={}&to={}", amount, from, to);
    
    match reqwest::blocking::get(&url) {
        Ok(resp) => {
            if let Ok(data) = resp.json::<FrankfurterResponse>() {
                if let Some(rate) = data.rates.get(&to) {
                    return Some(("CURRENCY".to_string(), format!("{:.2} {}", rate, to)));
                }
            }
        }
        Err(_) => {
            // Provide a visual hint that the request is pending or failed
            return Some(("CURRENCY".to_string(), "...".to_string()));
        }
    }

    None
}
