use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::command;

// ============================================
// Global Config (arcana.json)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub mode: String,
    #[serde(rename = "accentColor", skip_serializing_if = "Option::is_none")]
    pub accent_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSettings {
    pub hot_reload: bool,
    pub dev_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubSecret {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecretsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github: Option<GitHubSecret>,
}

/// UI configuration for loading user-built frontends
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UiConfig {
    /// Custom path to UI dist folder (supports ~ expansion)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dist_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcanaConfig {
    pub version: u32,
    pub theme: ThemeConfig,
    pub settings: GlobalSettings,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets: Option<SecretsConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui: Option<UiConfig>,
}

impl Default for ArcanaConfig {
    fn default() -> Self {
        ArcanaConfig {
            version: 2,
            theme: ThemeConfig {
                mode: "system".to_string(),
                accent_color: Some("#007AFF".to_string()),
            },
            settings: GlobalSettings {
                hot_reload: true,
                dev_mode: false,
            },
            secrets: None,
            ui: None,
        }
    }
}

// ============================================
// Path Helpers
// ============================================

pub fn get_config_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".config").join("arcana")
}

fn get_config_path() -> PathBuf {
    get_config_dir().join("arcana.json")
}

/// Get the UI dist path based on config priority
/// Returns: Some(path) if found, None if no UI available
///
/// Priority:
/// 1. arcana.json ui.distPath (if set and exists)
/// 2. ~/.config/arcana/dist/ (if exists)
/// 3. None
pub fn get_ui_dist_path() -> Option<PathBuf> {
    // 1. Check config for custom distPath
    if let Ok(config) = get_config_sync() {
        if let Some(ui) = &config.ui {
            if let Some(dist_path) = &ui.dist_path {
                // Expand ~ to home directory
                let expanded = if dist_path.starts_with("~/") {
                    if let Some(home) = dirs::home_dir() {
                        home.join(&dist_path[2..])
                    } else {
                        PathBuf::from(dist_path)
                    }
                } else {
                    PathBuf::from(dist_path)
                };

                if expanded.exists() && expanded.join("index.html").exists() {
                    return Some(expanded);
                }
            }
        }
    }

    // 2. Check default location ~/.config/arcana/dist/
    let default_path = get_config_dir().join("dist");
    if default_path.exists() && default_path.join("index.html").exists() {
        return Some(default_path);
    }

    // 3. No UI found
    None
}

/// Synchronous config reader for protocol handler
fn get_config_sync() -> Result<ArcanaConfig, String> {
    let config_path = get_config_path();
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
    } else {
        Ok(ArcanaConfig::default())
    }
}

// ============================================
// Config Commands
// ============================================

#[command]
pub fn get_config() -> Result<ArcanaConfig, String> {
    let config_path = get_config_path();

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        let config: ArcanaConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))?;
        Ok(config)
    } else {
        Ok(ArcanaConfig::default())
    }
}

#[command]
pub fn save_config(config: ArcanaConfig) -> Result<(), String> {
    let config_path = get_config_path();

    // Create parent directories if they don't exist
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(())
}
