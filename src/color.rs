use regex::Regex;

/* 
 * Color Module
 * 
 * Handles parsing and validation of various color formats (Hex, RGB, RGBA).
 * Used for the real-time color preview and the advanced color picker.
 */

/** Holds the parsed color data in multiple formats for UI display. */
pub struct ColorResult {
    pub hex: String,          // e.g., "#FF0000"
    pub rgb: String,          // e.g., "rgb(255, 0, 0)"
    pub slint_color: slint::Color, // Native Slint color type for the UI
}

/** 
 * Attempts to parse a string as a color.
 * Supports:
 * - Full Hex: #RRGGBB or RRGGBB
 * - Short Hex: #RGB or RGB
 * - CSS RGB: rgb(r, g, b)
 * - CSS RGBA: rgba(r, g, b, a)
 */
pub fn try_parse_color(query: &str) -> Option<ColorResult> {
    let query = query.trim().to_uppercase();
    
    // ─── 1. Full Hex Format (#FF0000 or FF0000) ──────────────────────
    let re_hex = Regex::new(r"^#?([0-9A-F]{6})$").ok()?;
    if let Some(caps) = re_hex.captures(&query) {
        let hex = caps.get(1)?.as_str();
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        
        return Some(ColorResult {
            hex: format!("#{}", hex),
            rgb: format!("rgb({}, {}, {})", r, g, b),
            slint_color: slint::Color::from_rgb_u8(r, g, b),
        });
    }
    
    // ─── 2. Short Hex Format (#F00 or F00) ───────────────────────────
    let re_hex_short = Regex::new(r"^#?([0-9A-F]{3})$").ok()?;
    if let Some(caps) = re_hex_short.captures(&query) {
        let hex = caps.get(1)?.as_str();
        // Expand short hex (e.g., F -> FF)
        let r_str = format!("{}{}", &hex[0..1], &hex[0..1]);
        let g_str = format!("{}{}", &hex[1..2], &hex[1..2]);
        let b_str = format!("{}{}", &hex[2..3], &hex[2..3]);
        
        let r = u8::from_str_radix(&r_str, 16).ok()?;
        let g = u8::from_str_radix(&g_str, 16).ok()?;
        let b = u8::from_str_radix(&b_str, 16).ok()?;
        
        return Some(ColorResult {
            hex: format!("#{}{}{}", &hex[0..1], &hex[1..2], &hex[2..3]),
            rgb: format!("rgb({}, {}, {})", r, g, b),
            slint_color: slint::Color::from_rgb_u8(r, g, b),
        });
    }

    // ─── 3. RGB/RGBA Format (rgb(20, 20, 20) or rgba(20, 20, 20, 0.5)) ──
    let re_rgb = Regex::new(r"RGBA?\s*\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)(?:\s*,\s*([\d.]+))?\s*\)").ok()?;
    if let Some(caps) = re_rgb.captures(&query) {
        let r: u8 = caps.get(1)?.as_str().parse().ok()?;
        let g: u8 = caps.get(2)?.as_str().parse().ok()?;
        let b: u8 = caps.get(3)?.as_str().parse().ok()?;
        
        // Handle optional alpha channel
        let a = if let Some(a_cap) = caps.get(4) {
            let a_str = a_cap.as_str();
            if a_str.contains('.') {
                // Percentage-based alpha (0.0 to 1.0)
                (a_str.parse::<f64>().ok()?.clamp(0.0, 1.0) * 255.0) as u8
            } else {
                // Byte-based alpha (0 to 255)
                a_str.parse::<u8>().ok()?
            }
        } else {
            255 // Full opacity by default
        };
        
        return Some(ColorResult {
            hex: if a == 255 { 
                format!("#{:02X}{:02X}{:02X}", r, g, b) 
            } else { 
                format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a) 
            },
            rgb: format!("rgba({}, {}, {}, {:.2})", r, g, b, a as f64 / 255.0),
            slint_color: slint::Color::from_argb_u8(a, r, g, b),
        });
    }

    None
}
