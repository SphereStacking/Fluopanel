use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use tauri::{AppHandle, Emitter, Manager};

use crate::commands::{aerospace_get_workspaces_sync, get_workspace_by_id};

const SOCKET_PATH: &str = "/tmp/fluopanel.sock";

/// Start the IPC server (called from main app)
pub fn start_server(app: AppHandle) {
    // Remove existing socket file if it exists
    let _ = std::fs::remove_file(SOCKET_PATH);

    std::thread::spawn(move || {
        let listener = match UnixListener::bind(SOCKET_PATH) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[IPC] Failed to bind socket: {}", e);
                return;
            }
        };

        println!("[IPC] Server listening on {}", SOCKET_PATH);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let app = app.clone();
                    std::thread::spawn(move || {
                        handle_client(stream, &app);
                    });
                }
                Err(e) => {
                    eprintln!("[IPC] Connection error: {}", e);
                }
            }
        }
    });
}

/// Handle incoming client connection
fn handle_client(stream: UnixStream, app: &AppHandle) {
    let reader = BufReader::new(&stream);

    for line in reader.lines() {
        match line {
            Ok(command) => {
                println!("[IPC] Received command: {}", command);
                execute_command(&command, app);
            }
            Err(e) => {
                eprintln!("[IPC] Read error: {}", e);
                break;
            }
        }
    }
}

/// Execute a command received via IPC
fn execute_command(command: &str, app: &AppHandle) {
    // Handle focus-changed:focused:prev format
    if let Some(rest) = command.strip_prefix("focus-changed:") {
        let parts: Vec<&str> = rest.split(':').collect();
        let focused_id = parts.first().map(|s| s.trim()).filter(|s| !s.is_empty());
        let prev_id = parts.get(1).map(|s| s.trim()).filter(|s| !s.is_empty());

        if let Some(focused) = focused_id {
            let focused_ws = get_workspace_by_id(focused, true);
            let prev_ws = prev_id.and_then(|id| get_workspace_by_id(id, false));

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit(
                    "aerospace-focus-changed",
                    serde_json::json!({
                        "focused": focused_ws,
                        "prev": prev_ws
                    }),
                );
            }
        }
        return;
    }

    // Legacy: full workspace refresh
    match command {
        "workspace-changed" => {
            if let Ok(workspaces) = aerospace_get_workspaces_sync() {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.emit("aerospace-workspace-changed", &workspaces);
                }
            }
        }
        _ => {}
    }
}

/// Send a command to the running instance (CLI mode)
pub fn send_command(event: &str) -> bool {
    let socket_path = Path::new(SOCKET_PATH);

    if !socket_path.exists() {
        eprintln!("fluopanel is not running (socket not found)");
        return false;
    }

    match UnixStream::connect(socket_path) {
        Ok(mut stream) => {
            if let Err(e) = writeln!(stream, "{}", event) {
                eprintln!("Failed to send command: {}", e);
                return false;
            }
            true
        }
        Err(e) => {
            eprintln!("Failed to connect to fluopanel: {}", e);
            false
        }
    }
}
