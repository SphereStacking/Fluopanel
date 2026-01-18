use once_cell::sync::Lazy;
use serde::Serialize;
use std::sync::Mutex;
use sysinfo::{Disks, Networks, System};
use tauri::command;

// 静的 System インスタンス（再利用してメモリ節約）
static SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| Mutex::new(System::new()));
static DISKS: Lazy<Mutex<Disks>> = Lazy::new(|| Mutex::new(Disks::new_with_refreshed_list()));

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryInfo {
    pub percent: f32,
    pub charging: bool,
    pub time_to_empty: Option<i32>,
    pub time_to_full: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct CpuInfo {
    pub usage: f32,
    pub temperature: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub usage: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInfo {
    pub interface: String,
    #[serde(rename = "type")]
    pub network_type: String,
    pub ssid: Option<String>,
    pub signal_strength: Option<i32>,
    pub connected: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeInfo {
    pub volume: f32,
    pub muted: bool,
    pub output_device: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveAppInfo {
    pub name: String,
    pub bundle_id: Option<String>,
    pub icon: Option<String>,
    pub pid: Option<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub usage: f32,
    pub mount_point: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub playing: bool,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Option<f64>,
    pub position: Option<f64>,
    pub app: Option<String>,
    pub artwork_url: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrightnessInfo {
    pub brightness: f32,
    pub display_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BluetoothDevice {
    pub name: String,
    pub address: String,
    pub connected: bool,
    pub device_type: Option<String>,
    pub battery_level: Option<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BluetoothInfo {
    pub enabled: bool,
    pub devices: Vec<BluetoothDevice>,
}

#[command]
pub fn get_battery_info() -> Result<BatteryInfo, String> {
    let manager = battery::Manager::new()
        .map_err(|e| format!("Failed to create battery manager: {}", e))?;

    let mut batteries = manager
        .batteries()
        .map_err(|e| format!("Failed to get batteries: {}", e))?;

    if let Some(Ok(battery)) = batteries.next() {
        use battery::State;

        let percent = battery.state_of_charge().value * 100.0;
        let charging = matches!(battery.state(), State::Charging | State::Full);

        let time_to_empty = battery.time_to_empty().map(|t| (t.value / 60.0) as i32);
        let time_to_full = battery.time_to_full().map(|t| (t.value / 60.0) as i32);

        Ok(BatteryInfo {
            percent,
            charging,
            time_to_empty,
            time_to_full,
        })
    } else {
        // No battery found (desktop Mac)
        Ok(BatteryInfo {
            percent: 100.0,
            charging: true,
            time_to_empty: None,
            time_to_full: None,
        })
    }
}

#[command]
pub fn get_cpu_info() -> Result<CpuInfo, String> {
    let mut sys = SYSTEM.lock().map_err(|e| format!("Lock error: {}", e))?;
    sys.refresh_cpu_all();

    let usage = sys.global_cpu_usage();

    Ok(CpuInfo {
        usage,
        temperature: None, // Temperature requires additional macOS APIs
    })
}

#[command]
pub fn get_memory_info() -> Result<MemoryInfo, String> {
    let mut sys = SYSTEM.lock().map_err(|e| format!("Lock error: {}", e))?;
    sys.refresh_memory();

    let total = sys.total_memory();
    let used = sys.used_memory();
    let usage = if total > 0 {
        (used as f32 / total as f32) * 100.0
    } else {
        0.0
    };

    Ok(MemoryInfo { total, used, usage })
}

#[command]
pub fn get_network_info() -> Result<NetworkInfo, String> {
    let networks = Networks::new_with_refreshed_list();

    // Find the primary network interface (usually en0 for WiFi on macOS)
    for (interface_name, _network) in &networks {
        if interface_name.starts_with("en") {
            // Try to get WiFi info using system_profiler
            let wifi_info = get_wifi_info();

            return Ok(NetworkInfo {
                interface: interface_name.clone(),
                network_type: if interface_name == "en0" {
                    "wifi".to_string()
                } else {
                    "ethernet".to_string()
                },
                ssid: wifi_info.as_ref().map(|(ssid, _)| ssid.clone()),
                signal_strength: wifi_info.as_ref().and_then(|(_, strength)| *strength),
                connected: true,
            });
        }
    }

    Ok(NetworkInfo {
        interface: "unknown".to_string(),
        network_type: "unknown".to_string(),
        ssid: None,
        signal_strength: None,
        connected: false,
    })
}

fn get_wifi_info() -> Option<(String, Option<i32>)> {
    use std::process::Command;

    // Use networksetup to get current WiFi network
    let output = Command::new("/usr/sbin/networksetup")
        .args(["-getairportnetwork", "en0"])
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Output format: "Current Wi-Fi Network: NetworkName"
        if let Some(ssid) = stdout.strip_prefix("Current Wi-Fi Network: ") {
            let ssid = ssid.trim().to_string();
            if !ssid.is_empty() && ssid != "You are not associated with an AirPort network." {
                // Get signal strength using airport utility
                let signal = get_wifi_signal_strength();
                return Some((ssid, signal));
            }
        }
    }

    None
}

fn get_wifi_signal_strength() -> Option<i32> {
    use std::process::Command;

    let output = Command::new("/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport")
        .args(["-I"])
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("agrCtlRSSI:") {
                let rssi: i32 = line.split(':').nth(1)?.trim().parse().ok()?;
                // Convert RSSI to percentage (rough approximation)
                // RSSI typically ranges from -30 (excellent) to -90 (very weak)
                let percentage = ((rssi + 90) * 100 / 60).clamp(0, 100);
                return Some(percentage);
            }
        }
    }

    None
}

// ============================================
// Volume commands (Native Core Audio API)
// ============================================

#[command]
pub fn get_volume_info() -> Result<VolumeInfo, String> {
    #[cfg(target_os = "macos")]
    {
        use super::audio;

        let volume = audio::get_output_volume().unwrap_or(0.0) * 100.0;
        let muted = audio::is_muted().unwrap_or(false);
        let output_device = audio::get_output_device_name().ok();

        Ok(VolumeInfo {
            volume,
            muted,
            output_device,
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(VolumeInfo {
            volume: 0.0,
            muted: false,
            output_device: None,
        })
    }
}

#[command]
pub fn set_volume(level: f32) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use super::audio;
        let level = level.clamp(0.0, 100.0) / 100.0;
        audio::set_output_volume(level)
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = level;
        Ok(())
    }
}

#[command]
pub fn set_mute(muted: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use super::audio;
        audio::set_muted(muted)
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = muted;
        Ok(())
    }
}

#[command]
pub fn toggle_mute() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use super::audio;
        let muted = audio::is_muted().unwrap_or(false);
        audio::set_muted(!muted)
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(())
    }
}

// ============================================
// Active App commands (Native NSWorkspace API)
// ============================================

#[command]
pub fn get_active_app_info() -> Result<ActiveAppInfo, String> {
    #[cfg(target_os = "macos")]
    {
        use objc2_app_kit::NSWorkspace;

        let workspace = NSWorkspace::sharedWorkspace();
        let front_app = workspace.frontmostApplication();

        match front_app {
            Some(app) => {
                let name = app
                    .localizedName()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                let bundle_id = app.bundleIdentifier().map(|s| s.to_string());

                let pid = app.processIdentifier();

                Ok(ActiveAppInfo {
                    name,
                    bundle_id,
                    icon: None, // Icon can be fetched separately using get_app_icon
                    pid: Some(pid),
                })
            }
            None => Ok(ActiveAppInfo {
                name: "Unknown".to_string(),
                bundle_id: None,
                icon: None,
                pid: None,
            }),
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(ActiveAppInfo {
            name: "Unknown".to_string(),
            bundle_id: None,
            icon: None,
            pid: None,
        })
    }
}

// ============================================
// Disk commands
// ============================================

#[command]
pub fn get_disk_info() -> Result<Vec<DiskInfo>, String> {
    let mut disks = DISKS.lock().map_err(|e| format!("Lock error: {}", e))?;
    disks.refresh_list();

    let result: Vec<DiskInfo> = disks
        .iter()
        .filter(|disk| {
            // Filter out system volumes and snapshots
            let mount = disk.mount_point().to_string_lossy();
            !mount.starts_with("/System")
                && !mount.contains("TimeMachine")
                && !mount.contains(".Snapshot")
        })
        .map(|disk| {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);
            let usage = if total > 0 {
                (used as f32 / total as f32) * 100.0
            } else {
                0.0
            };

            DiskInfo {
                total,
                used,
                available,
                usage,
                mount_point: disk.mount_point().to_string_lossy().to_string(),
            }
        })
        .collect();

    Ok(result)
}

// ============================================
// Media commands
// ============================================

#[command]
pub fn get_media_info() -> Result<MediaInfo, String> {
    use std::process::Command;

    // Try to get Now Playing info using osascript
    // This works with Music.app, Spotify, and other media apps
    let script = r#"
        set mediaInfo to ""

        -- Try Spotify first
        if application "Spotify" is running then
            tell application "Spotify"
                if player state is playing then
                    set mediaInfo to "true|" & name of current track & "|" & artist of current track & "|" & album of current track & "|" & (duration of current track / 1000) & "|" & (player position) & "|Spotify|"
                else if player state is paused then
                    set mediaInfo to "false|" & name of current track & "|" & artist of current track & "|" & album of current track & "|" & (duration of current track / 1000) & "|" & (player position) & "|Spotify|"
                end if
            end tell
        end if

        -- Try Music.app if no Spotify info
        if mediaInfo is "" and application "Music" is running then
            tell application "Music"
                if player state is playing then
                    set currentTrack to current track
                    set mediaInfo to "true|" & name of currentTrack & "|" & artist of currentTrack & "|" & album of currentTrack & "|" & (duration of currentTrack) & "|" & player position & "|Music|"
                else if player state is paused then
                    set currentTrack to current track
                    set mediaInfo to "false|" & name of currentTrack & "|" & artist of currentTrack & "|" & album of currentTrack & "|" & (duration of currentTrack) & "|" & player position & "|Music|"
                end if
            end tell
        end if

        return mediaInfo
    "#;

    let output = Command::new("osascript")
        .args(["-e", script])
        .output()
        .map_err(|e| format!("Failed to get media info: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = stdout.trim().split('|').collect();

        if parts.len() >= 7 && !parts[0].is_empty() {
            return Ok(MediaInfo {
                playing: parts[0] == "true",
                title: Some(parts[1].to_string()).filter(|s| !s.is_empty()),
                artist: Some(parts[2].to_string()).filter(|s| !s.is_empty()),
                album: Some(parts[3].to_string()).filter(|s| !s.is_empty()),
                duration: parts[4].parse().ok(),
                position: parts[5].parse().ok(),
                app: Some(parts[6].to_string()).filter(|s| !s.is_empty()),
                artwork_url: None,
            });
        }
    }

    // No media playing
    Ok(MediaInfo {
        playing: false,
        title: None,
        artist: None,
        album: None,
        duration: None,
        position: None,
        app: None,
        artwork_url: None,
    })
}

#[command]
pub fn media_play() -> Result<(), String> {
    use std::process::Command;

    // Try Spotify first, then Music
    let script = r#"
        if application "Spotify" is running then
            tell application "Spotify" to play
        else if application "Music" is running then
            tell application "Music" to play
        end if
    "#;

    Command::new("osascript")
        .args(["-e", script])
        .output()
        .map_err(|e| format!("Failed to play: {}", e))?;

    Ok(())
}

#[command]
pub fn media_pause() -> Result<(), String> {
    use std::process::Command;

    let script = r#"
        if application "Spotify" is running then
            tell application "Spotify" to pause
        else if application "Music" is running then
            tell application "Music" to pause
        end if
    "#;

    Command::new("osascript")
        .args(["-e", script])
        .output()
        .map_err(|e| format!("Failed to pause: {}", e))?;

    Ok(())
}

#[command]
pub fn media_next() -> Result<(), String> {
    use std::process::Command;

    let script = r#"
        if application "Spotify" is running then
            tell application "Spotify" to next track
        else if application "Music" is running then
            tell application "Music" to next track
        end if
    "#;

    Command::new("osascript")
        .args(["-e", script])
        .output()
        .map_err(|e| format!("Failed to skip: {}", e))?;

    Ok(())
}

#[command]
pub fn media_previous() -> Result<(), String> {
    use std::process::Command;

    let script = r#"
        if application "Spotify" is running then
            tell application "Spotify" to previous track
        else if application "Music" is running then
            tell application "Music" to previous track
        end if
    "#;

    Command::new("osascript")
        .args(["-e", script])
        .output()
        .map_err(|e| format!("Failed to go back: {}", e))?;

    Ok(())
}

// ============================================
// Brightness commands (Native IOKit API)
// ============================================

#[command]
pub fn get_brightness_info() -> Result<BrightnessInfo, String> {
    #[cfg(target_os = "macos")]
    {
        use super::brightness;

        let brightness_value = brightness::get_brightness().unwrap_or(0.5);

        Ok(BrightnessInfo {
            brightness: brightness_value * 100.0,
            display_name: None, // TODO: Get display name
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(BrightnessInfo {
            brightness: 100.0,
            display_name: None,
        })
    }
}

#[command]
pub fn set_brightness(level: f32) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use super::brightness;
        let level = level.clamp(0.0, 100.0) / 100.0;
        brightness::set_brightness(level)
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = level;
        Ok(())
    }
}

// ============================================
// Bluetooth commands
// ============================================

#[command]
pub fn get_bluetooth_info() -> Result<BluetoothInfo, String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        // Check if Bluetooth is enabled
        let power_output = Command::new("defaults")
            .args(["read", "/Library/Preferences/com.apple.Bluetooth", "ControllerPowerState"])
            .output();

        let enabled = power_output
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "1")
            .unwrap_or(false);

        // Get connected devices using system_profiler
        let devices = get_bluetooth_devices().unwrap_or_default();

        Ok(BluetoothInfo { enabled, devices })
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(BluetoothInfo {
            enabled: false,
            devices: vec![],
        })
    }
}

#[cfg(target_os = "macos")]
fn get_bluetooth_devices() -> Result<Vec<BluetoothDevice>, String> {
    use std::process::Command;

    let output = Command::new("system_profiler")
        .args(["SPBluetoothDataType", "-json"])
        .output()
        .map_err(|e| format!("Failed to get Bluetooth info: {}", e))?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let mut devices = Vec::new();

    // Parse connected devices from system_profiler output
    if let Some(bt_data) = json.get("SPBluetoothDataType").and_then(|v| v.as_array()) {
        for entry in bt_data {
            // Parse connected devices
            if let Some(connected) = entry.get("device_connected").and_then(|v| v.as_array()) {
                for device in connected {
                    if let Some(device_obj) = device.as_object() {
                        for (name, info) in device_obj {
                            let address = info
                                .get("device_address")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            let device_type = info
                                .get("device_minorType")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());

                            // Try to get battery level if available
                            let battery_level = info
                                .get("device_batteryLevelMain")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.trim_end_matches('%').parse().ok());

                            devices.push(BluetoothDevice {
                                name: name.clone(),
                                address,
                                connected: true,
                                device_type,
                                battery_level,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(devices)
}

#[command]
pub fn toggle_bluetooth() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        // Toggle Bluetooth using blueutil if available, otherwise use AppleScript
        let blueutil_result = Command::new("blueutil").args(["--power", "toggle"]).output();

        if blueutil_result.is_ok() {
            return Ok(());
        }

        // Fallback: use osascript (requires accessibility permissions)
        let script = r#"
            tell application "System Preferences"
                reveal pane id "com.apple.preferences.Bluetooth"
                activate
            end tell
        "#;

        Command::new("osascript")
            .args(["-e", script])
            .output()
            .map_err(|e| format!("Failed to toggle Bluetooth: {}", e))?;

        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(())
    }
}
