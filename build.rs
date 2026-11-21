//! Build script: embeds assets/echofs.ico into the Windows executable so the
//! exe itself (Explorer listing, pinned taskbar shortcuts, Properties dialog)
//! carries the app icon instead of the generic system one.
//!
//! The live window/taskbar icon is set separately by `gui.rs` via eframe's
//! viewport icon. Failure is not fatal — a warning is emitted and the exe
//! simply keeps the default icon (e.g. hosts without a Windows SDK `rc.exe`,
//! or windows-gnu without `windres`).
//!
//! NOTE on gating: build.rs is compiled for the HOST platform, and Cargo
//! evaluates target-specific `[build-dependencies]` gates against the host
//! too (Cargo book, Platform-Specific Dependencies). The `cfg` below
//! therefore mirrors the
//! `[target.'cfg(target_os = "windows")'.build-dependencies]` gate in
//! Cargo.toml: the crate is present exactly where this code is compiled, on
//! every host — including Linux/macOS hosts and cross builds, where the
//! function is compiled out and no icon is embedded.

fn main() {
    println!("cargo:rerun-if-changed=assets/echofs.ico");

    embed_icon();
}

#[cfg(target_os = "windows")]
fn embed_icon() {
    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/echofs.ico");
    if let Err(e) = res.compile() {
        println!("cargo:warning=echofs: could not embed the application icon into the exe: {e}");
    }
}

#[cfg(not(target_os = "windows"))]
fn embed_icon() {}
