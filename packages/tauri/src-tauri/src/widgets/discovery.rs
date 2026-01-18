use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetPosition {
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
pub struct WidgetWindowConfig {
    pub transparent: Option<bool>,
    pub always_on_top: Option<bool>,
    pub resizable: Option<bool>,
    pub decorations: Option<bool>,
    pub skip_taskbar: Option<bool>,
    pub click_through: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(rename = "type")]
    pub widget_type: WidgetType,
    pub position: WidgetPosition,
    pub window: Option<WidgetWindowConfig>,
    pub entry: String,
    pub dev_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WidgetType {
    Bar,
    Floating,
}

impl Default for WidgetManifest {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            version: "0.1.0".to_string(),
            widget_type: WidgetType::Floating,
            position: WidgetPosition {
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

/// Get the widgets directory path
pub fn get_widgets_dir() -> Result<PathBuf, String> {
    dirs::config_dir()
        .map(|d| d.join("arcana/widgets"))
        .ok_or_else(|| "Could not determine config directory".to_string())
}

/// Discover all widgets in the widgets directory
#[command]
pub fn discover_widgets() -> Result<Vec<WidgetManifest>, String> {
    let widgets_dir = get_widgets_dir()?;

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

        let manifest_path = path.join("widget.json");
        if !manifest_path.exists() {
            continue;
        }

        match fs::read_to_string(&manifest_path) {
            Ok(content) => match serde_json::from_str::<WidgetManifest>(&content) {
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
pub fn get_widget_manifest(widget_id: String) -> Result<WidgetManifest, String> {
    let widgets_dir = get_widgets_dir()?;
    let widget_dir = widgets_dir.join(&widget_id);
    let manifest_path = widget_dir.join("widget.json");

    if !manifest_path.exists() {
        return Err(format!("Widget '{}' not found", widget_id));
    }

    let content = fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
    let manifest: WidgetManifest = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    Ok(manifest)
}
