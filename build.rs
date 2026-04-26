/* 
 * APPSearch Build Script
 * 
 * This script runs automatically before the main compilation process.
 * Its main responsibilities are:
 * 1. Embedding Windows-specific resources (like the app icon).
 * 2. Compiling the Slint UI definition into Rust code.
 * 3. Defining rebuild triggers for Cargo.
 */

fn main() {
    // ─── 1. WINDOWS RESOURCE COMPILATION ────────────────────────────────
    // If we are targeting Windows, embed the application icon into the binary.
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/logo.ico");
        res.compile().expect("Failed to compile Windows resources (icon)");
    }

    // ─── 2. REBUILD TRIGGERS ───────────────────────────────────────────
    // Tell Cargo to rerun this build script if the following files change.
    println!("cargo:rerun-if-changed=assets/logo.ico");
    println!("cargo:rerun-if-changed=ui/app.slint");

    // ─── 3. SLINT UI COMPILATION ───────────────────────────────────────
    // Compiles the .slint markup into Rust code accessible via slint::include_modules!().
    slint_build::compile("ui/app.slint").expect("Failed to compile Slint UI");
}
