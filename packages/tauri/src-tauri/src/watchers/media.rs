//! Media Watcher
//!
//! Monitors Now Playing media state changes.
//! Uses AppleScript to query Spotify/Music apps.
//! Emits `media-changed` event when media state changes.
//!
//! Note: Uses polling with change detection due to private MediaRemote API.
//! Future improvement: Use MRMediaRemoteRegisterForNowPlayingNotifications.

use serde::Serialize;
use std::process::Command;
use std::sync::Once;
use std::time::Duration;
use tauri::{AppHandle, Emitter, async_runtime};

static INIT: Once = Once::new();

// Check interval (5 seconds - balanced between responsiveness and CPU usage)
const MEDIA_CHECK_INTERVAL_SECS: u64 = 5;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaEvent {
    pub playing: bool,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Option<f64>,
    pub position: Option<f64>,
    pub app: Option<String>,
    pub artwork_url: Option<String>,
}

/// Register the media watcher
pub fn register(app_handle: AppHandle) -> Result<(), String> {
    INIT.call_once(|| {
        let handle = app_handle.clone();
        async_runtime::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(MEDIA_CHECK_INTERVAL_SECS));
            let mut last_state: Option<MediaEvent> = None;

            loop {
                ticker.tick().await;

                if let Ok(event) = get_media_info() {
                    // Only emit if state changed (ignoring position for comparison)
                    let should_emit = match &last_state {
                        Some(last) => !media_states_equal(last, &event),
                        None => true,
                    };

                    if should_emit {
                        last_state = Some(event.clone());
                        let _ = handle.emit("media-changed", event);
                    }
                }
            }
        });
    });

    Ok(())
}

/// Compare media states, ignoring position (which always changes)
fn media_states_equal(a: &MediaEvent, b: &MediaEvent) -> bool {
    a.playing == b.playing
        && a.title == b.title
        && a.artist == b.artist
        && a.album == b.album
        && a.app == b.app
}

/// Get current media info
fn get_media_info() -> Result<MediaEvent, String> {
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
            return Ok(MediaEvent {
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
    Ok(MediaEvent {
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
