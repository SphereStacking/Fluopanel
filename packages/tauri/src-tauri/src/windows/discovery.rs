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

/// Widget manifest (also known as WindowManifest for backwards compatibility)
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

/// Get the widgets directory path (prefers "widgets", falls back to "windows" for backwards compatibility)
/// Uses XDG-style config directory (~/.config/arcana/) for cross-platform consistency
pub fn get_windows_dir() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|home| {
            // Use XDG-style config directory for consistency
            let config_base = home.join(".config/arcana");
            let widgets_dir = config_base.join("widgets");
            let windows_dir = config_base.join("windows");
            // Prefer widgets directory, fall back to windows for backwards compatibility
            if widgets_dir.exists() {
                widgets_dir
            } else if windows_dir.exists() {
                windows_dir
            } else {
                // Default to widgets for new installations
                widgets_dir
            }
        })
        .ok_or_else(|| "Could not determine home directory".to_string())
}

/// Get manifest path for a widget directory (prefers "widget.json", falls back to "window.json")
fn get_manifest_path(widget_dir: &PathBuf) -> Option<PathBuf> {
    let widget_json = widget_dir.join("widget.json");
    let window_json = widget_dir.join("window.json");

    if widget_json.exists() {
        Some(widget_json)
    } else if window_json.exists() {
        Some(window_json)
    } else {
        None
    }
}

/// Discover all widgets in the widgets directory
#[command]
pub fn discover_windows() -> Result<Vec<WindowManifest>, String> {
    let widgets_dir = get_windows_dir()?;

    if !widgets_dir.exists() {
        // Create widgets directory if it doesn't exist
        fs::create_dir_all(&widgets_dir).map_err(|e| e.to_string())?;
        return Ok(Vec::new());
    }

    let mut widgets = Vec::new();

    let entries = fs::read_dir(&widgets_dir).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let manifest_path = match get_manifest_path(&path) {
            Some(p) => p,
            None => continue,
        };

        match fs::read_to_string(&manifest_path) {
            Ok(content) => match serde_json::from_str::<WindowManifest>(&content) {
                Ok(manifest) => {
                    widgets.push(manifest);
                }
                Err(e) => {
                    eprintln!(
                        "[Widget] Failed to parse manifest at {:?}: {}",
                        manifest_path, e
                    );
                }
            },
            Err(e) => {
                eprintln!(
                    "[Widget] Failed to read manifest at {:?}: {}",
                    manifest_path, e
                );
            }
        }
    }

    Ok(widgets)
}

/// Get a specific widget manifest by ID
#[command]
pub fn get_window_manifest(window_id: String) -> Result<WindowManifest, String> {
    let widgets_dir = get_windows_dir()?;
    let widget_dir = widgets_dir.join(&window_id);

    let manifest_path = get_manifest_path(&widget_dir)
        .ok_or_else(|| format!("Widget '{}' not found (no widget.json or window.json)", window_id))?;

    let content = fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
    let manifest: WindowManifest = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    Ok(manifest)
}
