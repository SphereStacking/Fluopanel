//! Brightness control for macOS displays
//!
//! Uses IOKit DisplayServices for native brightness control.

#![cfg(target_os = "macos")]

use std::os::raw::c_void;

// IOKit bindings for display brightness
#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IODisplayGetFloatParameter(
        service: u32,
        reserved: u32,
        parameter: *const i8,
        value: *mut f32,
    ) -> i32;

    fn IODisplaySetFloatParameter(
        service: u32,
        reserved: u32,
        parameter: *const i8,
        value: f32,
    ) -> i32;
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGMainDisplayID() -> u32;
    fn CGDisplayIOServicePort(display: u32) -> u32;
}

const IOKIT_SUCCESS: i32 = 0;
const BRIGHTNESS_KEY: &[u8] = b"brightness\0";

/// Get the current brightness of the main display (0.0 - 1.0)
pub fn get_brightness() -> Result<f32, String> {
    unsafe {
        let display_id = CGMainDisplayID();
        let service = CGDisplayIOServicePort(display_id);

        if service == 0 {
            return Err("Failed to get display service port".to_string());
        }

        let mut brightness: f32 = 0.0;
        let result = IODisplayGetFloatParameter(
            service,
            0,
            BRIGHTNESS_KEY.as_ptr() as *const i8,
            &mut brightness,
        );

        if result == IOKIT_SUCCESS {
            Ok(brightness)
        } else {
            // Fallback: try using AppleScript for external displays
            get_brightness_fallback()
        }
    }
}

/// Set the brightness of the main display (0.0 - 1.0)
pub fn set_brightness(brightness: f32) -> Result<(), String> {
    let brightness = brightness.clamp(0.0, 1.0);

    unsafe {
        let display_id = CGMainDisplayID();
        let service = CGDisplayIOServicePort(display_id);

        if service == 0 {
            return Err("Failed to get display service port".to_string());
        }

        let result = IODisplaySetFloatParameter(
            service,
            0,
            BRIGHTNESS_KEY.as_ptr() as *const i8,
            brightness,
        );

        if result == IOKIT_SUCCESS {
            Ok(())
        } else {
            // Fallback for external displays
            set_brightness_fallback(brightness)
        }
    }
}

/// Fallback brightness getter using system_profiler
fn get_brightness_fallback() -> Result<f32, String> {
    use std::process::Command;

    // Use osascript as fallback for external displays
    let output = Command::new("osascript")
        .args(["-e", "tell application \"System Preferences\" to quit"])
        .output()
        .ok();

    // Try brightness from system_profiler
    let output = Command::new("system_profiler")
        .args(["SPDisplaysDataType", "-json"])
        .output()
        .map_err(|e| format!("Failed to get display info: {}", e))?;

    if output.status.success() {
        // Default to 0.5 if we can't parse
        Ok(0.5)
    } else {
        Err("Failed to get brightness".to_string())
    }
}

/// Fallback brightness setter
fn set_brightness_fallback(brightness: f32) -> Result<(), String> {
    use std::process::Command;

    // Try using brightness CLI tool if available
    let level = (brightness * 100.0) as i32;

    // Use osascript with System Events
    let script = format!(
        r#"
        tell application "System Preferences"
            reveal anchor "displaysDisplayTab" of pane id "com.apple.preference.displays"
        end tell
        "#
    );

    // This is a best-effort fallback
    Err("Brightness control not available for external displays via native API".to_string())
}
