use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Emitter, Manager, WebviewUrl};

#[cfg(not(target_os = "macos"))]
use tauri::WebviewWindowBuilder;

#[cfg(target_os = "macos")]
use tauri_nspanel::{tauri_panel, ManagerExt, PanelBuilder, PanelLevel};

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

/// Shadow padding constant (p-20 = 80px each side = 160px total)
const SHADOW_PADDING: f64 = 160.0;
/// Top margin for menu bar area
const TOP_MARGIN: f64 = 80.0;

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

/// Calculate popover position based on anchor and alignment
fn calculate_popover_position(
    anchor: &PopoverAnchor,
    popover_width: f64,
    popover_height: f64,
    align: &PopoverAlign,
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
        PopoverAlign::Start => anchor.x,
        PopoverAlign::Center => anchor.x + (anchor.width - popover_width) / 2.0,
        PopoverAlign::End => anchor.x + anchor.width - popover_width,
    };

    // Clamp to monitor bounds
    x = x.max(monitor_x).min(monitor_x + monitor_width - popover_width);
    y = y.max(monitor_y).min(monitor_y + monitor_height - popover_height);

    (x, y)
}

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
    let offset_y = offset_y.unwrap_or(8.0);

    // macOS: Check if panel already exists and reuse it
    #[cfg(target_os = "macos")]
    {
        if let Ok(panel) = app.get_webview_panel(&label) {
            if panel.is_visible() {
                // Toggle off: hide it (safe from event handler)
                panel.hide();
                let _ = app.emit("popover-closed", &popover_id);
                return Ok(PopoverInfo {
                    id: popover_id,
                    label,
                    closed: true,
                    max_height: 0.0,
                });
            } else {
                // Toggle on: update position, size, and show
                let (monitor_x, monitor_y, monitor_width, monitor_height) =
                    get_monitor_at_point(&app, anchor.x, anchor.y)?;

                // Clamp size to screen bounds
                let max_width = monitor_width - SHADOW_PADDING;
                let max_height = monitor_height - SHADOW_PADDING - TOP_MARGIN;
                let constrained_width = width.min(max_width);
                let constrained_height = height.min(max_height);

                let (x, y) = calculate_popover_position(
                    &anchor,
                    constrained_width,
                    constrained_height,
                    &align,
                    offset_y,
                    monitor_x,
                    monitor_y,
                    monitor_width,
                    monitor_height,
                );
                if let Some(window) = app.get_webview_window(&label) {
                    let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }));
                    // Don't reset size - keep the auto-sized dimensions from previous show
                }
                // Calculate max available height from anchor bottom to screen bottom
                let popover_top = anchor.y + anchor.height + offset_y;
                let available_max_height = (monitor_y + monitor_height - popover_top).max(100.0);

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
            let _ = window.destroy();
            let _ = app.emit("popover-closed", &popover_id);
            return Ok(PopoverInfo {
                id: popover_id,
                label,
                closed: true,
                max_height: 0.0,
            });
        }
    }

    // Get monitor info
    let (monitor_x, monitor_y, monitor_width, monitor_height) =
        get_monitor_at_point(&app, anchor.x, anchor.y)?;

    // Clamp size to screen bounds
    let max_width = monitor_width - SHADOW_PADDING;
    let max_height = monitor_height - SHADOW_PADDING - TOP_MARGIN;
    let constrained_width = width.min(max_width);
    let constrained_height = height.min(max_height);

    // Calculate position with constrained size
    let (x, y) = calculate_popover_position(
        &anchor,
        constrained_width,
        constrained_height,
        &align,
        offset_y,
        monitor_x,
        monitor_y,
        monitor_width,
        monitor_height,
    );

    // Calculate max available height from anchor bottom to screen bottom
    let popover_top = anchor.y + anchor.height + offset_y;
    let available_max_height = (monitor_y + monitor_height - popover_top).max(100.0);

    // Build URL with popover parameter and maxHeight
    let url = if cfg!(debug_assertions) {
        format!("http://localhost:1420/?popover={}&maxHeight={}", popover_id, available_max_height as u32)
    } else {
        format!("arcana://localhost/?popover={}&maxHeight={}", popover_id, available_max_height as u32)
    };

    let webview_url = WebviewUrl::External(url.parse().map_err(|e| format!("Invalid URL: {}", e))?);

    // macOS: Create as NSPanel for proper first-click behavior
    #[cfg(target_os = "macos")]
    {
        let app_for_blur = app.clone();
        let popover_id_for_blur = popover_id.clone();
        let label_for_blur = label.clone();

        let _panel = PanelBuilder::<_, PopoverPanel>::new(&app, &label)
            .url(webview_url)
            .level(PanelLevel::Floating)
            .title(&popover_id)
            .floating(true)
            .transparent(true)
            .becomes_key_only_if_needed(false)
            .with_window(|w| w.decorations(false).transparent(true))
            .position(tauri::Position::Logical(tauri::LogicalPosition { x, y }))
            .size(tauri::Size::Logical(tauri::LogicalSize {
                width: constrained_width,
                height: constrained_height,
            }))
            .build()
            .map_err(|e| e.to_string())?;

        // Setup blur handler - use hide() instead of close() to avoid Obj-C exception
        if let Some(window) = app.get_webview_window(&label) {
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(false) = event {
                    // Use order_out (hide) instead of close - safe from event handler
                    if let Ok(panel) = app_for_blur.get_webview_panel(&label_for_blur) {
                        panel.hide();
                    }
                    let _ = app_for_blur.emit("popover-closed", &popover_id_for_blur);
                }
            });
        }
    }

    // Non-macOS: Use standard WebviewWindow
    #[cfg(not(target_os = "macos"))]
    {
        let window = WebviewWindowBuilder::new(&app, &label, webview_url)
            .title(&popover_id)
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .visible(true)
            .focused(true)
            .position(x, y)
            .inner_size(constrained_width, constrained_height)
            .build()
            .map_err(|e| e.to_string())?;

        // Close on blur
        let app_for_blur = app.clone();
        let popover_id_for_blur = popover_id.clone();
        let label_for_blur = label.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::Focused(false) = event {
                let app = app_for_blur.clone();
                let label = label_for_blur.clone();
                let popover_id = popover_id_for_blur.clone();

                // Schedule close asynchronously to avoid potential issues
                // when closing window from within its own event handler
                tauri::async_runtime::spawn(async move {
                    if let Some(win) = app.get_webview_window(&label) {
                        let _ = win.close();
                    }
                    let _ = app.emit("popover-closed", &popover_id);
                });
            }
        });
    }

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
                let _ = app.emit("popover-closed", &popover_id);
            }
        }
        return Ok(());
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Some(window) = app.get_webview_window(&label) {
            window.destroy().map_err(|e| e.to_string())?;
            let _ = app.emit("popover-closed", &popover_id);
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
                    let _ = app.emit("popover-closed", &popover_id);
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            if let Some(window) = app.get_webview_window(&label) {
                let _ = window.destroy();
                let _ = app.emit("popover-closed", &popover_id);
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
