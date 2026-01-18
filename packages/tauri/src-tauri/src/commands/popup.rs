use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

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

/// Create a popup window (Toggle mode only - closes on blur)
#[command]
pub async fn create_popup_window(
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

    // Close existing popup with same ID if exists
    if let Some(window) = app.get_webview_window(&label) {
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

    // Build URL with popup parameter
    let url = if cfg!(debug_assertions) {
        format!("http://localhost:1420/?popup={}", popup_id)
    } else {
        format!("arcana://localhost/?popup={}", popup_id)
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

    // Close on blur (Toggle mode behavior)
    let app_for_blur = app.clone();
    let popup_id_for_blur = popup_id.clone();
    let window_for_blur = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            let _ = window_for_blur.close();
            let _ = app_for_blur.emit("popup-closed", &popup_id_for_blur);
        }
    });

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

    if let Some(window) = app.get_webview_window(&label) {
        window.close().map_err(|e| e.to_string())?;
    }

    Ok(())
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
