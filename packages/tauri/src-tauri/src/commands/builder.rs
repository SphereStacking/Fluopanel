use crate::windows::get_windows_dir;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{command, AppHandle, Manager};

/// Check if a widget needs to be built (contains .vue, .jsx, or .tsx files)
fn needs_build(widget_dir: &Path) -> bool {
    if !widget_dir.is_dir() {
        return false;
    }

    // Check for Vue/React source files
    let entries = match fs::read_dir(widget_dir) {
        Ok(entries) => entries,
        Err(_) => return false,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if ext == "vue" || ext == "jsx" || ext == "tsx" {
                // Check if build output exists and is newer
                let build_output = widget_dir.join(".arcana").join("index.html");
                if build_output.exists() {
                    // Compare modification times
                    let source_mtime = fs::metadata(&path)
                        .and_then(|m| m.modified())
                        .ok();
                    let build_mtime = fs::metadata(&build_output)
                        .and_then(|m| m.modified())
                        .ok();

                    if let (Some(src), Some(build)) = (source_mtime, build_mtime) {
                        if src <= build {
                            continue; // Build is up to date
                        }
                    }
                }
                return true;
            }
        }
    }

    false
}

/// Get the path to the builder script
fn get_builder_script(app: &AppHandle) -> Result<PathBuf, String> {
    // Try resource directory first (production)
    if let Ok(resource_dir) = app.path().resource_dir() {
        let builder_path: PathBuf = resource_dir.join("builder").join("build.mjs");
        if builder_path.exists() {
            return Ok(builder_path);
        }
    }

    // Fallback: development mode
    let dev_path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("builder")
        .join("build.mjs");

    if dev_path.exists() {
        Ok(dev_path)
    } else {
        Err("Builder script not found".to_string())
    }
}

/// Build a widget from Vue/React source files
#[command]
pub async fn build_widget(app: AppHandle, widget_id: String) -> Result<(), String> {
    let widget_dir = get_windows_dir()?.join(&widget_id);

    if !widget_dir.exists() {
        return Err(format!("Widget '{}' not found", widget_id));
    }

    if !needs_build(&widget_dir) {
        return Ok(()); // No build needed
    }

    let builder_path = get_builder_script(&app)?;

    // Run Node.js builder
    let output = Command::new("node")
        .arg(&builder_path)
        .arg("--widget")
        .arg(&widget_dir)
        .output()
        .map_err(|e| format!("Failed to run builder: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "Build failed:\nstdout: {}\nstderr: {}",
            stdout, stderr
        ));
    }

    Ok(())
}

/// Build all widgets that need building
#[command]
pub async fn build_all_widgets(app: AppHandle) -> Result<Vec<String>, String> {
    let widgets_dir = get_windows_dir()?;
    let mut built = Vec::new();

    if !widgets_dir.exists() {
        return Ok(built);
    }

    let entries = fs::read_dir(&widgets_dir).map_err(|e| e.to_string())?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if needs_build(&path) {
                if let Some(widget_id) = path.file_name().and_then(|n| n.to_str()) {
                    match build_widget(app.clone(), widget_id.to_string()).await {
                        Ok(()) => built.push(widget_id.to_string()),
                        Err(e) => eprintln!("[Builder] Failed to build {}: {}", widget_id, e),
                    }
                }
            }
        }
    }

    Ok(built)
}

/// Check if a widget has buildable source files
#[command]
pub fn widget_needs_build(widget_id: String) -> Result<bool, String> {
    let widget_dir = get_windows_dir()?.join(&widget_id);
    Ok(needs_build(&widget_dir))
}
