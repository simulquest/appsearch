use std::process::Command;

/* 
 * System Control Module
 * 
 * Provides integration with low-level Windows system functions.
 * Uses a combination of 'rundll32.exe' and PowerShell scripts to 
 * manage hardware states (Volume, Brightness) and power states (Lock, Sleep).
 */

/** Locks the workstation immediately. */
pub fn lock() {
    let _ = Command::new("rundll32.exe")
        .args(["user32.dll,LockWorkStation"])
        .spawn();
}

/** Puts the computer into a sleep state. */
pub fn sleep() {
    let _ = Command::new("rundll32.exe")
        .args(["powrprof.dll,SetSuspendState", "0,1,0"])
        .spawn();
}

/** 
 * Adjusts the system volume level.
 * Strategy: Uses a PowerShell script to simulate multimedia key presses (SendKeys).
 * This approach is extremely robust as it works across different audio drivers.
 */
pub fn set_volume(percent: f32) -> anyhow::Result<()> {
    // Each volume step press is typically 2%.
    let steps = (percent / 2.0).round() as i32;
    
    // PowerShell script:
    // 1. Create a COM WScript.Shell object.
    // 2. Press 'Volume Down' (char 174) 50 times to ensure we start at 0%.
    // 3. Press 'Volume Up' (char 175) the required number of times.
    let ps_script = format!(
        "$w = New-Object -ComObject WScript.Shell; \
         for($i=0; $i -lt 50; $i++) {{ $w.SendKeys([char]174) }}; \
         for($i=0; $i -lt {}; $i++) {{ $w.SendKeys([char]175) }}",
        steps
    );
    
    Command::new("powershell")
        .args(["-Command", &ps_script])
        .spawn()?;
    Ok(())
}

/** 
 * Sets the monitor brightness level via WMI (Windows Management Instrumentation).
 */
pub fn set_brightness(percent: u8) -> anyhow::Result<()> {
    let cmd = format!(
        "(Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightnessMethods).WmiSetBrightness(1, {})",
        percent
    );
    Command::new("powershell")
        .args(["-Command", &cmd])
        .spawn()?;
    Ok(())
}

/** 
 * Parses a string to identify system commands for real-time UI feedback.
 * Supported: "lock", "sleep", "vol [x]", "lum [x]", "bright [x]".
 */
pub fn try_parse_system_command(text: &str) -> Option<(String, String)> {
    let t = text.trim().to_lowercase();
    
    // Exact match commands
    if t == "lock" { return Some(("SYSTEM".to_string(), "Lock Workstation".to_string())); }
    if t == "sleep" { return Some(("SYSTEM".to_string(), "Put PC to Sleep".to_string())); }
    
    // Volume Control
    if t.starts_with("vol ") {
        let val_str = t.trim_start_matches("vol ").trim();
        if let Ok(val) = val_str.parse::<f32>() {
            let clamped = val.clamp(0.0, 100.0);
            return Some(("VOLUME".to_string(), format!("Set Volume to {}%", clamped)));
        }
    }
    
    // Brightness Control
    if t.starts_with("lum ") || t.starts_with("bright ") {
        let val_str = if t.starts_with("lum ") {
            t.trim_start_matches("lum ").trim()
        } else {
            t.trim_start_matches("bright ").trim()
        };
        if let Ok(val) = val_str.parse::<u8>() {
            let clamped = val.clamp(0, 100);
            return Some(("BRIGHTNESS".to_string(), format!("Set Brightness to {}%", clamped)));
        }
    }

    None
}

/** Executes the identified system command. Returns true if a command was handled. */
pub fn execute_system_command(text: &str) -> bool {
    let t = text.trim().to_lowercase();
    
    if t == "lock" {
        lock();
        true
    } else if t == "sleep" {
        sleep();
        true
    } else if t.starts_with("vol ") {
        let val_str = t.trim_start_matches("vol ").trim();
        if let Ok(val) = val_str.parse::<f32>() {
            let _ = set_volume(val.clamp(0.0, 100.0));
            true
        } else { false }
    } else if t.starts_with("lum ") || t.starts_with("bright ") {
        let val_str = if t.starts_with("lum ") {
            t.trim_start_matches("lum ").trim()
        } else {
            t.trim_start_matches("bright ").trim()
        };
        if let Ok(val) = val_str.parse::<u8>() {
            let _ = set_brightness(val.clamp(0, 100));
            true
        } else { false }
    } else {
        false
    }
}
