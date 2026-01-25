use tauri::{AppHandle, Manager, WebviewWindow};

use super::constants::geometry::*;

/// Get target window by label, or use the current window if no label provided
pub fn get_target_window(
    app: &AppHandle,
    current: WebviewWindow,
    label: Option<&str>,
) -> Result<WebviewWindow, String> {
    match label {
        Some(lbl) => app
            .get_webview_window(lbl)
            .ok_or_else(|| format!("Window '{}' not found", lbl)),
        None => Ok(current),
    }
}

/// Constrain dimensions to screen bounds (excluding shadow padding and top margin)
pub fn constrain_to_screen(
    width: f64,
    height: f64,
    monitor_width: f64,
    monitor_height: f64,
) -> (f64, f64) {
    let max_w = monitor_width - SHADOW_PADDING;
    let max_h = monitor_height - SHADOW_PADDING - TOP_MARGIN;
    (width.min(max_w), height.min(max_h))
}
