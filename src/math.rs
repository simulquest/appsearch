use regex::Regex;

/* 
 * Math Module
 * 
 * Provides a real-time calculator for mathematical expressions.
 * Powered by the 'meval' library for parsing and evaluation.
 * Supports standard operations, trigonometric functions, and custom shortcuts.
 */

/** 
 * Attempts to evaluate a string as a mathematical expression.
 * 
 * Features:
 * - Basic operators: +, -, *, /, ^
 * - Functions: sqrt, ln, exp, cos, sin, tan, etc.
 * - Constants: pi, e
 * - Special syntax: logB(X) is automatically rewritten to ln(X)/ln(B)
 */
pub fn try_evaluate(query: &str) -> Option<String> {
    let mut query = query.trim().to_lowercase();
    
    // ─── 1. Syntax Pre-processing ────────────────────────────────────
    
    // Rewrite logB(X) -> (ln(X)/ln(B))
    let re_log = Regex::new(r"log(\d+)\((.+)\)").ok()?;
    query = re_log.replace_all(&query, "(ln($2)/ln($1))").to_string();

    // Map keywords
    query = query.replace("exponential", "exp");

    // ─── 2. Heuristic Check ─────────────────────────────────────────
    // Only attempt evaluation if it actually looks like math to avoid false positives.
    let math_keywords = ["sqrt", "ln", "exp", "pi", "cos", "sin", "tan", "acos", "asin", "atan", "floor", "ceil", "abs"];
    let has_keyword = math_keywords.iter().any(|&k| query.contains(k));
    let has_operator = query.chars().any(|c| "+-*/^()".contains(c));

    if !has_operator && !has_keyword && query != "e" {
        return None;
    }

    // ─── 3. Evaluation ──────────────────────────────────────────────
    match meval::eval_str(&query) {
        Ok(result) => {
            // Ensure the result is valid (e.g., not dividing by zero)
            if result.is_finite() {
                // Adaptive formatting: suppress decimals for integers
                if result.fract() == 0.0 {
                    Some(format!("{:.0}", result))
                } else {
                    // Truncate trailing zeros for a cleaner UI
                    Some(format!("{:.4}", result).trim_end_matches('0').trim_end_matches('.').to_string())
                }
            } else {
                None
            }
        },
        Err(_) => None,
    }
}
