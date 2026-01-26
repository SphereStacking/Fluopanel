//! Widget file watcher
//!
//! Monitors the widgets directory for changes to .vue, .jsx, .tsx files
//! and triggers automatic rebuilds.

use crate::windows::get_windows_dir;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

/// Debounce duration for file changes (ms)
const DEBOUNCE_MS: u64 = 500;

/// File extensions that trigger a rebuild
const BUILD_EXTENSIONS: &[&str] = &["vue", "jsx", "tsx", "ts", "js", "css", "scss"];

/// Get the path to the builder script
fn get_builder_script(app: &AppHandle) -> Option<PathBuf> {
    // Try resource directory first (production)
    if let Ok(resource_dir) = app.path().resource_dir() {
        let builder_path: PathBuf = resource_dir.join("builder").join("build.mjs");
        if builder_path.exists() {
            return Some(builder_path);
        }
    }

    // Fallback: development mode
    let dev_path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("builder")
        .join("build.mjs");

    if dev_path.exists() {
        Some(dev_path)
    } else {
        None
    }
}

/// Extract widget ID from a file path
fn get_widget_id(path: &Path, widgets_dir: &Path) -> Option<String> {
    let relative = path.strip_prefix(widgets_dir).ok()?;
    let components: Vec<_> = relative.components().collect();
    if components.is_empty() {
        return None;
    }
    components[0]
        .as_os_str()
        .to_str()
        .map(|s| s.to_string())
}

/// Check if a file should trigger a rebuild
fn should_rebuild(path: &Path) -> bool {
    // Skip .arcana directory (build output)
    if path
        .components()
        .any(|c| c.as_os_str() == ".arcana" || c.as_os_str() == "node_modules")
    {
        return false;
    }

    // Check extension
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| BUILD_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Build a widget
fn build_widget(app: &AppHandle, widget_id: &str, widgets_dir: &Path) {
    let builder_script = match get_builder_script(app) {
        Some(path) => path,
        None => {
            eprintln!("[WidgetWatcher] Builder script not found");
            return;
        }
    };

    let widget_dir = widgets_dir.join(widget_id);

    // Check if widget has buildable source files
    let has_source = widget_dir.read_dir().ok().map_or(false, |entries| {
        entries.flatten().any(|e| {
            let path = e.path();
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "vue" || ext == "jsx" || ext == "tsx")
                .unwrap_or(false)
        })
    });

    if !has_source {
        return; // No buildable source files
    }

    eprintln!("[WidgetWatcher] Building widget: {}", widget_id);

    let output = Command::new("node")
        .arg(&builder_script)
        .arg("--widget")
        .arg(&widget_dir)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            eprintln!("[WidgetWatcher] Build complete: {}", widget_id);
            // Emit event to notify frontend
            let _ = app.emit("widget-rebuilt", widget_id);
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("[WidgetWatcher] Build failed for {}: {}", widget_id, stderr);
            let _ = app.emit(
                "widget-build-error",
                serde_json::json!({
                    "widgetId": widget_id,
                    "error": stderr.to_string()
                }),
            );
        }
        Err(e) => {
            eprintln!("[WidgetWatcher] Failed to run builder: {}", e);
        }
    }
}

/// Register the widget file watcher
pub fn register(app_handle: AppHandle) -> Result<(), String> {
    let widgets_dir = get_windows_dir()?;

    if !widgets_dir.exists() {
        eprintln!("[WidgetWatcher] Widgets directory doesn't exist, skipping watcher");
        return Ok(());
    }

    eprintln!("[WidgetWatcher] Starting watcher for: {:?}", widgets_dir);

    // Track pending rebuilds with debouncing
    let pending_rebuilds: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    let last_event: Arc<Mutex<Instant>> = Arc::new(Mutex::new(Instant::now()));

    let (tx, rx) = channel();

    let widgets_dir_clone = widgets_dir.clone();
    let app_clone = app_handle.clone();
    let pending_clone = pending_rebuilds.clone();
    let last_clone = last_event.clone();

    // Create watcher
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                // Only handle modify and create events
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) => {
                        for path in event.paths {
                            if should_rebuild(&path) {
                                if let Some(widget_id) = get_widget_id(&path, &widgets_dir_clone) {
                                    let mut pending = pending_clone.lock().unwrap();
                                    pending.insert(widget_id);
                                    *last_clone.lock().unwrap() = Instant::now();
                                    let _ = tx.send(());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        },
        Config::default(),
    )
    .map_err(|e| format!("Failed to create watcher: {}", e))?;

    watcher
        .watch(&widgets_dir, RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch directory: {}", e))?;

    // Spawn debounce thread
    std::thread::spawn(move || {
        loop {
            // Wait for an event
            if rx.recv().is_err() {
                break;
            }

            // Debounce: wait for DEBOUNCE_MS without new events
            loop {
                std::thread::sleep(Duration::from_millis(DEBOUNCE_MS));
                let elapsed = last_event.lock().unwrap().elapsed();
                if elapsed >= Duration::from_millis(DEBOUNCE_MS) {
                    break;
                }
            }

            // Process pending rebuilds
            let widgets_to_build: Vec<String> = {
                let mut pending = pending_rebuilds.lock().unwrap();
                pending.drain().collect()
            };

            if let Ok(widgets_dir) = get_windows_dir() {
                for widget_id in widgets_to_build {
                    build_widget(&app_clone, &widget_id, &widgets_dir);
                }
            }
        }
    });

    // Keep watcher alive
    std::mem::forget(watcher);

    Ok(())
}
