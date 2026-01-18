//! Network Watcher
//!
//! Monitors network state changes.
//! Currently uses a timer-based approach for reliability.
//! Emits `network-changed` event when network state changes.
//!
//! Future improvement: Use SCDynamicStore for true event-driven monitoring.

use serde::Serialize;
use std::process::Command;
use std::sync::Once;
use std::time::Duration;
use sysinfo::Networks;
use tauri::{AppHandle, Emitter, async_runtime};

static INIT: Once = Once::new();

// Check interval (5 seconds - more responsive than UI polling)
const NETWORK_CHECK_INTERVAL_SECS: u64 = 5;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NetworkEvent {
    pub interface: String,
    #[serde(rename = "type")]
    pub network_type: String,
    pub ssid: Option<String>,
    pub signal_strength: Option<i32>,
    pub connected: bool,
}

/// Register the network watcher
pub fn register(app_handle: AppHandle) -> Result<(), String> {
    INIT.call_once(|| {
        let handle = app_handle.clone();
        async_runtime::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(NETWORK_CHECK_INTERVAL_SECS));
            let mut last_state: Option<NetworkEvent> = None;

            loop {
                ticker.tick().await;

                if let Ok(event) = get_network_info() {
                    // Only emit if state changed
                    let should_emit = match &last_state {
                        Some(last) => last != &event,
                        None => true,
                    };

                    if should_emit {
                        last_state = Some(event.clone());
                        let _ = handle.emit("network-changed", event);
                    }
                }
            }
        });
    });

    Ok(())
}

/// Get current network info
fn get_network_info() -> Result<NetworkEvent, String> {
    let networks = Networks::new_with_refreshed_list();

    // Find the primary network interface (usually en0 for WiFi on macOS)
    for (interface_name, _network) in &networks {
        if interface_name.starts_with("en") {
            // Try to get WiFi info
            let wifi_info = get_wifi_info();

            return Ok(NetworkEvent {
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

    Ok(NetworkEvent {
        interface: "unknown".to_string(),
        network_type: "unknown".to_string(),
        ssid: None,
        signal_strength: None,
        connected: false,
    })
}

fn get_wifi_info() -> Option<(String, Option<i32>)> {
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
                let signal = get_wifi_signal_strength();
                return Some((ssid, signal));
            }
        }
    }

    None
}

fn get_wifi_signal_strength() -> Option<i32> {
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
