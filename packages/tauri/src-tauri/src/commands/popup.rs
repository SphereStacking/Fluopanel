use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use once_cell::sync::Lazy;
use tauri::{command, AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri::async_runtime::JoinHandle;

// macOS-specific imports for window number monitoring
#[cfg(target_os = "macos")]
use cocoa::base::nil;
#[cfg(target_os = "macos")]
use cocoa::foundation::NSPoint;
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};

/// Trigger element bounds in screen coordinates (macOS bottom-left origin)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Options for popup positioning when trigger is hovered
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PopupTriggerOptions {
    pub popup_width: f64,
    pub popup_height: f64,
    #[serde(default)]
    pub popup_align: PopupAlign,
    #[serde(default = "default_offset_y")]
    pub popup_offset_y: f64,
}

fn default_offset_y() -> f64 {
    8.0
}

/// Registered trigger for global mouse monitoring
#[derive(Debug, Clone)]
struct RegisteredTrigger {
    id: String,
    bounds: TriggerBounds,
    popup_options: PopupTriggerOptions,
    /// Whether mouse is currently over this trigger
    is_hovering: bool,
}

/// Store registered triggers for global mouse monitoring
static REGISTERED_TRIGGERS: Lazy<Mutex<HashMap<String, RegisteredTrigger>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Global trigger monitor task handle
static TRIGGER_MONITOR_TASK: Lazy<Mutex<Option<JoinHandle<()>>>> =
    Lazy::new(|| Mutex::new(None));

/// Padding around trigger bounds (0 = exact button size, increase for easier hover)
/// 15px is a balance between accuracy and usability
const TRIGGER_PADDING: f64 = 15.0;

/// Hover coordination state for popup windows
struct HoverCoordinator {
    close_timer: Option<JoinHandle<()>>,
    /// Window number for this popup (macOS only, used for identification)
    #[cfg(target_os = "macos")]
    window_number: i64,
    /// Background monitor task handle
    monitor_task: Option<JoinHandle<()>>,
    /// Trigger element bounds (macOS coordinate system, bottom-left origin)
    trigger_bounds: TriggerBounds,
}

/// Store hover coordinators for popups
static HOVER_COORDINATORS: Lazy<Mutex<HashMap<String, HoverCoordinator>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Delay before closing popup when mouse leaves both trigger and popup (ms)
const HOVER_CLOSE_DELAY_MS: u64 = 150;

/// Interval for window number monitoring (ms)
const WINDOW_MONITOR_INTERVAL_MS: u64 = 100;

/// Get current mouse location in screen coordinates (macOS)
#[cfg(target_os = "macos")]
fn get_mouse_location() -> (f64, f64) {
    unsafe {
        let location: NSPoint = msg_send![class!(NSEvent), mouseLocation];
        (location.x, location.y)
    }
}

/// Get the window number at a screen point (macOS)
/// Returns the window number of the frontmost window at the given point
#[cfg(target_os = "macos")]
fn get_window_number_at_point(x: f64, y: f64) -> i64 {
    unsafe {
        let point = NSPoint::new(x, y);
        let window_number: isize = msg_send![
            class!(NSWindow),
            windowNumberAtPoint:point
            belowWindowWithWindowNumber:0_isize
        ];
        window_number as i64
    }
}

/// Check if cursor is within trigger bounds (with optional padding)
/// Both cursor and trigger should be in macOS coordinate system (bottom-left origin)
fn is_cursor_over_trigger(mouse_x: f64, mouse_y: f64, trigger: &TriggerBounds) -> bool {
    // trigger coordinates are already in macOS coordinate system
    // TRIGGER_PADDING allows expanding the hit area (0 = strict matching)
    mouse_x >= trigger.x - TRIGGER_PADDING
        && mouse_x <= trigger.x + trigger.width + TRIGGER_PADDING
        && mouse_y >= trigger.y - TRIGGER_PADDING
        && mouse_y <= trigger.y + trigger.height + TRIGGER_PADDING
}

/// Popup alignment relative to anchor element
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PopupAlign {
    Start,
    #[default]
    Center,
    End,
}

/// Popup mode determines open/close triggers
#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PopupMode {
    #[default]
    Toggle,
    Hover,
    HoverSticky,
}

/// Popup anchor position (from trigger element's getBoundingClientRect)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PopupAnchor {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Open popup response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PopupInfo {
    pub id: String,
    pub label: String,
}

/// Get monitor info containing the anchor point
fn get_monitor_at_point(app: &AppHandle, x: f64, y: f64) -> Result<(f64, f64, f64, f64), String> {
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;

    if monitors.is_empty() {
        return Err("No monitors available".to_string());
    }

    // Find monitor containing the point
    for monitor in &monitors {
        let pos = monitor.position();
        let size = monitor.size();
        let scale = monitor.scale_factor();

        let mx = pos.x as f64 / scale;
        let my = pos.y as f64 / scale;
        let mw = size.width as f64 / scale;
        let mh = size.height as f64 / scale;

        if x >= mx && x < mx + mw && y >= my && y < my + mh {
            return Ok((mx, my, mw, mh));
        }
    }

    // Fallback to primary monitor
    let monitor = app
        .primary_monitor()
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| monitors[0].clone());

    let pos = monitor.position();
    let size = monitor.size();
    let scale = monitor.scale_factor();

    Ok((
        pos.x as f64 / scale,
        pos.y as f64 / scale,
        size.width as f64 / scale,
        size.height as f64 / scale,
    ))
}

/// Start window number monitor for popup (macOS)
/// This monitors if the cursor is still over the popup window (and optionally trigger element)
/// - For Hover mode: checks both popup and trigger
/// - For Toggle mode: checks only popup (trigger_bounds is ignored)
#[cfg(target_os = "macos")]
fn start_window_number_monitor(popup_id: String, trigger_bounds: Option<TriggerBounds>, app: AppHandle) -> JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        // Wait a bit for the window to be fully created
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Try to get the window number (may need a few attempts)
        let mut popup_window_number: Option<i64> = None;
        for _ in 0..10 {
            let label = format!("popup-{}", popup_id);
            if let Some(webview_window) = app.get_webview_window(&label) {
                // Try to get window number via NSWindow
                use cocoa::base::id;
                use std::sync::atomic::{AtomicI64, Ordering};
                let window_number = std::sync::Arc::new(AtomicI64::new(-1));
                let window_number_clone = window_number.clone();

                let _ = webview_window.with_webview(move |webview| {
                    unsafe {
                        let ns_window_ptr = webview.ns_window();
                        if !ns_window_ptr.is_null() {
                            let ns_window: id = ns_window_ptr as id;
                            let num: isize = msg_send![ns_window, windowNumber];
                            window_number_clone.store(num as i64, Ordering::SeqCst);
                        }
                    }
                });

                // Give the closure time to execute on main thread
                tokio::time::sleep(Duration::from_millis(50)).await;

                let num = window_number.load(Ordering::SeqCst);
                if num > 0 {
                    popup_window_number = Some(num);
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let popup_window_number = match popup_window_number {
            Some(n) => {
                eprintln!("[DEBUG] Got window number {} for popup {}", n, popup_id);
                n
            },
            None => {
                eprintln!("Warning: Could not get window number for popup {}, monitor disabled", popup_id);
                return;
            }
        };

        // Update the coordinator with the window number
        if let Ok(mut coordinators) = HOVER_COORDINATORS.lock() {
            if let Some(coordinator) = coordinators.get_mut(&popup_id) {
                coordinator.window_number = popup_window_number;
            }
        }

        if let Some(ref tb) = trigger_bounds {
            eprintln!("[DEBUG] Trigger bounds: x={}, y={}, w={}, h={}",
                tb.x, tb.y, tb.width, tb.height);
        } else {
            eprintln!("[DEBUG] No trigger bounds (toggle mode)");
        }

        // Main monitoring loop
        let mut consecutive_outside_count = 0;
        const REQUIRED_OUTSIDE_COUNT: u32 = 2; // Require 2 consecutive checks outside before scheduling close

        loop {
            tokio::time::sleep(Duration::from_millis(WINDOW_MONITOR_INTERVAL_MS)).await;

            // Check if popup still exists in coordinator
            let popup_exists = {
                HOVER_COORDINATORS.lock()
                    .map(|c| c.contains_key(&popup_id))
                    .unwrap_or(false)
            };

            if !popup_exists {
                break;
            }

            // Get mouse location (macOS coordinate system, bottom-left origin)
            let (mouse_x, mouse_y) = get_mouse_location();

            // Check if cursor is over popup window (via window number)
            let window_at_point = get_window_number_at_point(mouse_x, mouse_y);
            let is_over_popup = window_at_point == popup_window_number;

            // Check if cursor is over trigger element (via coordinate bounds)
            // Only for hover mode (when trigger_bounds is Some)
            let is_over_trigger = trigger_bounds.as_ref()
                .map(|tb| is_cursor_over_trigger(mouse_x, mouse_y, tb))
                .unwrap_or(false);

            if is_over_popup || is_over_trigger {
                // Cursor is over popup or trigger - reset counter and cancel any close timer
                consecutive_outside_count = 0;
                cancel_close_timer(&popup_id);
            } else {
                // Cursor is outside popup (and trigger if hover mode)
                consecutive_outside_count += 1;

                // Schedule close after consecutive outside checks (debounce for quick movements)
                if consecutive_outside_count >= REQUIRED_OUTSIDE_COUNT {
                    let should_schedule = {
                        HOVER_COORDINATORS.lock()
                            .map(|c| c.get(&popup_id).map(|coord| coord.close_timer.is_none()).unwrap_or(false))
                            .unwrap_or(false)
                    };
                    if should_schedule {
                        eprintln!("[DEBUG] Scheduling close for popup {} (count={})", popup_id, consecutive_outside_count);
                        schedule_hover_close(popup_id.clone(), app.clone());
                    }
                }
            }
        }
    })
}

/// Initialize hover coordinator for a popup (macOS version with window number monitor)
/// trigger_bounds: Some for hover mode (checks both popup and trigger), None for toggle mode (checks only popup)
#[cfg(target_os = "macos")]
fn init_hover_coordinator(popup_id: &str, trigger_bounds: Option<TriggerBounds>, app: AppHandle) {
    let tb_clone = trigger_bounds.clone();
    let monitor_task = start_window_number_monitor(popup_id.to_string(), tb_clone, app);

    if let Ok(mut coordinators) = HOVER_COORDINATORS.lock() {
        coordinators.insert(popup_id.to_string(), HoverCoordinator {
            close_timer: None,
            window_number: 0, // Will be set by monitor task after window is ready
            monitor_task: Some(monitor_task),
            trigger_bounds: trigger_bounds.unwrap_or(TriggerBounds { x: 0.0, y: 0.0, width: 0.0, height: 0.0 }),
        });
    }
}

/// Initialize hover coordinator for a popup (non-macOS fallback)
#[cfg(not(target_os = "macos"))]
fn init_hover_coordinator(popup_id: &str, trigger_bounds: Option<TriggerBounds>, _app: AppHandle) {
    if let Ok(mut coordinators) = HOVER_COORDINATORS.lock() {
        coordinators.insert(popup_id.to_string(), HoverCoordinator {
            close_timer: None,
            monitor_task: None,
            trigger_bounds: trigger_bounds.unwrap_or(TriggerBounds { x: 0.0, y: 0.0, width: 0.0, height: 0.0 }),
        });
    }
}

/// Cleanup hover coordinator for a popup
fn cleanup_hover_coordinator(popup_id: &str) {
    if let Ok(mut coordinators) = HOVER_COORDINATORS.lock() {
        if let Some(coordinator) = coordinators.remove(popup_id) {
            // Cancel any pending close timer
            if let Some(timer) = coordinator.close_timer {
                timer.abort();
            }
            // Cancel monitor task
            if let Some(monitor) = coordinator.monitor_task {
                monitor.abort();
            }
        }
    }
}

/// Schedule popup close after delay (called when mouse leaves both trigger and popup)
/// The close timer can be cancelled by cancel_close_timer if cursor re-enters
fn schedule_hover_close(popup_id: String, app: AppHandle) {
    let popup_id_for_timer = popup_id.clone();

    let timer = tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(HOVER_CLOSE_DELAY_MS)).await;

        // Check if timer was cancelled (coordinator might have been removed or timer replaced)
        let should_close = {
            HOVER_COORDINATORS.lock()
                .map(|c| c.contains_key(&popup_id_for_timer))
                .unwrap_or(false)
        };

        if should_close {
            eprintln!("[DEBUG] Closing popup {}", popup_id_for_timer);
            let label = format!("popup-{}", popup_id_for_timer);
            if let Some(window) = app.get_webview_window(&label) {
                let _ = window.close();
                let _ = app.emit("popup-closed", &popup_id_for_timer);
            }
            cleanup_hover_coordinator(&popup_id_for_timer);
        }
    });

    // Store the timer handle
    if let Ok(mut coordinators) = HOVER_COORDINATORS.lock() {
        if let Some(coordinator) = coordinators.get_mut(&popup_id) {
            // Cancel existing timer if any
            if let Some(old_timer) = coordinator.close_timer.take() {
                old_timer.abort();
            }
            coordinator.close_timer = Some(timer);
        }
    }
}

/// Cancel any pending close timer
fn cancel_close_timer(popup_id: &str) {
    if let Ok(mut coordinators) = HOVER_COORDINATORS.lock() {
        if let Some(coordinator) = coordinators.get_mut(popup_id) {
            if let Some(timer) = coordinator.close_timer.take() {
                timer.abort();
            }
        }
    }
}

/// Called by trigger window when mouse enters trigger element
/// Note: This is kept for backward compatibility but hover detection is now handled by Rust monitor
#[command]
pub fn popup_trigger_enter(popup_id: String) {
    eprintln!("[DEBUG] popup_trigger_enter: {} (no-op, handled by Rust monitor)", popup_id);
    // Cancel any pending close timer as a safety measure
    cancel_close_timer(&popup_id);
}

/// Called by trigger window when mouse leaves trigger element
/// Note: This is kept for backward compatibility but hover detection is now handled by Rust monitor
#[command]
pub fn popup_trigger_leave(_app: AppHandle, popup_id: String) {
    eprintln!("[DEBUG] popup_trigger_leave: {} (no-op, handled by Rust monitor)", popup_id);
    // No action needed - Rust monitor handles trigger detection via coordinates
}

/// Called by popup window when mouse enters popup
/// Note: This is kept for backward compatibility but hover detection is now handled by Rust monitor
#[command]
pub fn popup_window_enter(popup_id: String) {
    eprintln!("[DEBUG] popup_window_enter: {} (no-op, handled by Rust monitor)", popup_id);
    // Cancel any pending close timer as a safety measure
    cancel_close_timer(&popup_id);
}

/// Called by popup window when mouse leaves popup
/// Note: This is kept for backward compatibility but hover detection is now handled by Rust monitor
#[command]
pub fn popup_window_leave(_app: AppHandle, popup_id: String) {
    eprintln!("[DEBUG] popup_window_leave: {} (no-op, handled by Rust monitor)", popup_id);
    // No action needed - Rust monitor handles popup detection via window number
}

/// Calculate popup position based on anchor and alignment
fn calculate_popup_position(
    anchor: &PopupAnchor,
    popup_width: f64,
    popup_height: f64,
    align: &PopupAlign,
    offset_y: f64,
    monitor_x: f64,
    monitor_y: f64,
    monitor_width: f64,
    monitor_height: f64,
) -> (f64, f64) {
    // Y: below anchor with offset
    let mut y = anchor.y + anchor.height + offset_y;

    // X: based on alignment
    let mut x = match align {
        PopupAlign::Start => anchor.x,
        PopupAlign::Center => anchor.x + (anchor.width - popup_width) / 2.0,
        PopupAlign::End => anchor.x + anchor.width - popup_width,
    };

    // Clamp to monitor bounds
    x = x.max(monitor_x).min(monitor_x + monitor_width - popup_width);
    y = y.max(monitor_y).min(monitor_y + monitor_height - popup_height);

    (x, y)
}

/// Create a popup window
#[command]
pub async fn create_popup_window(
    app: AppHandle,
    popup_id: String,
    anchor: PopupAnchor,
    width: f64,
    height: f64,
    align: Option<PopupAlign>,
    offset_y: Option<f64>,
    mode: Option<PopupMode>,
) -> Result<PopupInfo, String> {
    let label = format!("popup-{}", popup_id);
    let align = align.unwrap_or_default();
    let offset_y = offset_y.unwrap_or(8.0);
    let mode = mode.unwrap_or_default();

    // Close existing popup with same ID if exists
    if let Some(window) = app.get_webview_window(&label) {
        cleanup_hover_coordinator(&popup_id);
        let _ = window.destroy();
    }

    // Get monitor info
    let (monitor_x, monitor_y, monitor_width, monitor_height) =
        get_monitor_at_point(&app, anchor.x, anchor.y)?;

    // Calculate position
    let (x, y) = calculate_popup_position(
        &anchor,
        width,
        height,
        &align,
        offset_y,
        monitor_x,
        monitor_y,
        monitor_width,
        monitor_height,
    );

    // Convert mode to URL parameter string
    let mode_str = match mode {
        PopupMode::Toggle => "toggle",
        PopupMode::Hover => "hover",
        PopupMode::HoverSticky => "hover-sticky",
    };

    // Build URL with popup and mode parameters
    let url = if cfg!(debug_assertions) {
        format!("http://localhost:1420/?popup={}&mode={}", popup_id, mode_str)
    } else {
        format!("arcana://localhost/?popup={}&mode={}", popup_id, mode_str)
    };

    let webview_url = WebviewUrl::External(url.parse().map_err(|e| format!("Invalid URL: {}", e))?);

    // Create window
    let window = WebviewWindowBuilder::new(&app, &label, webview_url)
        .title(&popup_id)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .visible(false)
        .focused(true)
        .position(x, y)
        .inner_size(width, height)
        .build()
        .map_err(|e| e.to_string())?;

    // Handle focus loss based on popup mode
    let app_for_blur = app.clone();
    let popup_id_for_blur = popup_id.clone();
    let window_for_blur = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            // Close directly for toggle and hover-sticky modes
            // Hover mode is managed by JS events (trigger + popup mouse tracking)
            if mode != PopupMode::Hover {
                let _ = window_for_blur.close();
                let _ = app_for_blur.emit("popup-closed", &popup_id_for_blur);
            }
        }
    });

    // Initialize hover coordinator for hover and toggle modes
    // - Hover: monitor both popup and trigger
    // - Toggle: monitor both popup and trigger (close when cursor leaves both)
    // - HoverSticky: no monitor (uses blur-based closing only)
    match mode {
        PopupMode::Hover | PopupMode::Toggle => {
            // Convert anchor from JS coordinates (top-left origin) to macOS coordinates (bottom-left origin)
            // Formula: y_macos = monitor_y + monitor_height - y_js - height
            let trigger_y_macos = monitor_y + monitor_height - anchor.y - anchor.height;
            let trigger_bounds = TriggerBounds {
                x: anchor.x,
                y: trigger_y_macos,
                width: anchor.width,
                height: anchor.height,
            };
            init_hover_coordinator(&popup_id, Some(trigger_bounds), app.clone());
        }
        PopupMode::HoverSticky => {
            // HoverSticky: no monitor, uses blur-based closing
        }
    }

    // Show window after setup
    window.show().map_err(|e| e.to_string())?;

    Ok(PopupInfo {
        id: popup_id,
        label,
    })
}

/// Close a popup window
#[command]
pub fn close_popup_window(app: AppHandle, popup_id: String) -> Result<(), String> {
    let label = format!("popup-{}", popup_id);

    // Cleanup hover coordinator
    cleanup_hover_coordinator(&popup_id);

    if let Some(window) = app.get_webview_window(&label) {
        window.close().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Ok(())
    }
}

/// Close all popup windows
#[command]
pub fn close_all_popups(app: AppHandle) -> Result<(), String> {
    let windows: Vec<String> = app
        .webview_windows()
        .keys()
        .filter(|k| k.starts_with("popup-"))
        .cloned()
        .collect();

    for label in windows {
        let popup_id = label.strip_prefix("popup-").unwrap_or(&label);
        cleanup_hover_coordinator(popup_id);

        if let Some(window) = app.get_webview_window(&label) {
            let _ = window.close();
        }
    }

    Ok(())
}

/// Get all open popup IDs
#[command]
pub fn get_open_popups(app: AppHandle) -> Vec<String> {
    app.webview_windows()
        .keys()
        .filter(|k| k.starts_with("popup-"))
        .map(|k| k.strip_prefix("popup-").unwrap_or(k).to_string())
        .collect()
}

/// Update popup position (for repositioning when anchor moves)
#[command]
pub fn update_popup_position(
    app: AppHandle,
    popup_id: String,
    anchor: PopupAnchor,
    width: f64,
    height: f64,
    align: Option<PopupAlign>,
    offset_y: Option<f64>,
) -> Result<(), String> {
    let label = format!("popup-{}", popup_id);
    let align = align.unwrap_or_default();
    let offset_y = offset_y.unwrap_or(8.0);

    let window = app
        .get_webview_window(&label)
        .ok_or_else(|| format!("Popup '{}' not found", popup_id))?;

    let (monitor_x, monitor_y, monitor_width, monitor_height) =
        get_monitor_at_point(&app, anchor.x, anchor.y)?;

    let (x, y) = calculate_popup_position(
        &anchor,
        width,
        height,
        &align,
        offset_y,
        monitor_x,
        monitor_y,
        monitor_width,
        monitor_height,
    );

    window
        .set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }))
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ============================================================================
// Trigger Registration & Global Mouse Monitoring
// ============================================================================

/// Start the global trigger monitor if not already running
#[cfg(target_os = "macos")]
fn start_trigger_monitor(app: AppHandle) {
    let mut task_guard = TRIGGER_MONITOR_TASK.lock().unwrap();
    if task_guard.is_some() {
        return; // Already running
    }

    let handle = tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(50)).await;

            // Get current mouse location
            let (mouse_x, mouse_y) = get_mouse_location();

            // Check all registered triggers
            let events_to_emit = {
                let mut triggers = REGISTERED_TRIGGERS.lock().unwrap();
                let mut events: Vec<(String, bool)> = Vec::new();

                for trigger in triggers.values_mut() {
                    let is_over = is_cursor_over_trigger(mouse_x, mouse_y, &trigger.bounds);

                    if is_over && !trigger.is_hovering {
                        // Mouse entered trigger
                        trigger.is_hovering = true;
                        events.push((trigger.id.clone(), true));
                    } else if !is_over && trigger.is_hovering {
                        // Mouse left trigger
                        trigger.is_hovering = false;
                        events.push((trigger.id.clone(), false));
                    }
                }

                events
            };

            // Emit events outside the lock
            for (trigger_id, entered) in events_to_emit {
                if entered {
                    eprintln!("[DEBUG] trigger-hover-enter: {}", trigger_id);
                    let _ = app.emit("trigger-hover-enter", &trigger_id);
                } else {
                    eprintln!("[DEBUG] trigger-hover-leave: {}", trigger_id);
                    let _ = app.emit("trigger-hover-leave", &trigger_id);
                }
            }
        }
    });

    *task_guard = Some(handle);
}

/// Stop the global trigger monitor if no triggers are registered
fn stop_trigger_monitor_if_empty() {
    let triggers = REGISTERED_TRIGGERS.lock().unwrap();
    if triggers.is_empty() {
        let mut task_guard = TRIGGER_MONITOR_TASK.lock().unwrap();
        if let Some(task) = task_guard.take() {
            task.abort();
        }
    }
}

/// Register a hover trigger for global mouse monitoring
/// The trigger bounds should be in screen coordinates (JS top-left origin)
#[command]
pub fn register_hover_trigger(
    app: AppHandle,
    trigger_id: String,
    bounds: TriggerBounds,
    popup_width: f64,
    popup_height: f64,
    popup_align: Option<PopupAlign>,
    popup_offset_y: Option<f64>,
) -> Result<(), String> {
    eprintln!(
        "[DEBUG] register_hover_trigger: {} at ({}, {}, {}, {})",
        trigger_id, bounds.x, bounds.y, bounds.width, bounds.height
    );

    // Convert bounds from JS coordinates (top-left origin) to macOS coordinates (bottom-left origin)
    // We need monitor info to do this conversion
    let (_monitor_x, monitor_y, _monitor_width, monitor_height) =
        get_monitor_at_point(&app, bounds.x, bounds.y)?;

    // Convert: y_macos = monitor_y + monitor_height - y_js - height
    let macos_y = monitor_y + monitor_height - bounds.y - bounds.height;

    let macos_bounds = TriggerBounds {
        x: bounds.x,
        y: macos_y,
        width: bounds.width,
        height: bounds.height,
    };

    let trigger = RegisteredTrigger {
        id: trigger_id.clone(),
        bounds: macos_bounds,
        popup_options: PopupTriggerOptions {
            popup_width,
            popup_height,
            popup_align: popup_align.unwrap_or_default(),
            popup_offset_y: popup_offset_y.unwrap_or(8.0),
        },
        is_hovering: false,
    };

    {
        let mut triggers = REGISTERED_TRIGGERS.lock().unwrap();
        triggers.insert(trigger_id, trigger);
    }

    // Start the global monitor if not running
    #[cfg(target_os = "macos")]
    start_trigger_monitor(app);

    Ok(())
}

/// Unregister a hover trigger
#[command]
pub fn unregister_hover_trigger(trigger_id: String) -> Result<(), String> {
    eprintln!("[DEBUG] unregister_hover_trigger: {}", trigger_id);

    {
        let mut triggers = REGISTERED_TRIGGERS.lock().unwrap();
        triggers.remove(&trigger_id);
    }

    stop_trigger_monitor_if_empty();

    Ok(())
}

/// Update trigger bounds (e.g., after window resize/move)
#[command]
pub fn update_trigger_bounds(
    app: AppHandle,
    trigger_id: String,
    bounds: TriggerBounds,
) -> Result<(), String> {
    // Convert bounds from JS coordinates to macOS coordinates
    let (_monitor_x, monitor_y, _monitor_width, monitor_height) =
        get_monitor_at_point(&app, bounds.x, bounds.y)?;

    let macos_y = monitor_y + monitor_height - bounds.y - bounds.height;

    let macos_bounds = TriggerBounds {
        x: bounds.x,
        y: macos_y,
        width: bounds.width,
        height: bounds.height,
    };

    let mut triggers = REGISTERED_TRIGGERS.lock().unwrap();
    if let Some(trigger) = triggers.get_mut(&trigger_id) {
        trigger.bounds = macos_bounds;
        Ok(())
    } else {
        Err(format!("Trigger '{}' not found", trigger_id))
    }
}

/// Get registered trigger info (for debugging)
#[command]
pub fn get_registered_triggers() -> Vec<String> {
    let triggers = REGISTERED_TRIGGERS.lock().unwrap();
    triggers.keys().cloned().collect()
}
