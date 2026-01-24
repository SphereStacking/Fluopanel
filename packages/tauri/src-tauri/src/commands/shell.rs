use std::process::Command;
use tauri::command;

#[command]
pub fn execute_shell(command: String) -> Result<String, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
