use regex::Regex;
use std::collections::HashMap;

/* 
 * Unit Conversion Module
 * 
 * Provides a robust engine for converting values between different units.
 * Supports:
 * - Time (Seconds, Minutes, Hours, Days, Months, Years)
 * - Distance (Meters, Kilometers, Miles, Feet, Inches, etc.)
 * - Weight (Kilograms, Grams, Pounds, Ounces, etc.)
 * - Temperature (Celsius, Fahrenheit, Kelvin)
 * 
 * Features:
 * - Context-aware unit detection (e.g., distinguishing between 'm' for meter vs 'm' for minute/month).
 * - Multi-part input parsing (e.g., "1h 30m in seconds").
 * - Smart formatting (e.g., "70s in min" -> "1min 10s").
 */

/** Internal representation of a physical unit. */
#[derive(Debug, Clone)]
struct Unit {
    name: String,      // Full name (e.g., "kilometer")
    symbol: String,    // Standard symbol (e.g., "km")
    aliases: Vec<String>, // Alternative names (e.g., ["kilometers", "kilometre"])
    to_base: f64,      // Multiplier to convert to the category's base unit
    category: String,  // e.g., "distance", "time", "weight"
}

impl Unit {
    fn new(name: &str, symbol: &str, aliases: Vec<&str>, to_base: f64, category: &str) -> Self {
        Unit {
            name: name.to_string(),
            symbol: symbol.to_string(),
            aliases: aliases.iter().map(|&s| s.to_string()).collect(),
            to_base,
            category: category.to_string(),
        }
    }
    
    /** Checks if a string matches this unit's name, symbol, or aliases. */
    fn matches(&self, unit_str: &str) -> bool {
        let unit_lower = unit_str.to_lowercase();
        self.name == unit_lower || 
        self.symbol == unit_lower ||
        self.aliases.contains(&unit_lower)
    }
}

/** The main converter engine holding all supported unit definitions. */
pub struct UnitConverter {
    units: HashMap<String, Unit>,
}

impl UnitConverter {
    /** Initializes the converter with all supported unit categories and their base conversions. */
    pub fn new() -> Self {
        let mut units = HashMap::new();
        
        // ─── 1. TIME (Base: seconds) ──────────────────────────────────
        let time_units = vec![
            Unit::new("second", "s", vec!["seconds", "sec"], 1.0, "time"),
            Unit::new("minute", "min", vec!["minutes", "mins"], 60.0, "time"),
            Unit::new("hour", "h", vec!["hours", "hrs"], 3600.0, "time"),
            Unit::new("day", "d", vec!["days"], 86400.0, "time"),
            Unit::new("month", "mo", vec!["months", "mth"], 2592000.0, "time"), // Average month (30 days)
            Unit::new("year", "y", vec!["years", "yr", "a"], 31536000.0, "time"), // Standard year
        ];
        
        for unit in time_units { units.insert(unit.symbol.clone(), unit); }
        
        // ─── 2. DISTANCE (Base: meters) ───────────────────────────────
        let distance_units = vec![
            Unit::new("meter", "m", vec!["meters", "metre", "metres"], 1.0, "distance"),
            Unit::new("kilometer", "km", vec!["kilometers", "kilometre", "kmeters"], 1000.0, "distance"),
            Unit::new("centimeter", "cm", vec!["centimeters", "centimetre", "cmeters"], 0.01, "distance"),
            Unit::new("millimeter", "mm", vec!["millimeters", "millimetre", "mmeters"], 0.001, "distance"),
            Unit::new("mile", "mi", vec!["miles"], 1609.344, "distance"),
            Unit::new("foot", "ft", vec!["feet"], 0.3048, "distance"),
            Unit::new("inch", "in", vec!["inches"], 0.0254, "distance"),
        ];
        
        for unit in distance_units { units.insert(unit.symbol.clone(), unit); }
        
        // ─── 3. WEIGHT (Base: kilograms) ──────────────────────────────
        let weight_units = vec![
            Unit::new("kilogram", "kg", vec!["kilograms", "kilo"], 1.0, "weight"),
            Unit::new("gram", "g", vec!["grams"], 0.001, "weight"),
            Unit::new("milligram", "mg", vec!["milligrams"], 0.000001, "weight"),
            Unit::new("pound", "lb", vec!["pounds", "lbs"], 0.453592, "weight"),
            Unit::new("ounce", "oz", vec!["ounces"], 0.0283495, "weight"),
        ];
        
        for unit in weight_units { units.insert(unit.symbol.clone(), unit); }
        
        // ─── 4. TEMPERATURE (Base: Kelvin - special handling required) ─
        let temperature_units = vec![
            Unit::new("celsius", "°c", vec!["c", "celcius"], 0.0, "temperature"),
            Unit::new("fahrenheit", "°f", vec!["f"], 0.0, "temperature"),
            Unit::new("kelvin", "k", vec!["°k"], 1.0, "temperature"),
        ];
        
        for unit in temperature_units { units.insert(unit.symbol.clone(), unit); }
        
        UnitConverter { units }
    }
    
    /** 
     * Resolves a unit from a string, using category context to resolve ambiguities.
     * Example: "m" is treated as 'meter' if the target is 'distance', or 'month' if it matches 'time'.
     */
    fn find_unit_with_context(&self, unit_str: &str, expected_category: Option<&str>) -> Option<&Unit> {
        let unit_lower = unit_str.to_lowercase();
        
        // Handle ambiguous "m" symbol
        if unit_lower == "m" {
            if let Some(category) = expected_category {
                if category == "distance" { return self.units.get("m"); }
                if category == "time" { return self.units.get("mo"); }
            }
            return self.units.get("m"); // Default to meters
        }
        
        self.units.values().find(|u| u.matches(&unit_lower))
    }
    
    /** Converts a value to the standard base unit for its category. */
    fn get_value_in_base(&self, value: f64, unit: &Unit) -> f64 {
        if unit.category == "temperature" {
            match unit.symbol.as_str() {
                "°c" => value + 273.15,
                "°f" => (value - 32.0) * 5.0 / 9.0 + 273.15,
                "k" => value,
                _ => value,
            }
        } else {
            value * unit.to_base
        }
    }

    /** Formats a numerical result with adaptive decimal precision. */
    fn format_result(&self, value: f64, unit: &str) -> String {
        let abs_value = value.abs();
        if abs_value >= 10000.0 { format!("{:.2e} {}", value, unit) }
        else if abs_value >= 1000.0 { format!("{:.0} {}", value, unit) }
        else if abs_value >= 100.0 { format!("{:.1} {}", value, unit) }
        else if abs_value >= 1.0 { format!("{:.2} {}", value, unit) }
        else if abs_value >= 0.01 { format!("{:.3} {}", value, unit) }
        else { format!("{:.6} {}", value, unit) }
    }
    
    /** 
     * Parses a natural language query and performs the conversion.
     * Pattern: [value][unit] ( [value][unit] ... ) [in|to|=] [target_unit]
     * Example: "5km in miles", "1h 30m to minutes"
     */
    pub fn parse_and_convert(&self, query: &str) -> Option<String> {
        let query = query.trim().to_lowercase();
        
        // ─── 1. Identify Target Unit ──────────────────────────────────
        let re_target = Regex::new(r"(?:\s+|^)(?:in|to|[=:])\s*([a-z°]+)$").ok()?;
        let (parts_str, target_unit_str) = if let Some(caps) = re_target.captures(&query) {
            let target = caps.get(1)?.as_str();
            let end_idx = caps.get(0)?.start();
            (&query[..end_idx], target)
        } else {
            return None;
        };

        let target_unit = self.find_unit_with_context(target_unit_str, None)?;
        let target_category = target_unit.category.clone();

        // ─── 2. Parse Source Parts ────────────────────────────────────
        let re_parts = Regex::new(r"([\d.]+)\s*([a-z°]*)").ok()?;
        let mut total_in_base = 0.0;
        let mut last_unit_category = None;
        let mut found_any = false;

        let mut it = re_parts.captures_iter(parts_str).peekable();
        while let Some(caps) = it.next() {
            let val: f64 = caps.get(1)?.as_str().parse().ok()?;
            let mut unit_str = caps.get(2)?.as_str();
            
            // Implicit unit inheritance (e.g., "10h30" -> 30 inherits minutes from 'h')
            if unit_str.is_empty() {
                if let Some(cat) = &last_unit_category {
                    if cat == "time" { unit_str = "min"; }
                }
            }

            if let Some(unit) = self.find_unit_with_context(unit_str, Some(&target_category)) {
                if unit.category == target_category {
                    total_in_base += self.get_value_in_base(val, unit);
                    last_unit_category = Some(unit.category.clone());
                    found_any = true;
                }
            }
        }

        if !found_any { return None; }

        // ─── 3. Final Conversion & Formatting ─────────────────────────
        Some(self.format_final_result(total_in_base, target_unit))
    }

    /** Handles final conversion and applies category-specific formatting rules. */
    fn format_final_result(&self, value_in_base: f64, target_unit: &Unit) -> String {
        // Special logic for Temperatures
        if target_unit.category == "temperature" {
            let result = match target_unit.symbol.as_str() {
                "°c" => value_in_base - 273.15,
                "°f" => (value_in_base - 273.15) * 9.0 / 5.0 + 32.0,
                "k" => value_in_base,
                _ => value_in_base,
            };
            return format!("{:.2} {}", result, target_unit.symbol);
        }

        // Special logic for Time (e.g., 70s -> 1min 10s)
        if target_unit.category == "time" {
            if target_unit.symbol == "min" && value_in_base >= 60.0 && value_in_base % 60.0 != 0.0 {
                let m = (value_in_base / 60.0).floor();
                let s = (value_in_base % 60.0).round();
                return format!("{:.0}min {:.0}s", m, s);
            }
            if target_unit.symbol == "h" && value_in_base >= 3600.0 && value_in_base % 3600.0 != 0.0 {
                let h = (value_in_base / 3600.0).floor();
                let m = ((value_in_base % 3600.0) / 60.0).round();
                if m > 0.0 { return format!("{:.0}h {:.0}min", h, m); }
            }
        }

        let result = value_in_base / target_unit.to_base;
        self.format_result(result, &target_unit.symbol)
    }
}

/** 
 * Public API entry point for the converter.
 * Automatically initializes the engine and parses the query.
 */
pub fn try_convert(query: &str) -> Option<String> {
    let converter = UnitConverter::new();
    converter.parse_and_convert(query)
}