use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Emitter, Manager, WebviewUrl};

#[cfg(not(target_os = "macos"))]
use tauri::WebviewWindowBuilder;

#[cfg(target_os = "macos")]
use tauri_nspanel::{tauri_panel, ManagerExt, PanelBuilder, PanelLevel};

use super::constants::geometry::*;
use super::helpers::constrain_to_screen;

// Define NSPanel class for popovers (macOS only)
#[cfg(target_os = "macos")]
tauri_panel! {
    panel!(PopoverPanel {
        config: {
            can_become_key_window: true,
            is_floating_panel: true
        }
    })
}

/// Popover alignment relative to anchor element
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PopoverAlign {
    Start,
    #[default]
    Center,
    End,
}

/// Popover anchor position (from trigger element's getBoundingClientRect)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PopoverAnchor {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Open popover response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PopoverInfo {
    pub id: String,
    pub label: String,
    pub closed: bool,
    /// Maximum available height for the popover (from anchor bottom to screen bottom)
    pub max_height: f64,
}

/// Monitor bounds (x, y, width, height) in logical pixels
type MonitorBounds = (f64, f64, f64, f64);

// ============================================================================
// Helper Functions
// ============================================================================

/// Get monitor bounds containing the anchor point
fn get_monitor_at_point(app: &AppHandle, x: f64, y: f64) -> Result<MonitorBounds, String> {
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;

    if monitors.is_empty() {
        return Err("No monitors available".to_string());
    }

    // Find monitor containing the point
    for monitor in &monitors {
        let pos = monitor.position();
        let size = monitor.size();
        let scale = monitor.scale_factor();

        let monitor_x = pos.x as f64 / scale;
        let monitor_y = pos.y as f64 / scale;
        let monitor_width = size.width as f64 / scale;
        let monitor_height = size.height as f64 / scale;

        if x >= monitor_x
            && x < monitor_x + monitor_width
            && y >= monitor_y
            && y < monitor_y + monitor_height
        {
            return Ok((monitor_x, monitor_y, monitor_width, monitor_height));
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

/// Calculate popover position based on anchor, alignment, and monitor bounds
fn calculate_popover_position(
    anchor: &PopoverAnchor,
    popover_width: f64,
    popover_height: f64,
    align: &PopoverAlign,
    offset_y: f64,
    monitor: MonitorBounds,
) -> (f64, f64) {
    let (monitor_x, monitor_y, monitor_width, monitor_height) = monitor;

    // Y: below anchor with offset
    let mut y = anchor.y + anchor.height + offset_y;

    // X: based on alignment
    let mut x = match align {
        PopoverAlign::Start => anchor.x,
        PopoverAlign::Center => anchor.x + (anchor.width - popover_width) / 2.0,
        PopoverAlign::End => anchor.x + anchor.width - popover_width,
    };

    // Clamp to monitor bounds
    x = x.max(monitor_x).min(monitor_x + monitor_width - popover_width);
    y = y
        .max(monitor_y)
        .min(monitor_y + monitor_height - popover_height);

    (x, y)
}

/// Calculate maximum available height from anchor bottom to screen bottom
fn calculate_available_height(
    anchor: &PopoverAnchor,
    offset_y: f64,
    monitor_y: f64,
    monitor_height: f64,
) -> f64 {
    let popover_top = anchor.y + anchor.height + offset_y;
    (monitor_y + monitor_height - popover_top).max(MIN_AVAILABLE_HEIGHT)
}

/// Build popover URL with parameters
fn build_popover_url(popover_id: &str, max_height: u32) -> Result<WebviewUrl, String> {
    let url = if cfg!(debug_assertions) {
        format!(
            "http://localhost:1420/?popover={}&maxHeight={}",
            popover_id, max_height
        )
    } else {
        format!(
            "arcana://localhost/?popover={}&maxHeight={}",
            popover_id, max_height
        )
    };

    let parsed_url = url.parse().map_err(|e| format!("Invalid URL: {}", e))?;
    Ok(WebviewUrl::External(parsed_url))
}

/// Emit popover-closed event with error logging
fn emit_popover_closed(app: &AppHandle, popover_id: &str) {
    if let Err(e) = app.emit("popover-closed", popover_id) {
        eprintln!("[popover] Failed to emit popover-closed event: {}", e);
    }
}

// ============================================================================
// macOS Panel Creation
// ============================================================================

#[cfg(target_os = "macos")]
fn create_macos_panel(
    app: &AppHandle,
    label: &str,
    popover_id: &str,
    webview_url: WebviewUrl,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let app_for_blur = app.clone();
    let popover_id_for_blur = popover_id.to_string();
    let label_for_blur = label.to_string();

    let _panel = PanelBuilder::<_, PopoverPanel>::new(app, label)
        .url(webview_url)
        .level(PanelLevel::Floating)
        .title(popover_id)
        .floating(true)
        .transparent(true)
        .becomes_key_only_if_needed(false)
        .with_window(|w| w.decorations(false).transparent(true))
        .position(tauri::Position::Logical(tauri::LogicalPosition { x, y }))
        .size(tauri::Size::Logical(tauri::LogicalSize { width, height }))
        .build()
        .map_err(|e| e.to_string())?;

    // Setup blur handler - use hide() instead of close() to avoid Obj-C exception
    if let Some(window) = app.get_webview_window(label) {
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::Focused(false) = event {
                // Use order_out (hide) instead of close - safe from event handler
                if let Ok(panel) = app_for_blur.get_webview_panel(&label_for_blur) {
                    panel.hide();
                }
                emit_popover_closed(&app_for_blur, &popover_id_for_blur);
            }
        });
    }

    Ok(())
}

// ============================================================================
// Non-macOS Window Creation
// ============================================================================

#[cfg(not(target_os = "macos"))]
fn create_standard_window(
    app: &AppHandle,
    label: &str,
    popover_id: &str,
    webview_url: WebviewUrl,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let window = WebviewWindowBuilder::new(app, label, webview_url)
        .title(popover_id)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .visible(true)
        .focused(true)
        .position(x, y)
        .inner_size(width, height)
        .build()
        .map_err(|e| e.to_string())?;

    // Close on blur
    let app_for_blur = app.clone();
    let popover_id_for_blur = popover_id.to_string();
    let label_for_blur = label.to_string();

    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            let app = app_for_blur.clone();
            let label = label_for_blur.clone();
            let popover_id = popover_id_for_blur.clone();

            // Schedule close asynchronously to avoid potential issues
            // when closing window from within its own event handler
            tauri::async_runtime::spawn(async move {
                if let Some(win) = app.get_webview_window(&label) {
                    if let Err(e) = win.close() {
                        eprintln!("[popover] Failed to close window: {}", e);
                    }
                }
                emit_popover_closed(&app, &popover_id);
            });
        }
    });

    Ok(())
}

// ============================================================================
// Public Commands
// ============================================================================

/// Open a popover window (toggle mode: if visible, hide it; if hidden, show it; otherwise create new)
#[command]
pub fn open_popover(
    app: AppHandle,
    popover_id: String,
    anchor: PopoverAnchor,
    width: f64,
    height: f64,
    align: Option<PopoverAlign>,
    offset_y: Option<f64>,
) -> Result<PopoverInfo, String> {
    let label = format!("popover-{}", popover_id);
    let align = align.unwrap_or_default();
    let offset_y = offset_y.unwrap_or(DEFAULT_POPOVER_OFFSET_Y);

    // macOS: Check if panel already exists and reuse it
    #[cfg(target_os = "macos")]
    {
        if let Ok(panel) = app.get_webview_panel(&label) {
            if panel.is_visible() {
                // Toggle off: hide it (safe from event handler)
                panel.hide();
                emit_popover_closed(&app, &popover_id);
                return Ok(PopoverInfo {
                    id: popover_id,
                    label,
                    closed: true,
                    max_height: 0.0,
                });
            } else {
                // Toggle on: update position and show
                let monitor = get_monitor_at_point(&app, anchor.x, anchor.y)?;
                let (_monitor_x, monitor_y, monitor_width, monitor_height) = monitor;

                let (constrained_width, constrained_height) =
                    constrain_to_screen(width, height, monitor_width, monitor_height);

                let (x, y) = calculate_popover_position(
                    &anchor,
                    constrained_width,
                    constrained_height,
                    &align,
                    offset_y,
                    monitor,
                );

                if let Some(window) = app.get_webview_window(&label) {
                    if let Err(e) =
                        window.set_position(tauri::Position::Logical(tauri::LogicalPosition {
                            x,
                            y,
                        }))
                    {
                        eprintln!("[popover] Failed to set position: {}", e);
                    }
                    // Don't reset size - keep the auto-sized dimensions from previous show
                }

                let available_max_height =
                    calculate_available_height(&anchor, offset_y, monitor_y, monitor_height);

                panel.show();
                return Ok(PopoverInfo {
                    id: popover_id,
                    label,
                    closed: false,
                    max_height: available_max_height,
                });
            }
        }
    }

    // Non-macOS: Toggle using window destroy
    #[cfg(not(target_os = "macos"))]
    {
        if let Some(window) = app.get_webview_window(&label) {
            if let Err(e) = window.destroy() {
                eprintln!("[popover] Failed to destroy window: {}", e);
            }
            emit_popover_closed(&app, &popover_id);
            return Ok(PopoverInfo {
                id: popover_id,
                label,
                closed: true,
                max_height: 0.0,
            });
        }
    }

    // Get monitor info
    let monitor = get_monitor_at_point(&app, anchor.x, anchor.y)?;
    let (_monitor_x, monitor_y, monitor_width, monitor_height) = monitor;

    // Clamp size to screen bounds
    let (constrained_width, constrained_height) =
        constrain_to_screen(width, height, monitor_width, monitor_height);

    // Calculate position with constrained size
    let (x, y) = calculate_popover_position(
        &anchor,
        constrained_width,
        constrained_height,
        &align,
        offset_y,
        monitor,
    );

    // Calculate max available height
    let available_max_height =
        calculate_available_height(&anchor, offset_y, monitor_y, monitor_height);

    // Build URL with popover parameter and maxHeight
    let webview_url = build_popover_url(&popover_id, available_max_height as u32)?;

    // Create platform-specific window
    #[cfg(target_os = "macos")]
    create_macos_panel(
        &app,
        &label,
        &popover_id,
        webview_url,
        x,
        y,
        constrained_width,
        constrained_height,
    )?;

    #[cfg(not(target_os = "macos"))]
    create_standard_window(
        &app,
        &label,
        &popover_id,
        webview_url,
        x,
        y,
        constrained_width,
        constrained_height,
    )?;

    Ok(PopoverInfo {
        id: popover_id,
        label,
        closed: false,
        max_height: available_max_height,
    })
}

/// Close a popover window (hide only on macOS to avoid Obj-C exceptions)
#[command]
pub fn close_popover(app: AppHandle, popover_id: String) -> Result<(), String> {
    let label = format!("popover-{}", popover_id);

    #[cfg(target_os = "macos")]
    {
        // Just hide the panel - don't destroy to avoid Obj-C exceptions
        if let Ok(panel) = app.get_webview_panel(&label) {
            if panel.is_visible() {
                panel.hide();
                emit_popover_closed(&app, &popover_id);
            }
        }
        return Ok(());
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Some(window) = app.get_webview_window(&label) {
            window.destroy().map_err(|e| e.to_string())?;
            emit_popover_closed(&app, &popover_id);
        }
        return Ok(());
    }

    #[allow(unreachable_code)]
    Ok(())
}

/// Close all popover windows (hide only on macOS to avoid Obj-C exceptions)
#[command]
pub fn close_all_popovers(app: AppHandle) -> Result<(), String> {
    let windows: Vec<String> = app
        .webview_windows()
        .keys()
        .filter(|k| k.starts_with("popover-"))
        .cloned()
        .collect();

    for label in windows {
        let popover_id = label.strip_prefix("popover-").unwrap_or(&label).to_string();

        #[cfg(target_os = "macos")]
        {
            // Just hide the panel - don't destroy to avoid Obj-C exceptions
            // Panels will be reused when reopened
            if let Ok(panel) = app.get_webview_panel(&label) {
                if panel.is_visible() {
                    panel.hide();
                    emit_popover_closed(&app, &popover_id);
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            if let Some(window) = app.get_webview_window(&label) {
                if let Err(e) = window.destroy() {
                    eprintln!("[popover] Failed to destroy window {}: {}", label, e);
                }
                emit_popover_closed(&app, &popover_id);
            }
        }
    }

    Ok(())
}

/// Get all open popover IDs
#[command]
pub fn get_open_popovers(app: AppHandle) -> Vec<String> {
    app.webview_windows()
        .keys()
        .filter(|k| k.starts_with("popover-"))
        .map(|k| k.strip_prefix("popover-").unwrap_or(k).to_string())
        .collect()
}
