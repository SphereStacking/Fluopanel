mod commands;
mod ipc;
mod widgets;

use clap::{Parser, Subcommand};
use commands::{
    aerospace_focus_workspace, aerospace_get_focused_workspace, aerospace_get_workspaces,
    clear_icon_cache, close_all_popups, close_popup_window, create_popup_window,
    get_active_app_info, get_app_icon, get_app_icons, get_battery_info, get_bluetooth_info,
    get_brightness_info, get_config, get_cpu_info, get_disk_info, get_media_info,
    get_memory_info, get_monitors, get_network_info, get_open_popups, get_registered_triggers,
    get_volume_info, media_next, media_pause, media_play, media_previous, popup_trigger_enter,
    popup_trigger_leave, popup_window_enter, popup_window_leave, register_hover_trigger,
    save_config, set_brightness, set_mute, set_volume, set_window_geometry, set_window_position,
    set_window_size, store_delete, store_get, store_keys, store_set, toggle_bluetooth,
    toggle_mute, unregister_hover_trigger, update_popup_position, update_trigger_bounds,
};
use widgets::{
    close_widget_window, create_inline_widget_window, create_widget_window,
    discover_widgets, get_widget_manifest, get_widget_windows, get_widgets_dir,
    hide_window, show_widget_window, update_widget_position,
};
use once_cell::sync::OnceCell;
use std::path::PathBuf;
use tauri::http::Response;
use tauri::Emitter;
use tauri::Manager;

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
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
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
            // Widget commands
            discover_widgets,
            get_widget_manifest,
            create_widget_window,
            close_widget_window,
            get_widget_windows,
            show_widget_window,
            // Inline widget commands
            create_inline_widget_window,
            update_widget_position,
            hide_window,
            // Popup commands
            create_popup_window,
            close_popup_window,
            close_all_popups,
            get_open_popups,
            update_popup_position,
            // Popup hover coordination commands
            popup_trigger_enter,
            popup_trigger_leave,
            popup_window_enter,
            popup_window_leave,
            // Trigger registration commands
            register_hover_trigger,
            unregister_hover_trigger,
            update_trigger_bounds,
            get_registered_triggers,
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

            // Check if this is a widget request: /widget/{widget_id}/{file_path}
            let file_path = if path.starts_with("/widget/") {
                let parts: Vec<&str> = path[8..].splitn(2, '/').collect();
                if parts.len() >= 1 {
                    let widget_id = parts[0];
                    let file = if parts.len() >= 2 { parts[1] } else { "index.html" };
                    get_widgets_dir()
                        .map(|d| d.join(widget_id).join(file))
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

            // Hide from Dock (set as accessory app)
            #[cfg(target_os = "macos")]
            {
                use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy};
                unsafe {
                    let app_instance = NSApp();
                    app_instance.setActivationPolicy_(NSApplicationActivationPolicy::NSApplicationActivationPolicyAccessory);
                }
            }

            // Register display change observer (macOS)
            #[cfg(target_os = "macos")]
            {
                use cocoa::base::{id, nil};
                use cocoa::foundation::NSString;
                use objc::runtime::{Object, Sel};
                use objc::{class, declare::ClassDecl, msg_send, sel, sel_impl};
                use std::sync::Once;

                static REGISTER_OBSERVER: Once = Once::new();

                REGISTER_OBSERVER.call_once(|| {
                    unsafe {
                        // Create observer class dynamically
                        let superclass = class!(NSObject);
                        let mut decl = ClassDecl::new("ScreenChangeObserver", superclass).unwrap();

                        extern "C" fn handle_screen_change(
                            _this: &Object,
                            _cmd: Sel,
                            _notif: id,
                        ) {
                            if let Some(handle) = GLOBAL_APP_HANDLE.get() {
                                let _ = handle.emit("monitor-changed", ());
                            }
                        }

                        decl.add_method(
                            sel!(screenDidChange:),
                            handle_screen_change as extern "C" fn(&Object, Sel, id),
                        );

                        let observer_class = decl.register();
                        let observer: id = msg_send![observer_class, new];

                        // Register with NSNotificationCenter
                        let center: id = msg_send![class!(NSNotificationCenter), defaultCenter];
                        let name = NSString::alloc(nil)
                            .init_str("NSApplicationDidChangeScreenParametersNotification");
                        let _: () = msg_send![center, addObserver:observer
                                                      selector:sel!(screenDidChange:)
                                                      name:name
                                                      object:nil];
                    }
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
