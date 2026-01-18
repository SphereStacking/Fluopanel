use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarConfig {
    pub position: String,
    pub height: u32,
    pub opacity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetConfig {
    pub enabled: bool,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetsConfig {
    pub workspaces: WidgetConfig,
    pub clock: WidgetConfig,
    pub battery: WidgetConfig,
    pub cpu: WidgetConfig,
    pub memory: WidgetConfig,
    pub network: WidgetConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitHubConfig {
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub bar: BarConfig,
    pub widgets: WidgetsConfig,
    pub theme: ThemeConfig,
    #[serde(default)]
    pub github: Option<GitHubConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            bar: BarConfig {
                position: "top".to_string(),
                height: 32,
                opacity: 0.9,
            },
            widgets: WidgetsConfig {
                workspaces: WidgetConfig {
                    enabled: true,
                    extra: serde_json::json!({}),
                },
                clock: WidgetConfig {
                    enabled: true,
                    extra: serde_json::json!({ "format": "HH:mm" }),
                },
                battery: WidgetConfig {
                    enabled: true,
                    extra: serde_json::json!({}),
                },
                cpu: WidgetConfig {
                    enabled: true,
                    extra: serde_json::json!({}),
                },
                memory: WidgetConfig {
                    enabled: true,
                    extra: serde_json::json!({}),
                },
                network: WidgetConfig {
                    enabled: true,
                    extra: serde_json::json!({}),
                },
            },
            theme: ThemeConfig {
                mode: "system".to_string(),
            },
            github: None,
        }
    }
}

fn get_config_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".config").join("arcana").join("config.json")
}

#[command]
pub fn get_config() -> Result<AppConfig, String> {
    let config_path = get_config_path();

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        let config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))?;
        Ok(config)
    } else {
        Ok(AppConfig::default())
    }
}

#[command]
pub fn save_config(config: AppConfig) -> Result<(), String> {
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
