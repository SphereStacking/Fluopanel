//! Hover Focus (Autoraise) Module
//!
//! Automatically focuses windows when the cursor enters their bounds.
//! Uses NSEvent global monitoring for mouse movement detection.

use once_cell::sync::OnceCell;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, WebviewWindow};

static STATE: OnceCell<Mutex<HoverFocusState>> = OnceCell::new();

struct HoverFocusState {
    app_handle: Option<AppHandle>,
    last_focused_label: Option<String>,
}

/// Initialize the hover focus system
pub fn init(app_handle: AppHandle) {
    STATE.get_or_init(|| {
        Mutex::new(HoverFocusState {
            app_handle: Some(app_handle),
            last_focused_label: None,
        })
    });

    // Start the mouse monitor on macOS
    #[cfg(target_os = "macos")]
    start_mouse_monitor();
}

/// Check if a point is inside a window and return the window label
#[cfg(target_os = "macos")]
fn get_window_at_point(x: f64, y: f64) -> Option<String> {
    let state = STATE.get()?.lock().ok()?;
    let app_handle = state.app_handle.as_ref()?;

    // Get all webview windows
    let windows: std::collections::HashMap<String, WebviewWindow> = app_handle.webview_windows();

    for (label, window) in windows.iter() {
        // Skip popover windows
        if label.starts_with("popover-") {
            continue;
        }

        // Skip main coordinator window
        if label == "main" {
            continue;
        }

        // Check if window is visible
        if !window.is_visible().unwrap_or(false) {
            continue;
        }

        // Get window position and size
        if let (Ok(pos), Ok(size)) = (window.outer_position(), window.outer_size()) {
            let win_x = pos.x as f64;
            let win_y = pos.y as f64;
            let win_w = size.width as f64;
            let win_h = size.height as f64;

            // Check if point is inside window bounds
            if x >= win_x && x <= win_x + win_w && y >= win_y && y <= win_y + win_h {
                return Some(label.clone());
            }
        }
    }

    None
}

/// Focus a window by label
#[cfg(target_os = "macos")]
fn focus_window(label: &str) {
    let state_lock = match STATE.get() {
        Some(s) => s,
        None => return,
    };

    let state = match state_lock.lock() {
        Ok(s) => s,
        Err(_) => return,
    };

    if let Some(app_handle) = state.app_handle.as_ref() {
        if let Some(window) = app_handle.get_webview_window(label) {
            let _ = window.set_focus();
        }
    }
}

/// Start the global mouse monitor (macOS only)
#[cfg(target_os = "macos")]
fn start_mouse_monitor() {
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2::{msg_send, ClassType};
    use objc2_app_kit::{NSEvent, NSEventMask};
    use objc2_foundation::NSPoint;
    use std::sync::Once;

    static START_MONITOR: Once = Once::new();

    START_MONITOR.call_once(|| {
        std::thread::spawn(|| {
            // Give the app time to initialize windows
            std::thread::sleep(std::time::Duration::from_secs(1));

            unsafe {
                // Create event handler block
                let handler = block2::StackBlock::new(|_event: *mut AnyObject| {
                    // Get mouse location in screen coordinates
                    let mouse_location: NSPoint = NSEvent::mouseLocation();

                    // macOS uses bottom-left origin, convert to top-left
                    let screens: *const AnyObject = msg_send![
                        objc2_app_kit::NSScreen::class(),
                        screens
                    ];
                    if screens.is_null() {
                        return;
                    }

                    let main_screen: *const AnyObject = msg_send![screens, firstObject];
                    if main_screen.is_null() {
                        return;
                    }

                    let frame: objc2_foundation::NSRect = msg_send![main_screen, frame];
                    let screen_height = frame.size.height;

                    // Convert Y coordinate (flip from bottom-left to top-left)
                    let x = mouse_location.x;
                    let y = screen_height - mouse_location.y;

                    // Check which window is under cursor
                    if let Some(label) = get_window_at_point(x, y) {
                        // Get last focused label
                        let last_label = STATE
                            .get()
                            .and_then(|s| s.lock().ok())
                            .and_then(|s| s.last_focused_label.clone());

                        // Only focus if it's a different window
                        if last_label.as_ref() != Some(&label) {
                            focus_window(&label);

                            // Update last focused
                            if let Some(state_lock) = STATE.get() {
                                if let Ok(mut state) = state_lock.lock() {
                                    state.last_focused_label = Some(label);
                                }
                            }
                        }
                    } else {
                        // Cursor not over any window, clear last focused
                        if let Some(state_lock) = STATE.get() {
                            if let Ok(mut state) = state_lock.lock() {
                                state.last_focused_label = None;
                            }
                        }
                    }
                });

                // Register global event monitor for mouse moved events
                let mask = NSEventMask::MouseMoved;
                let _monitor: Option<Retained<AnyObject>> = msg_send![
                    NSEvent::class(),
                    addGlobalMonitorForEventsMatchingMask: mask.0,
                    handler: &*handler
                ];

                // Keep the monitor alive
                if let Some(monitor) = _monitor {
                    std::mem::forget(monitor);
                }

                let _ = handler;
            }
        });
    });
}
