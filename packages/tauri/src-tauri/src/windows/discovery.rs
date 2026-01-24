use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowPosition {
    pub monitor: Option<String>,
    pub top: Option<serde_json::Value>,
    pub left: Option<serde_json::Value>,
    pub right: Option<serde_json::Value>,
    pub bottom: Option<serde_json::Value>,
    pub width: Option<serde_json::Value>,
    pub height: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowConfig {
    pub transparent: Option<bool>,
    pub always_on_top: Option<bool>,
    pub resizable: Option<bool>,
    pub decorations: Option<bool>,
    pub skip_taskbar: Option<bool>,
    pub click_through: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(rename = "type")]
    pub window_type: WindowType,
    pub position: WindowPosition,
    pub window: Option<WindowConfig>,
    pub entry: String,
    pub dev_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowType {
    Bar,
    Floating,
}

impl Default for WindowManifest {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            version: "0.1.0".to_string(),
            window_type: WindowType::Floating,
            position: WindowPosition {
                monitor: None,
                top: None,
                left: None,
                right: None,
                bottom: None,
                width: Some(serde_json::json!(300)),
                height: Some(serde_json::json!(200)),
            },
            window: None,
            entry: "index.html".to_string(),
            dev_url: None,
        }
    }
}

/// Get the windows directory path
pub fn get_windows_dir() -> Result<PathBuf, String> {
    dirs::config_dir()
        .map(|d| d.join("arcana/windows"))
        .ok_or_else(|| "Could not determine config directory".to_string())
}

/// Discover all windows in the windows directory
#[command]
pub fn discover_windows() -> Result<Vec<WindowManifest>, String> {
    let windows_dir = get_windows_dir()?;

    if !windows_dir.exists() {
        // Create windows directory if it doesn't exist
        fs::create_dir_all(&windows_dir).map_err(|e| e.to_string())?;
        return Ok(Vec::new());
    }

    let mut windows = Vec::new();

    let entries = fs::read_dir(&windows_dir).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("window.json");
        if !manifest_path.exists() {
            continue;
        }

        match fs::read_to_string(&manifest_path) {
            Ok(content) => match serde_json::from_str::<WindowManifest>(&content) {
                Ok(manifest) => {
                    windows.push(manifest);
                }
                Err(e) => {
                    eprintln!(
                        "[Window] Failed to parse manifest at {:?}: {}",
                        manifest_path, e
                    );
                }
            },
            Err(e) => {
                eprintln!(
                    "[Window] Failed to read manifest at {:?}: {}",
                    manifest_path, e
                );
            }
        }
    }

    Ok(windows)
}

/// Get a specific window manifest by ID
#[command]
pub fn get_window_manifest(window_id: String) -> Result<WindowManifest, String> {
    let windows_dir = get_windows_dir()?;
    let window_dir = windows_dir.join(&window_id);
    let manifest_path = window_dir.join("window.json");

    if !manifest_path.exists() {
        return Err(format!("Window '{}' not found", window_id));
    }

    let content = fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
    let manifest: WindowManifest = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    Ok(manifest)
}
