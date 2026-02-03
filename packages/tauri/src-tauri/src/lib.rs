mod cli;
mod commands;
mod ipc;
mod watchers;
mod windows;

use clap::{Parser, Subcommand};
use commands::{
    aerospace_focus_workspace, aerospace_get_focused_workspace, aerospace_get_workspaces,
    clear_icon_cache, close_all_popovers, close_popover, execute_shell, get_active_app_info,
    get_app_icon, get_app_icons, get_battery_info, get_bluetooth_info, get_brightness_info,
    get_config, get_cpu_info, get_disk_info, get_media_info, get_memory_info, get_monitors,
    get_network_info, get_open_popovers, get_volume_info, media_next, media_pause, media_play,
    media_previous, open_popover, save_config, set_brightness, set_mute, set_volume,
    set_window_geometry, set_window_position, set_window_size, store_delete, store_get,
    store_keys, store_set, toggle_bluetooth, toggle_mute,
};
use windows::{
    close_window, create_inline_window, hide_window, show_window, update_window_position,
};
use once_cell::sync::OnceCell;
use std::path::PathBuf;
use tauri::http::Response;
use tauri::{Emitter, Manager};

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
            // Inline window commands
            create_inline_window,
            update_window_position,
            hide_window,
            close_window,
            show_window,
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
            // Shell commands
            execute_shell,
        ])
        .register_uri_scheme_protocol("arcana", |ctx, request| {
            // Combine host and path for routing
            // arcana://localhost/index.html -> host="localhost", path="/index.html"
            let uri = request.uri();
            let host = uri.host().unwrap_or("");
            let uri_path = uri.path();
            let path = if host.is_empty() || host == "localhost" {
                uri_path.to_string()
            } else {
                format!("/{}{}", host, uri_path)
            };
            let path = if path == "/" || path.is_empty() {
                "/index.html".to_string()
            } else {
                path
            };
            let path = path.as_str();

            // Helper: get MIME type for file
            let get_mime = |path: &PathBuf| -> &'static str {
                match path.extension().and_then(|e| e.to_str()) {
                    Some("html") => "text/html",
                    Some("js") | Some("mjs") => "application/javascript",
                    Some("css") => "text/css",
                    Some("json") => "application/json",
                    Some("png") => "image/png",
                    Some("jpg") | Some("jpeg") => "image/jpeg",
                    Some("gif") => "image/gif",
                    Some("svg") => "image/svg+xml",
                    Some("ico") => "image/x-icon",
                    Some("woff") => "font/woff",
                    Some("woff2") => "font/woff2",
                    Some("ttf") => "font/ttf",
                    Some("otf") => "font/otf",
                    _ => "application/octet-stream",
                }
            };

            // Helper: serve file with MIME type
            let serve_file = |file_path: &PathBuf| -> Response<Vec<u8>> {
                if file_path.exists() {
                    match std::fs::read(file_path) {
                        Ok(content) => {
                            Response::builder()
                                .header("Content-Type", get_mime(file_path))
                                .header("Access-Control-Allow-Origin", "*")
                                .body(content)
                                .unwrap()
                        }
                        Err(_) => Response::builder().status(404).body(Vec::new()).unwrap(),
                    }
                } else {
                    Response::builder().status(404).body(Vec::new()).unwrap()
                }
            };

            // Helper: create error response when UI is not found
            let ui_not_found_response = || -> Response<Vec<u8>> {
                let config_dir = commands::config::get_config_dir();
                let default_dist = config_dir.join("dist");
                let html = format!(
                    r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Arcana - UI Not Found</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            padding: 40px;
            background: #1a1a1a;
            color: #e0e0e0;
            line-height: 1.6;
        }}
        h1 {{ color: #ff6b6b; margin-bottom: 20px; }}
        h2 {{ color: #4ecdc4; margin-top: 30px; }}
        code {{
            background: #2d2d2d;
            padding: 2px 8px;
            border-radius: 4px;
            font-family: 'SF Mono', Monaco, monospace;
        }}
        pre {{
            background: #2d2d2d;
            padding: 16px;
            border-radius: 8px;
            overflow-x: auto;
        }}
        ol {{ padding-left: 24px; }}
        li {{ margin: 8px 0; }}
    </style>
</head>
<body>
    <h1>Arcana UI Not Found</h1>
    <p>No user interface distribution was found. Arcana requires a built UI to display.</p>

    <h2>Setup Options</h2>

    <h3>Option 1: Default Location</h3>
    <p>Place your built UI in:</p>
    <pre><code>{}</code></pre>

    <h3>Option 2: Custom Path</h3>
    <p>Set <code>ui.distPath</code> in <code>~/.config/arcana/arcana.json</code>:</p>
    <pre><code>{{
  "version": 2,
  "ui": {{
    "distPath": "~/path/to/your/dist"
  }}
}}</code></pre>

    <h2>Quick Start</h2>
    <ol>
        <li>Clone the starter: <code>git clone https://github.com/.../arcana-starter-vue</code></li>
        <li>Install dependencies: <code>npm install</code></li>
        <li>Build: <code>npm run build</code></li>
        <li>Link: <code>ln -s $(pwd)/dist {}</code></li>
        <li>Restart Arcana</li>
    </ol>
</body>
</html>"#,
                    default_dist.display(),
                    default_dist.display()
                );
                Response::builder()
                    .status(503)
                    .header("Content-Type", "text/html; charset=utf-8")
                    .body(html.into_bytes())
                    .unwrap()
            };

            // Route: /lib/{file} - Serve shared libraries for widget runtime
            if path.starts_with("/lib/") {
                let file = &path[5..];

                // Try resource directory first (bundled with app in production)
                if let Ok(resource_dir) = ctx.app_handle().path().resource_dir() {
                    let lib_path: PathBuf = resource_dir.join("libs").join(file);
                    if lib_path.exists() {
                        return serve_file(&lib_path);
                    }
                }

                // Fallback: development mode - look in src-tauri/libs/
                // This works when running `cargo tauri dev`
                let dev_lib_path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("libs")
                    .join(file);
                if dev_lib_path.exists() {
                    return serve_file(&dev_lib_path);
                }

                return Response::builder().status(404).body(Vec::new()).unwrap();
            }

            // Route: User UI - Serve from user's dist folder
            // Priority: 1. config ui.distPath, 2. ~/.config/arcana/dist/
            let ui_dist = match commands::config::get_ui_dist_path() {
                Some(path) => path,
                None => return ui_not_found_response(),
            };

            // Determine file to serve
            let file_path = if path == "/index.html" {
                ui_dist.join("index.html")
            } else {
                // Remove leading slash and serve from dist
                let relative = path.trim_start_matches('/');
                ui_dist.join(relative)
            };

            // Try to serve the file
            if file_path.exists() {
                return serve_file(&file_path);
            }

            // SPA fallback: serve index.html for non-existent paths (Vue Router support)
            let index_path = ui_dist.join("index.html");
            if index_path.exists() {
                return serve_file(&index_path);
            }

            Response::builder().status(404).body(Vec::new()).unwrap()
        })
        .setup(|app| {
            // Store AppHandle globally for event emission from native callbacks
            GLOBAL_APP_HANDLE.set(app.handle().clone()).ok();

            // Start IPC server for CLI commands
            ipc::start_server(app.handle().clone());

            // Initialize system watchers (active app, battery, volume, media, network)
            watchers::init_all(app.handle().clone());

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

                // Observer for app deactivation (close popovers when clicking outside Arcana)
                define_class!(
                    #[unsafe(super(NSObject))]
                    #[name = "AppDeactivateObserver"]
                    #[ivars = ()]
                    struct AppDeactivateObserver;

                    impl AppDeactivateObserver {
                        #[unsafe(method(appDidResignActive:))]
                        fn app_did_resign_active(&self, _notification: &NSNotification) {
                            if let Some(handle) = GLOBAL_APP_HANDLE.get() {
                                let _ = close_all_popovers(handle.clone());
                            }
                        }
                    }
                );

                static REGISTER_OBSERVER: Once = Once::new();

                REGISTER_OBSERVER.call_once(|| {
                    // Screen change observer
                    let screen_observer: Retained<ScreenChangeObserver> = unsafe {
                        msg_send![ScreenChangeObserver::alloc(), init]
                    };

                    let center = NSNotificationCenter::defaultCenter();
                    let screen_name = NSString::from_str("NSApplicationDidChangeScreenParametersNotification");

                    unsafe {
                        center.addObserver_selector_name_object(
                            &screen_observer,
                            sel!(screenDidChange:),
                            Some(&screen_name),
                            None,
                        );
                    }

                    // App deactivate observer (close popovers when app loses focus)
                    let deactivate_observer: Retained<AppDeactivateObserver> = unsafe {
                        msg_send![AppDeactivateObserver::alloc(), init]
                    };

                    let deactivate_name = NSString::from_str("NSApplicationDidResignActiveNotification");

                    unsafe {
                        center.addObserver_selector_name_object(
                            &deactivate_observer,
                            sel!(appDidResignActive:),
                            Some(&deactivate_name),
                            None,
                        );
                    }

                    // Leak observers to keep them alive
                    std::mem::forget(screen_observer);
                    std::mem::forget(deactivate_observer);
                });
            }

            // Window position/size is now controlled by useArcanaInit in the frontend

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
