use serde::Serialize;

use super::helpers::{constrain_to_screen, get_target_window};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub scale_factor: f64,
}

#[tauri::command]
pub fn get_monitors(window: tauri::WebviewWindow) -> Result<Vec<MonitorInfo>, String> {
    let monitors = window.available_monitors().map_err(|e| e.to_string())?;

    let monitor_infos: Vec<MonitorInfo> = monitors
        .into_iter()
        .map(|m| {
            let size = m.size();
            let position = m.position();
            let scale = m.scale_factor();
            // Return logical pixels (divide physical by scale factor)
            MonitorInfo {
                name: m.name().cloned().unwrap_or_else(|| "Unknown".to_string()),
                width: (size.width as f64 / scale) as u32,
                height: (size.height as f64 / scale) as u32,
                x: (position.x as f64 / scale) as i32,
                y: (position.y as f64 / scale) as i32,
                scale_factor: scale,
            }
        })
        .collect();

    Ok(monitor_infos)
}

#[tauri::command]
pub fn set_window_geometry(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    label: Option<String>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let target_window = get_target_window(&app, window, label.as_deref())?;

    target_window
        .set_position(tauri::Position::Logical(tauri::LogicalPosition {
            x: x as f64,
            y: y as f64,
        }))
        .map_err(|e: tauri::Error| e.to_string())?;

    target_window
        .set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: width as f64,
            height: height as f64,
        }))
        .map_err(|e: tauri::Error| e.to_string())?;

    Ok(())
}

/// Set only the position (x, y) of a window
#[tauri::command]
pub fn set_window_position(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    label: Option<String>,
    x: i32,
    y: i32,
) -> Result<(), String> {
    let target_window = get_target_window(&app, window, label.as_deref())?;

    target_window
        .set_position(tauri::Position::Logical(tauri::LogicalPosition {
            x: x as f64,
            y: y as f64,
        }))
        .map_err(|e: tauri::Error| e.to_string())?;

    Ok(())
}

/// Set only the size (width, height) of a window - called by frontend based on content
/// Automatically clamps to screen bounds to prevent content overflow
#[tauri::command]
pub fn set_window_size(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    label: Option<String>,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let target_window = get_target_window(&app, window, label.as_deref())?;

    // Popover windows are already clamped by popover.rs (accurate maxHeight based on anchor position)
    // and useAutoSize (clamps content to maxHeight). Skip additional constraints here.
    let is_popover = target_window.label().starts_with("popover-");

    let (constrained_width, constrained_height) = if is_popover {
        // Popover: use size as-is (already properly constrained)
        (width as f64, height as f64)
    } else if let Ok(Some(monitor)) = target_window.current_monitor() {
        // Regular windows: apply screen bounds constraint
        let scale = monitor.scale_factor();
        let monitor_width = monitor.size().width as f64 / scale;
        let monitor_height = monitor.size().height as f64 / scale;
        constrain_to_screen(width as f64, height as f64, monitor_width, monitor_height)
    } else {
        (width as f64, height as f64)
    };

    target_window
        .set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: constrained_width,
            height: constrained_height,
        }))
        .map_err(|e: tauri::Error| e.to_string())?;

    Ok(())
}
