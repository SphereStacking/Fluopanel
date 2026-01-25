use super::discovery::{WindowManifest, WindowType};
use serde::Deserialize;
use tauri::{command, AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

#[cfg(target_os = "macos")]

/// Window position configuration (bounding box)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowPosition {
    pub monitor: Option<String>,
    pub top: Option<i32>,
    pub bottom: Option<i32>,
    pub left: Option<i32>,
    pub right: Option<i32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Calculated window geometry
struct WindowGeometry {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

/// Validate position configuration
fn validate_position(position: &WindowPosition) -> Result<(), String> {
    // Horizontal: need (left + right) OR (left + width) OR (right + width)
    let has_horizontal = match (position.left, position.right, position.width) {
        (Some(_), Some(_), _) => true,      // left + right
        (Some(_), None, Some(_)) => true,   // left + width
        (None, Some(_), Some(_)) => true,   // right + width
        _ => false,
    };

    if !has_horizontal {
        return Err("Position must specify (left + right) or (left + width) or (right + width)".to_string());
    }

    // Vertical: need (top + bottom) OR (top + height) OR (bottom + height)
    let has_vertical = match (position.top, position.bottom, position.height) {
        (Some(_), Some(_), _) => true,      // top + bottom
        (Some(_), None, Some(_)) => true,   // top + height
        (None, Some(_), Some(_)) => true,   // bottom + height
        _ => false,
    };

    if !has_vertical {
        return Err("Position must specify (top + bottom) or (top + height) or (bottom + height)".to_string());
    }

    Ok(())
}

/// Calculate window geometry from position config and monitor info
fn calculate_geometry(
    position: &WindowPosition,
    monitor_x: i32,
    monitor_y: i32,
    monitor_width: u32,
    monitor_height: u32,
) -> WindowGeometry {
    // Calculate width
    let width = if let (Some(left), Some(right)) = (position.left, position.right) {
        (monitor_width as i32 - left - right).max(1) as u32
    } else {
        position.width.unwrap() // Safe: validated
    };

    // Calculate height
    let height = if let (Some(top), Some(bottom)) = (position.top, position.bottom) {
        (monitor_height as i32 - top - bottom).max(1) as u32
    } else {
        position.height.unwrap() // Safe: validated
    };

    // Calculate x position
    let x = if let Some(left) = position.left {
        monitor_x + left
    } else {
        // right + width case
        monitor_x + monitor_width as i32 - position.right.unwrap() - width as i32
    };

    // Calculate y position
    let y = if let Some(top) = position.top {
        monitor_y + top
    } else {
        // bottom + height case
        monitor_y + monitor_height as i32 - position.bottom.unwrap() - height as i32
    };

    WindowGeometry { x, y, width, height }
}

/// Get monitor info by name or primary
/// Returns (x, y, width, height) in logical pixels for the visible frame
/// On macOS, uses NSScreen.visibleFrame to exclude menu bar and dock
#[cfg(target_os = "macos")]
fn get_monitor_info(_app: &AppHandle, _monitor_name: Option<&str>) -> Result<(i32, i32, u32, u32), String> {
    use objc2::{msg_send, runtime::AnyObject, ClassType};
    use objc2_app_kit::NSScreen;
    use objc2_foundation::NSRect;

    unsafe {
        let screens: *const AnyObject = msg_send![NSScreen::class(), screens];
        if screens.is_null() {
            return Err("No screens available".to_string());
        }

        let main_screen: *const AnyObject = msg_send![screens, firstObject];
        if main_screen.is_null() {
            return Err("No main screen".to_string());
        }

        // visibleFrame excludes menu bar and dock
        let visible: NSRect = msg_send![main_screen, visibleFrame];
        // frame is the full screen
        let frame: NSRect = msg_send![main_screen, frame];

        // macOS uses bottom-left origin, convert to top-left
        // menu_bar_height = frame.height - visible.height - visible.origin.y (dock height)
        let menu_bar_height = frame.size.height - visible.size.height - visible.origin.y;

        Ok((
            visible.origin.x as i32,
            menu_bar_height as i32,
            visible.size.width as u32,
            visible.size.height as u32,
        ))
    }
}

#[cfg(not(target_os = "macos"))]
fn get_monitor_info(app: &AppHandle, monitor_name: Option<&str>) -> Result<(i32, i32, u32, u32), String> {
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;

    if monitors.is_empty() {
        return Err("No monitors available".to_string());
    }

    let monitor = if let Some(name) = monitor_name {
        if name == "primary" {
            app.primary_monitor()
                .map_err(|e| e.to_string())?
                .unwrap_or_else(|| monitors[0].clone())
        } else {
            monitors
                .iter()
                .find(|m| m.name().map(|n| n == name).unwrap_or(false))
                .cloned()
                .unwrap_or_else(|| monitors[0].clone())
        }
    } else {
        app.primary_monitor()
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| monitors[0].clone())
    };

    let size = monitor.size();
    let pos = monitor.position();
    let scale = monitor.scale_factor();

    // Return logical pixels
    Ok((
        (pos.x as f64 / scale) as i32,
        (pos.y as f64 / scale) as i32,
        (size.width as f64 / scale) as u32,
        (size.height as f64 / scale) as u32,
    ))
}

/// Create an inline window (for <Window> component pattern)
#[command]
pub async fn create_inline_window(
    app: AppHandle,
    window_id: String,
    url: String,
    transparent: bool,
    _always_on_top: bool,
    decorations: bool,
    resizable: bool,
    _skip_taskbar: bool,
    position: WindowPosition,
) -> Result<(), String> {
    let label = format!("inline-window-{}", window_id);

    // Check if window already exists
    if app.get_webview_window(&label).is_some() {
        return Err(format!("Inline window '{}' already exists", label));
    }

    // Validate position constraints
    validate_position(&position)?;

    // Get monitor info
    let (monitor_x, monitor_y, monitor_width, monitor_height) =
        get_monitor_info(&app, position.monitor.as_deref())?;

    // Calculate geometry
    let geometry = calculate_geometry(
        &position,
        monitor_x,
        monitor_y,
        monitor_width,
        monitor_height,
    );

    // Parse URL - Tauri handles custom protocols registered via register_uri_scheme_protocol
    let parsed_url: url::Url = url.parse().map_err(|e| format!("Invalid URL: {}", e))?;
    let webview_url = WebviewUrl::External(parsed_url);

    let _window = WebviewWindowBuilder::new(&app, &label, webview_url)
        .title(&window_id)
        .decorations(decorations)
        .transparent(transparent)
        .always_on_top(_always_on_top)
        .skip_taskbar(_skip_taskbar)
        .resizable(resizable)
        .visible(false)
        .focused(false)
        .position(geometry.x as f64, geometry.y as f64)
        .inner_size(geometry.width as f64, geometry.height as f64)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Update window position
#[command]
pub fn update_window_position(
    app: AppHandle,
    label: String,
    position: WindowPosition,
) -> Result<(), String> {
    let window = app
        .get_webview_window(&label)
        .ok_or_else(|| format!("Window '{}' not found", label))?;

    // Validate position constraints
    validate_position(&position)?;

    // Get monitor info
    let (monitor_x, monitor_y, monitor_width, monitor_height) =
        get_monitor_info(&app, position.monitor.as_deref())?;

    // Calculate geometry
    let geometry = calculate_geometry(
        &position,
        monitor_x,
        monitor_y,
        monitor_width,
        monitor_height,
    );

    // Apply position and size
    window
        .set_position(tauri::Position::Logical(tauri::LogicalPosition {
            x: geometry.x as f64,
            y: geometry.y as f64,
        }))
        .map_err(|e| e.to_string())?;

    window
        .set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: geometry.width as f64,
            height: geometry.height as f64,
        }))
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Hide a window by label
#[command]
pub fn hide_window(app: AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        // Ignore cursor events so clicks pass through
        window
            .set_ignore_cursor_events(true)
            .map_err(|e| e.to_string())?;
        window.hide().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err(format!("Window '{}' not found", label))
    }
}

/// Create a new window from manifest
#[command]
pub async fn create_window(
    app: AppHandle,
    window_id: String,
    instance_id: String,
    manifest: WindowManifest,
) -> Result<(), String> {
    let label = format!("window-{}-{}", window_id, instance_id);

    // Check if window already exists
    if app.get_webview_window(&label).is_some() {
        return Err(format!("Window '{}' already exists", label));
    }

    // Determine URL: dev mode or production
    let url = if cfg!(debug_assertions) && manifest.dev_url.is_some() {
        WebviewUrl::External(
            manifest
                .dev_url
                .as_ref()
                .unwrap()
                .parse()
                .map_err(|e| format!("Invalid dev URL: {}", e))?,
        )
    } else {
        // Custom protocol: arcana://window/{window_id}/{entry}
        let url_str = format!("arcana://window/{}/{}", window_id, manifest.entry);
        WebviewUrl::CustomProtocol(url_str.parse().map_err(|e| format!("Invalid URL: {}", e))?)
    };

    // Get window config with defaults based on window type
    let window_config = manifest.window.as_ref();
    let is_bar = matches!(manifest.window_type, WindowType::Bar);

    let transparent = window_config
        .and_then(|c| c.transparent)
        .unwrap_or(true);
    let always_on_top = window_config
        .and_then(|c| c.always_on_top)
        .unwrap_or(is_bar);
    let resizable = window_config
        .and_then(|c| c.resizable)
        .unwrap_or(!is_bar);
    let decorations = window_config
        .and_then(|c| c.decorations)
        .unwrap_or(false);
    let skip_taskbar = window_config
        .and_then(|c| c.skip_taskbar)
        .unwrap_or(true);

    // Build the window
    let builder = WebviewWindowBuilder::new(&app, &label, url)
        .title(&manifest.name)
        .decorations(decorations)
        .transparent(transparent)
        .always_on_top(always_on_top)
        .skip_taskbar(skip_taskbar)
        .resizable(resizable)
        .visible(false) // Hidden initially, shown after positioning
        .focused(false);

    let _window = builder.build().map_err(|e| e.to_string())?;

    // Note: Position will be applied by the frontend via set_window_geometry
    // The frontend handles CSS-like positioning (top, left, right, bottom, etc.)

    Ok(())
}

/// Close a window
#[command]
pub fn close_window(app: AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.close().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err(format!("Window '{}' not found", label))
    }
}

/// Get all active windows
#[command]
pub fn get_windows(app: AppHandle) -> Vec<String> {
    app.webview_windows()
        .keys()
        .filter(|k| k.starts_with("window-") || k.starts_with("inline-window-"))
        .cloned()
        .collect()
}

/// Show a window (after positioning is applied)
#[command]
pub fn show_window(app: AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.show().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err(format!("Window '{}' not found", label))
    }
}
