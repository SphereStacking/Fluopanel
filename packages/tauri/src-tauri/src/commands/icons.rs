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
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSAutoreleasePool;
    use objc::{class, msg_send, sel, sel_impl};
    use std::ffi::CStr;

    unsafe {
        let _pool = NSAutoreleasePool::new(nil);

        // Get NSWorkspace shared instance
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];

        // Try to find the app bundle path
        let bundle_path = find_app_bundle_path(app_name)?;

        // Create NSString from path
        let path_nsstring = create_nsstring(&bundle_path);

        // Get the icon for the application
        let icon: id = msg_send![workspace, iconForFile: path_nsstring];

        if icon == nil {
            return None;
        }

        // Resize icon to desired size
        let size = cocoa::foundation::NSSize::new(ICON_SIZE, ICON_SIZE);
        let _: () = msg_send![icon, setSize: size];

        // Convert to PNG data via NSBitmapImageRep
        let tiff_data: id = msg_send![icon, TIFFRepresentation];
        if tiff_data == nil {
            return None;
        }

        let bitmap_rep: id = msg_send![class!(NSBitmapImageRep), imageRepWithData: tiff_data];
        if bitmap_rep == nil {
            return None;
        }

        // Convert to PNG (NSBitmapImageFileTypePNG = 4)
        let png_data: id = msg_send![
            bitmap_rep,
            representationUsingType: 4_u64
            properties: nil
        ];
        if png_data == nil {
            return None;
        }

        // Get bytes and encode as base64
        let length: usize = msg_send![png_data, length];
        let bytes: *const u8 = msg_send![png_data, bytes];
        let slice = std::slice::from_raw_parts(bytes, length);

        Some(base64::engine::general_purpose::STANDARD.encode(slice))
    }
}

#[cfg(target_os = "macos")]
fn find_app_bundle_path(app_name: &str) -> Option<String> {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};
    use std::ffi::CStr;

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
    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let running_apps: id = msg_send![workspace, runningApplications];
        let count: usize = msg_send![running_apps, count];

        for i in 0..count {
            let app: id = msg_send![running_apps, objectAtIndex: i];
            let localized_name: id = msg_send![app, localizedName];

            if localized_name != nil {
                let name_str: *const std::os::raw::c_char = msg_send![localized_name, UTF8String];
                if !name_str.is_null() {
                    let name = CStr::from_ptr(name_str).to_string_lossy();

                    if name == app_name {
                        let bundle_url: id = msg_send![app, bundleURL];
                        if bundle_url != nil {
                            let path: id = msg_send![bundle_url, path];
                            if path != nil {
                                let path_str: *const std::os::raw::c_char =
                                    msg_send![path, UTF8String];
                                if !path_str.is_null() {
                                    return Some(
                                        CStr::from_ptr(path_str).to_string_lossy().into_owned(),
                                    );
                                }
                            }
                        }
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

#[cfg(target_os = "macos")]
unsafe fn create_nsstring(s: &str) -> cocoa::base::id {
    use cocoa::base::id;
    use objc::{class, msg_send, sel, sel_impl};

    let cls = class!(NSString);
    let bytes = s.as_ptr() as *const std::os::raw::c_char;
    let len = s.len();
    let nsstring: id = msg_send![cls, alloc];
    msg_send![nsstring, initWithBytes:bytes length:len encoding:4_u64] // NSUTF8StringEncoding = 4
}

#[cfg(not(target_os = "macos"))]
fn fetch_icon_for_app(_app_name: &str) -> Option<String> {
    None
}
