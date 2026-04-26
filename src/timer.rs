use regex::Regex;
use std::time::Duration;
use std::os::windows::process::CommandExt;
use slint::ComponentHandle;

/* 
 * Timer Module
 * 
 * Provides a simple countdown timer functionality.
 * Timers run in the background and trigger a Windows system notification 
 * when the duration expires.
 */

/** 
 * Parses a string to identify timer requests for real-time UI feedback.
 * Supported Format: "timer [value][unit]"
 * Units: s, sec, m, min, h
 * Example: "timer 5m", "timer 10 sec"
 */
pub fn try_parse_timer(text: &str) -> Option<(String, String)> {
    let t = text.trim().to_lowercase();
    if !t.starts_with("timer ") {
        return None;
    }
    
    let duration_str = t.trim_start_matches("timer ").trim();
    
    // Regex to capture numeric value and unit suffix
    let re = Regex::new(r"(\d+)\s*(s|m|h|sec|min)").ok()?;
    let caps = re.captures(duration_str)?;
    
    let val: u64 = caps.get(1)?.as_str().parse().ok()?;
    let unit = caps.get(2)?.as_str();
    
    let label = match unit {
        "s" | "sec" => format!("{} seconds", val),
        "m" | "min" => format!("{} minutes", val),
        "h" => format!("{} hours", val),
        _ => "time".to_string(),
    };
    
    Some(("TIMER".to_string(), format!("Start timer for {}", label)))
}

/** 
 * Spawns a background thread to track the timer duration.
 * When the time is up, a PowerShell script is used to display a Windows Balloon Tip notification.
 */
pub fn start_timer<T: ComponentHandle + 'static>(text: &str, _ui_handle: slint::Weak<T>) -> bool {
    let t = text.trim().to_lowercase();
    if !t.starts_with("timer ") {
        return false;
    }
    
    let duration_str = t.trim_start_matches("timer ").trim();
    let re = Regex::new(r"(\d+)\s*(s|m|h|sec|min)").ok();
    if let Some(re) = re {
        if let Some(caps) = re.captures(duration_str) {
            let val: u64 = caps.get(1).unwrap().as_str().parse().unwrap_or(0);
            let unit = caps.get(2).unwrap().as_str();
            
            let seconds = match unit {
                "s" | "sec" => val,
                "m" | "min" => val * 60,
                "h" => val * 3600,
                _ => 0,
            };
            
            if seconds > 0 {
                // Background execution to avoid blocking the UI thread
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_secs(seconds));
                    
                    // Display a native Windows notification using PowerShell
                    // This creates a NotifyIcon COM object and shows a BalloonTip.
                    let _ = std::process::Command::new("powershell")
                        .args(["-Command", "Add-Type -AssemblyName System.Windows.Forms; $notify = New-Object System.Windows.Forms.NotifyIcon; $notify.Icon = [System.Drawing.SystemIcons]::Information; $notify.Visible = $true; $notify.ShowBalloonTip(5000, 'APPSearch Timer', 'Your timer has finished!', [System.Windows.Forms.ToolTipIcon]::Info); Start-Sleep -s 6; $notify.Dispose()"])
                        .creation_flags(0x08000000)
                        .spawn();
                });
                return true;
            }
        }
    }
    false
}
