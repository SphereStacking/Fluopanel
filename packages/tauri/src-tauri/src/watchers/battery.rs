//! Battery Watcher
//!
//! Monitors battery state changes using IOKit Power Source notifications.
//! Emits `battery-changed` event when battery level or charging state changes.

use serde::Serialize;
use std::sync::Once;
use std::thread;
use tauri::{AppHandle, Emitter};

static INIT: Once = Once::new();
static mut APP_HANDLE: Option<AppHandle> = None;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryEvent {
    pub percent: f32,
    pub charging: bool,
    pub time_to_empty: Option<i32>,
    pub time_to_full: Option<i32>,
}

// IOKit types and functions
#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOPSNotificationCreateRunLoopSource(
        callback: extern "C" fn(*mut std::ffi::c_void),
        context: *mut std::ffi::c_void,
    ) -> *mut std::ffi::c_void;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRunLoopGetCurrent() -> *mut std::ffi::c_void;
    fn CFRunLoopAddSource(
        rl: *mut std::ffi::c_void,
        source: *mut std::ffi::c_void,
        mode: *const std::ffi::c_void,
    );
    fn CFRunLoopRun();
}

// kCFRunLoopDefaultMode constant
extern "C" {
    static kCFRunLoopDefaultMode: *const std::ffi::c_void;
}

/// Callback function for power source changes
extern "C" fn power_source_callback(_context: *mut std::ffi::c_void) {
    if let Some(handle) = unsafe { APP_HANDLE.as_ref() } {
        if let Some(event) = get_battery_info() {
            let _ = handle.emit("battery-changed", event);
        }
    }
}

/// Register the battery watcher
pub fn register(app_handle: AppHandle) -> Result<(), String> {
    INIT.call_once(|| {
        unsafe {
            APP_HANDLE = Some(app_handle);
        }

        // Spawn a thread to run the CFRunLoop
        thread::spawn(|| {
            unsafe {
                let source =
                    IOPSNotificationCreateRunLoopSource(power_source_callback, std::ptr::null_mut());

                if !source.is_null() {
                    let run_loop = CFRunLoopGetCurrent();
                    CFRunLoopAddSource(run_loop, source, kCFRunLoopDefaultMode);
                    CFRunLoopRun();
                }
            }
        });
    });

    Ok(())
}

/// Get current battery info using the battery crate
fn get_battery_info() -> Option<BatteryEvent> {
    let manager = battery::Manager::new().ok()?;
    let mut batteries = manager.batteries().ok()?;

    if let Some(Ok(battery)) = batteries.next() {
        use battery::State;

        let percent = battery.state_of_charge().value * 100.0;
        let charging = matches!(battery.state(), State::Charging | State::Full);
        let time_to_empty = battery.time_to_empty().map(|t| (t.value / 60.0) as i32);
        let time_to_full = battery.time_to_full().map(|t| (t.value / 60.0) as i32);

        Some(BatteryEvent {
            percent,
            charging,
            time_to_empty,
            time_to_full,
        })
    } else {
        None
    }
}
