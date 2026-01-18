use serde::Serialize;

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
                name: m.name().map(|s| s.clone()).unwrap_or_else(|| "Unknown".to_string()),
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
    use tauri::Manager;

    // If label is provided, look up that window; otherwise use the calling window
    let target_window = if let Some(ref lbl) = label {
        app.get_webview_window(lbl)
            .ok_or_else(|| format!("Window '{}' not found", lbl))?
    } else {
        window
    };

    // Use logical coordinates (Tauri will handle scale factor automatically)
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
    use tauri::Manager;

    let target_window = if let Some(ref lbl) = label {
        app.get_webview_window(lbl)
            .ok_or_else(|| format!("Window '{}' not found", lbl))?
    } else {
        window
    };

    target_window
        .set_position(tauri::Position::Logical(tauri::LogicalPosition {
            x: x as f64,
            y: y as f64,
        }))
        .map_err(|e: tauri::Error| e.to_string())?;

    Ok(())
}

/// Set only the size (width, height) of a window - called by frontend based on content
#[tauri::command]
pub fn set_window_size(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    label: Option<String>,
    width: u32,
    height: u32,
) -> Result<(), String> {
    use tauri::Manager;

    let target_window = if let Some(ref lbl) = label {
        app.get_webview_window(lbl)
            .ok_or_else(|| format!("Window '{}' not found", lbl))?
    } else {
        window
    };

    target_window
        .set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: width as f64,
            height: height as f64,
        }))
        .map_err(|e: tauri::Error| e.to_string())?;

    Ok(())
}
