use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Emitter, Manager, WebviewUrl};

#[cfg(not(target_os = "macos"))]
use tauri::WebviewWindowBuilder;

#[cfg(target_os = "macos")]
use tauri_nspanel::{tauri_panel, ManagerExt, PanelBuilder, PanelLevel};

// Define NSPanel class for popups (macOS only)
#[cfg(target_os = "macos")]
tauri_panel! {
    panel!(PopupPanel {
        config: {
            can_become_key_window: true,
            is_floating_panel: true
        }
    })
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
    pub closed: bool,
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

/// Open a popup window (toggle mode: if visible, hide it; if hidden, show it; otherwise create new)
#[command]
pub fn open_popup(
    app: AppHandle,
    popup_id: String,
    anchor: PopupAnchor,
    width: f64,
    height: f64,
    align: Option<PopupAlign>,
    offset_y: Option<f64>,
) -> Result<PopupInfo, String> {
    let label = format!("popup-{}", popup_id);
    let align = align.unwrap_or_default();
    let offset_y = offset_y.unwrap_or(8.0);

    // macOS: Check if panel already exists and reuse it
    #[cfg(target_os = "macos")]
    {
        if let Ok(panel) = app.get_webview_panel(&label) {
            if panel.is_visible() {
                // Toggle off: hide it (safe from event handler)
                panel.hide();
                let _ = app.emit("popup-closed", &popup_id);
                return Ok(PopupInfo {
                    id: popup_id,
                    label,
                    closed: true,
                });
            } else {
                // Toggle on: update position and show
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
                if let Some(window) = app.get_webview_window(&label) {
                    let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }));
                }
                panel.show();
                return Ok(PopupInfo {
                    id: popup_id,
                    label,
                    closed: false,
                });
            }
        }
    }

    // Non-macOS: Toggle using window destroy
    #[cfg(not(target_os = "macos"))]
    {
        if let Some(window) = app.get_webview_window(&label) {
            let _ = window.destroy();
            let _ = app.emit("popup-closed", &popup_id);
            return Ok(PopupInfo {
                id: popup_id,
                label,
                closed: true,
            });
        }
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

    // Build URL with popup parameter
    let url = if cfg!(debug_assertions) {
        format!("http://localhost:1420/?popup={}", popup_id)
    } else {
        format!("arcana://localhost/?popup={}", popup_id)
    };

    let webview_url = WebviewUrl::External(url.parse().map_err(|e| format!("Invalid URL: {}", e))?);

    // macOS: Create as NSPanel for proper first-click behavior
    #[cfg(target_os = "macos")]
    {
        let app_for_blur = app.clone();
        let popup_id_for_blur = popup_id.clone();
        let label_for_blur = label.clone();

        let _panel = PanelBuilder::<_, PopupPanel>::new(&app, &label)
            .url(webview_url)
            .level(PanelLevel::Floating)
            .title(&popup_id)
            .floating(true)
            .transparent(true)
            .becomes_key_only_if_needed(true)
            .with_window(|w| w.decorations(false).transparent(true))
            .position(tauri::Position::Logical(tauri::LogicalPosition { x, y }))
            .size(tauri::Size::Logical(tauri::LogicalSize { width, height }))
            .build()
            .map_err(|e| e.to_string())?;

        // Setup blur handler - use hide() instead of close() to avoid Obj-C exception
        if let Some(window) = app.get_webview_window(&label) {
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(false) = event {
                    // Use order_out (hide) instead of close - safe from event handler
                    // This just removes the panel from screen without destroying it
                    if let Ok(panel) = app_for_blur.get_webview_panel(&label_for_blur) {
                        panel.hide();
                    }
                    let _ = app_for_blur.emit("popup-closed", &popup_id_for_blur);
                }
            });
        }
    }

    // Non-macOS: Use standard WebviewWindow
    #[cfg(not(target_os = "macos"))]
    {
        let window = WebviewWindowBuilder::new(&app, &label, webview_url)
            .title(&popup_id)
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
        let popup_id_for_blur = popup_id.clone();
        let label_for_blur = label.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::Focused(false) = event {
                let app = app_for_blur.clone();
                let label = label_for_blur.clone();
                let popup_id = popup_id_for_blur.clone();

                // Schedule close asynchronously to avoid potential issues
                // when closing window from within its own event handler
                tauri::async_runtime::spawn(async move {
                    if let Some(win) = app.get_webview_window(&label) {
                        let _ = win.close();
                    }
                    let _ = app.emit("popup-closed", &popup_id);
                });
            }
        });
    }

    Ok(PopupInfo {
        id: popup_id,
        label,
        closed: false,
    })
}

/// Close a popup window (explicitly destroy it)
#[command]
pub fn close_popup(app: AppHandle, popup_id: String) -> Result<(), String> {
    let label = format!("popup-{}", popup_id);

    #[cfg(target_os = "macos")]
    {
        // First hide the panel safely
        if let Ok(panel) = app.get_webview_panel(&label) {
            panel.hide();
        }
        // Then schedule destroy outside event context
        if let Some(window) = app.get_webview_window(&label) {
            tauri::async_runtime::spawn(async move {
                let _ = window.destroy();
            });
        }
        let _ = app.emit("popup-closed", &popup_id);
        return Ok(());
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Some(window) = app.get_webview_window(&label) {
            window.destroy().map_err(|e| e.to_string())?;
            let _ = app.emit("popup-closed", &popup_id);
        }
        return Ok(());
    }

    #[allow(unreachable_code)]
    Ok(())
}

/// Close all popup windows (explicitly destroy them)
#[command]
pub fn close_all_popups(app: AppHandle) -> Result<(), String> {
    let windows: Vec<String> = app
        .webview_windows()
        .keys()
        .filter(|k| k.starts_with("popup-"))
        .cloned()
        .collect();

    for label in windows {
        let popup_id = label.strip_prefix("popup-").unwrap_or(&label).to_string();

        #[cfg(target_os = "macos")]
        {
            // First hide the panel safely
            if let Ok(panel) = app.get_webview_panel(&label) {
                panel.hide();
            }
            // Then schedule destroy
            if let Some(window) = app.get_webview_window(&label) {
                tauri::async_runtime::spawn(async move {
                    let _ = window.destroy();
                });
            }
            let _ = app.emit("popup-closed", &popup_id);
        }

        #[cfg(not(target_os = "macos"))]
        {
            if let Some(window) = app.get_webview_window(&label) {
                let _ = window.destroy();
                let _ = app.emit("popup-closed", &popup_id);
            }
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
