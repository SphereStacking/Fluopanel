//! System event watchers for macOS
//!
//! Monitors system events and emits Tauri events to the frontend.
//! Replaces frontend polling with native event-driven architecture.

#[cfg(target_os = "macos")]
pub mod active_app;
#[cfg(target_os = "macos")]
pub mod battery;
#[cfg(target_os = "macos")]
pub mod media;
#[cfg(target_os = "macos")]
pub mod network;
#[cfg(target_os = "macos")]
pub mod system_monitor;
#[cfg(target_os = "macos")]
pub mod volume;

use tauri::AppHandle;

/// Initialize all system watchers
pub fn init_all(app_handle: AppHandle) {
    #[cfg(target_os = "macos")]
    {
        if let Err(e) = active_app::register(app_handle.clone()) {
            eprintln!("Failed to register active app watcher: {}", e);
        }

        if let Err(e) = volume::register(app_handle.clone()) {
            eprintln!("Failed to register volume watcher: {}", e);
        }

        if let Err(e) = battery::register(app_handle.clone()) {
            eprintln!("Failed to register battery watcher: {}", e);
        }

        if let Err(e) = system_monitor::register(app_handle.clone()) {
            eprintln!("Failed to register system monitor watcher: {}", e);
        }

        if let Err(e) = network::register(app_handle.clone()) {
            eprintln!("Failed to register network watcher: {}", e);
        }

        if let Err(e) = media::register(app_handle) {
            eprintln!("Failed to register media watcher: {}", e);
        }
    }
}
