use std::path::Path;
use anyhow::Result;

/* 
 * Launcher Module
 * 
 * A simple abstraction layer for opening files, directories, or URLs 
 * using the operating system's default handler.
 */

/** 
 * Launches the specified path using the system's default application.
 * This can be an executable, a document, a folder, or a web URL.
 */
pub fn launch(path: &Path) -> Result<()> {
    // Delegates to the 'open' crate which handles OS-specific logic 
    // (e.g., ShellExecute on Windows, open on macOS, xdg-open on Linux)
    open::that(path)?;
    Ok(())
}
