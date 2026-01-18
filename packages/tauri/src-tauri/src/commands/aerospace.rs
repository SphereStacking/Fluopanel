use serde::{Deserialize, Serialize};
use std::process::Command;
use tauri::command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    pub id: i64,
    pub app: String,
    pub title: String,
    pub focused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub display_name: Option<String>,
    pub focused: bool,
    pub visible: bool,
    pub windows: Vec<Window>,
    pub monitor: i32,
}

#[derive(Debug, Deserialize)]
struct AerospaceWorkspace {
    workspace: String,
    monitor: Option<i32>,
    #[serde(rename = "monitor-id")]
    monitor_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct AerospaceWindow {
    #[serde(rename = "window-id")]
    window_id: i64,
    #[serde(rename = "app-name")]
    app_name: String,
    #[serde(rename = "window-title")]
    window_title: Option<String>,
}

const AEROSPACE_PATH: &str = "/opt/homebrew/bin/aerospace";

/// Sync version for internal use (CLI, IPC)
fn run_aerospace_command(args: &[&str]) -> Result<String, String> {
    let output = Command::new(AEROSPACE_PATH)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute aerospace: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Async version that runs in a blocking thread pool to avoid UI freeze
async fn run_aerospace_command_async(args: &[&str]) -> Result<String, String> {
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    tauri::async_runtime::spawn_blocking(move || {
        let output = Command::new(AEROSPACE_PATH)
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to execute aerospace: {}", e))?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Internal sync function for CLI use
pub fn aerospace_get_workspaces_sync() -> Result<Vec<Workspace>, String> {
    // Get all workspaces
    let workspaces_output = run_aerospace_command(&["list-workspaces", "--all", "--json"])?;
    let aerospace_workspaces: Vec<AerospaceWorkspace> = serde_json::from_str(&workspaces_output)
        .map_err(|e| format!("Failed to parse workspaces JSON: {}", e))?;

    // Get focused workspace
    let focused_output = run_aerospace_command(&["list-workspaces", "--focused"])?;
    let focused_id = focused_output.trim();

    // Get visible workspaces by querying each monitor
    // aerospace requires --monitor with --visible
    let monitors_output = run_aerospace_command(&["list-monitors"]);
    let mut visible_ids: Vec<String> = Vec::new();

    if let Ok(monitors) = monitors_output {
        for monitor in monitors.lines() {
            let monitor_id = monitor.trim();
            if !monitor_id.is_empty() {
                if let Ok(visible) = run_aerospace_command(&["list-workspaces", "--monitor", monitor_id, "--visible"]) {
                    for ws in visible.lines() {
                        let ws_trimmed = ws.trim();
                        if !ws_trimmed.is_empty() {
                            visible_ids.push(ws_trimmed.to_string());
                        }
                    }
                }
            }
        }
    }

    // Build workspace list with windows per workspace
    let workspaces: Vec<Workspace> = aerospace_workspaces
        .into_iter()
        .map(|ws| {
            // Get windows for this specific workspace
            let ws_windows: Vec<Window> = run_aerospace_command(&["list-windows", "--workspace", &ws.workspace, "--json"])
                .ok()
                .and_then(|output| serde_json::from_str::<Vec<AerospaceWindow>>(&output).ok())
                .unwrap_or_default()
                .into_iter()
                .map(|w| Window {
                    id: w.window_id,
                    app: w.app_name,
                    title: w.window_title.unwrap_or_default(),
                    focused: false,
                })
                .collect();

            Workspace {
                id: ws.workspace.clone(),
                display_name: None,
                focused: ws.workspace == focused_id,
                visible: visible_ids.iter().any(|v| v == &ws.workspace),
                windows: ws_windows,
                monitor: ws.monitor_id.or(ws.monitor).unwrap_or(0),
            }
        })
        .collect();

    Ok(workspaces)
}

/// Tauri command wrapper - async to avoid UI freeze
#[command]
pub async fn aerospace_get_workspaces() -> Result<Vec<Workspace>, String> {
    tauri::async_runtime::spawn_blocking(aerospace_get_workspaces_sync)
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

#[command]
pub async fn aerospace_get_focused_workspace() -> Result<Option<Workspace>, String> {
    let workspaces = aerospace_get_workspaces().await?;
    Ok(workspaces.into_iter().find(|w| w.focused))
}

#[command]
pub async fn aerospace_focus_workspace(id: String) -> Result<(), String> {
    run_aerospace_command_async(&["workspace", &id]).await?;
    Ok(())
}

/// Get a single workspace by ID (optimized for focus change events)
pub fn get_workspace_by_id(id: &str, is_focused: bool) -> Option<Workspace> {
    // Get windows for this workspace
    let windows: Vec<Window> = run_aerospace_command(&["list-windows", "--workspace", id, "--json"])
        .ok()
        .and_then(|output| serde_json::from_str::<Vec<AerospaceWindow>>(&output).ok())
        .unwrap_or_default()
        .into_iter()
        .map(|w| Window {
            id: w.window_id,
            app: w.app_name,
            title: w.window_title.unwrap_or_default(),
            focused: false,
        })
        .collect();

    Some(Workspace {
        id: id.to_string(),
        display_name: None,
        focused: is_focused,
        visible: is_focused, // focused workspace is always visible
        windows,
        monitor: 0, // Not critical for focus change events
    })
}
