mod commands;
mod ipc;
mod windows;

use clap::{Parser, Subcommand};
use commands::{
    aerospace_focus_workspace, aerospace_get_focused_workspace, aerospace_get_workspaces,
    clear_icon_cache, close_all_popovers, close_popover, get_active_app_info, get_app_icon,
    get_app_icons, get_battery_info, get_bluetooth_info, get_brightness_info, get_config,
    get_cpu_info, get_disk_info, get_media_info, get_memory_info, get_monitors,
    get_network_info, get_open_popovers, get_volume_info, media_next, media_pause, media_play,
    media_previous, open_popover, save_config, set_brightness, set_mute, set_volume,
    set_window_geometry, set_window_position, set_window_size, store_delete, store_get,
    store_keys, store_set, toggle_bluetooth, toggle_mute,
};
use windows::{
    close_window, create_inline_window, create_window,
    discover_windows, get_window_manifest, get_windows, get_windows_dir,
    hide_window, show_window, update_window_position,
};
use once_cell::sync::OnceCell;
use std::path::PathBuf;
use tauri::http::Response;
use tauri::Emitter;

#[derive(Parser)]
#[command(name = "arcana")]
#[command(about = "Customizable widget framework for macOS")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Emit an event to the running instance
    Emit {
        /// Event name (e.g., workspace-changed)
        event: String,
    },
    /// Notify workspace focus change (optimized, only fetches 2 workspaces)
    FocusChanged {
        /// Focused workspace ID
        focused: String,
        /// Previous workspace ID (optional)
        prev: Option<String>,
    },
}

// Global AppHandle for emitting events from native callbacks
static GLOBAL_APP_HANDLE: OnceCell<tauri::AppHandle> = OnceCell::new();

fn get_user_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("arcana/dist"))
}

fn has_user_config() -> bool {
    get_user_config_dir()
        .map(|d| d.join("index.html").exists())
        .unwrap_or(false)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cli = Cli::parse();

    // CLI mode: send command to running instance and exit
    if let Some(command) = cli.command {
        let success = match command {
            Commands::Emit { event } => ipc::send_command(&event),
            Commands::FocusChanged { focused, prev } => {
                let cmd = match prev {
                    Some(p) => format!("focus-changed:{}:{}", focused, p),
                    None => format!("focus-changed:{}", focused),
                };
                ipc::send_command(&cmd)
            }
        };
        std::process::exit(if success { 0 } else { 1 });
    }

    // Normal app startup
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_nspanel::init());

    // MCP Bridge plugin (debug builds only)
    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }

    builder
        .invoke_handler(tauri::generate_handler![
            aerospace_get_workspaces,
            aerospace_get_focused_workspace,
            aerospace_focus_workspace,
            get_battery_info,
            get_cpu_info,
            get_memory_info,
            get_network_info,
            get_config,
            save_config,
            get_app_icon,
            get_app_icons,
            clear_icon_cache,
            get_monitors,
            set_window_geometry,
            set_window_position,
            set_window_size,
            // Volume commands
            get_volume_info,
            set_volume,
            set_mute,
            toggle_mute,
            // Active app commands
            get_active_app_info,
            // Disk commands
            get_disk_info,
            // Media commands
            get_media_info,
            media_play,
            media_pause,
            media_next,
            media_previous,
            // Brightness commands
            get_brightness_info,
            set_brightness,
            // Bluetooth commands
            get_bluetooth_info,
            toggle_bluetooth,
            // Window commands
            discover_windows,
            get_window_manifest,
            create_window,
            close_window,
            get_windows,
            show_window,
            // Inline window commands
            create_inline_window,
            update_window_position,
            hide_window,
            // Popover commands
            open_popover,
            close_popover,
            close_all_popovers,
            get_open_popovers,
            // Store commands
            store_set,
            store_get,
            store_delete,
            store_keys,
        ])
        .register_uri_scheme_protocol("arcana", |_ctx, request| {
            let path = request.uri().path();
            let path = if path == "/" || path.is_empty() {
                "/index.html"
            } else {
                path
            };

            // Check if this is a window request: /window/{window_id}/{file_path}
            let file_path = if path.starts_with("/window/") {
                let parts: Vec<&str> = path[8..].splitn(2, '/').collect();
                if parts.len() >= 1 {
                    let window_id = parts[0];
                    let file = if parts.len() >= 2 { parts[1] } else { "index.html" };
                    get_windows_dir()
                        .map(|d| d.join(window_id).join(file))
                        .unwrap_or_default()
                } else {
                    PathBuf::new()
                }
            } else if has_user_config() {
                // Legacy: serve from ~/.config/arcana/dist/
                get_user_config_dir().unwrap().join(&path[1..])
            } else {
                PathBuf::new()
            };

            if file_path.exists() {
                match std::fs::read(&file_path) {
                    Ok(content) => {
                        let mime = match file_path.extension().and_then(|e| e.to_str()) {
                            Some("html") => "text/html",
                            Some("js") => "application/javascript",
                            Some("css") => "text/css",
                            Some("json") => "application/json",
                            Some("png") => "image/png",
                            Some("svg") => "image/svg+xml",
                            Some("woff") => "font/woff",
                            Some("woff2") => "font/woff2",
                            _ => "application/octet-stream",
                        };
                        Response::builder()
                            .header("Content-Type", mime)
                            .header("Access-Control-Allow-Origin", "*")
                            .body(content)
                            .unwrap()
                    }
                    Err(_) => {
                        Response::builder()
                            .status(404)
                            .body(Vec::new())
                            .unwrap()
                    }
                }
            } else {
                Response::builder()
                    .status(404)
                    .body(Vec::new())
                    .unwrap()
            }
        })
        .setup(|app| {
            // Store AppHandle globally for event emission from native callbacks
            GLOBAL_APP_HANDLE.set(app.handle().clone()).ok();

            // Start IPC server for CLI commands
            ipc::start_server(app.handle().clone());

            // Initialize hover focus (autoraise) feature
            #[cfg(target_os = "macos")]
            windows::hover_focus::init(app.handle().clone());

            // Hide from Dock (set as accessory app)
            #[cfg(target_os = "macos")]
            {
                use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
                use objc2_foundation::MainThreadMarker;

                let mtm = MainThreadMarker::new().expect("must be on main thread");
                let app_instance = NSApplication::sharedApplication(mtm);
                app_instance.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
            }

            // Register display change observer (macOS)
            #[cfg(target_os = "macos")]
            {
                use objc2::rc::Retained;
                use objc2::{define_class, msg_send, sel, AllocAnyThread};
                use objc2_foundation::{
                    NSNotification, NSNotificationCenter, NSObject, NSString,
                };
                use std::sync::Once;

                define_class!(
                    #[unsafe(super(NSObject))]
                    #[name = "ScreenChangeObserver"]
                    #[ivars = ()]
                    struct ScreenChangeObserver;

                    impl ScreenChangeObserver {
                        #[unsafe(method(screenDidChange:))]
                        fn screen_did_change(&self, _notification: &NSNotification) {
                            if let Some(handle) = GLOBAL_APP_HANDLE.get() {
                                let _ = handle.emit("monitor-changed", ());
                            }
                        }
                    }
                );

                static REGISTER_OBSERVER: Once = Once::new();

                REGISTER_OBSERVER.call_once(|| {
                    let observer: Retained<ScreenChangeObserver> = unsafe {
                        msg_send![ScreenChangeObserver::alloc(), init]
                    };

                    let center = NSNotificationCenter::defaultCenter();
                    let name = NSString::from_str("NSApplicationDidChangeScreenParametersNotification");

                    unsafe {
                        center.addObserver_selector_name_object(
                            &observer,
                            sel!(screenDidChange:),
                            Some(&name),
                            None,
                        );
                    }

                    // Leak observer to keep it alive
                    std::mem::forget(observer);
                });
            }

            // Window position/size is now controlled by useArcanaInit in the frontend

            // Navigate to custom protocol if user config exists (only in release mode)
            // In dev mode, Tauri uses devUrl from tauri.conf.json
            #[cfg(not(debug_assertions))]
            if has_user_config() {
                let window = app.get_webview_window("main").unwrap();
                let _ = window.navigate("arcana://localhost/".parse().unwrap());
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
