use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::command;

lazy_static::lazy_static! {
    static ref ICON_CACHE: Mutex<HashMap<String, Option<String>>> = Mutex::new(HashMap::new());
}

const MAX_CACHE_SIZE: usize = 100;
const ICON_SIZE: f64 = 16.0;

#[derive(Debug, Serialize, Clone)]
pub struct AppIcon {
    pub app: String,
    pub icon: Option<String>,
}

#[command]
pub fn get_app_icon(app_name: String) -> Result<AppIcon, String> {
    // Check cache first
    {
        let cache = ICON_CACHE.lock().map_err(|e| e.to_string())?;
        if let Some(cached) = cache.get(&app_name) {
            return Ok(AppIcon {
                app: app_name.clone(),
                icon: cached.clone(),
            });
        }
    }

    let icon_data = fetch_icon_for_app(&app_name);

    // Store in cache
    {
        let mut cache = ICON_CACHE.lock().map_err(|e| e.to_string())?;

        // Simple eviction: clear half the cache if full
        if cache.len() >= MAX_CACHE_SIZE {
            let keys_to_remove: Vec<_> = cache.keys().take(MAX_CACHE_SIZE / 2).cloned().collect();
            for key in keys_to_remove {
                cache.remove(&key);
            }
        }

        cache.insert(app_name.clone(), icon_data.clone());
    }

    Ok(AppIcon {
        app: app_name,
        icon: icon_data,
    })
}

#[command]
pub fn get_app_icons(app_names: Vec<String>) -> Result<Vec<AppIcon>, String> {
    let mut results = Vec::with_capacity(app_names.len());

    for app_name in app_names {
        results.push(get_app_icon(app_name)?);
    }

    Ok(results)
}

#[command]
pub fn clear_icon_cache() -> Result<(), String> {
    let mut cache = ICON_CACHE.lock().map_err(|e| e.to_string())?;
    cache.clear();
    Ok(())
}

#[cfg(target_os = "macos")]
fn fetch_icon_for_app(app_name: &str) -> Option<String> {
    use base64::Engine;
    use objc2::msg_send;
    use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSWorkspace};
    use objc2_foundation::{NSDictionary, NSSize, NSString};

    // Try to find the app bundle path
    let bundle_path = find_app_bundle_path(app_name)?;

    let workspace = NSWorkspace::sharedWorkspace();

    // Create NSString from path
    let path_nsstring = NSString::from_str(&bundle_path);

    // Get the icon for the application
    let icon = workspace.iconForFile(&path_nsstring);

    // Resize icon to desired size
    let size = NSSize::new(ICON_SIZE, ICON_SIZE);
    icon.setSize(size);

    // Convert to PNG data via NSBitmapImageRep
    let tiff_data = icon.TIFFRepresentation()?;

    let bitmap_rep = NSBitmapImageRep::imageRepWithData(&tiff_data)?;

    // Convert to PNG with empty properties dictionary
    let empty_dict: objc2::rc::Retained<NSDictionary<NSString, objc2::runtime::AnyObject>> =
        NSDictionary::new();
    let png_data = unsafe {
        bitmap_rep.representationUsingType_properties(NSBitmapImageFileType::PNG, &empty_dict)
    }?;

    // Get bytes and encode as base64
    let len: usize = unsafe { msg_send![&*png_data, length] };
    let bytes_ptr: *const u8 = unsafe { msg_send![&*png_data, bytes] };
    let slice = unsafe { std::slice::from_raw_parts(bytes_ptr, len) };

    Some(base64::engine::general_purpose::STANDARD.encode(slice))
}

#[cfg(target_os = "macos")]
fn find_app_bundle_path(app_name: &str) -> Option<String> {
    use objc2_app_kit::NSWorkspace;

    // First, try standard application directories
    let search_paths = [
        "/Applications",
        "/System/Applications",
        "/System/Applications/Utilities",
        "/Applications/Utilities",
    ];

    for base_path in &search_paths {
        let full_path = format!("{}/{}.app", base_path, app_name);
        if std::path::Path::new(&full_path).exists() {
            return Some(full_path);
        }
    }

    // Try home directory Applications
    if let Some(home) = dirs::home_dir() {
        let home_apps = home.join("Applications").join(format!("{}.app", app_name));
        if home_apps.exists() {
            return Some(home_apps.to_string_lossy().to_string());
        }
    }

    // Fallback: use NSWorkspace to find running app by name
    let workspace = NSWorkspace::sharedWorkspace();
    let running_apps = workspace.runningApplications();

    for app in running_apps {
        if let Some(name) = app.localizedName() {
            if name.to_string() == app_name {
                if let Some(bundle_url) = app.bundleURL() {
                    if let Some(path) = bundle_url.path() {
                        return Some(path.to_string());
                    }
                }
            }
        }
    }

    // Final fallback: use mdfind to locate the app
    if let Ok(output) = std::process::Command::new("mdfind")
        .args(["kMDItemKind == 'Application'", "-name", app_name])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.ends_with(".app") && line.contains(app_name) {
                    return Some(line.to_string());
                }
            }
        }
    }

    None
}

#[cfg(not(target_os = "macos"))]
fn fetch_icon_for_app(_app_name: &str) -> Option<String> {
    None
}
