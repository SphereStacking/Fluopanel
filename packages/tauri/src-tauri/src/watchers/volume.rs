//! Volume Watcher
//!
//! Monitors audio volume and mute state changes using Core Audio APIs.
//! Emits `volume-changed` event when volume or mute state changes.

use crate::commands::audio;
use coreaudio_sys::*;
use serde::Serialize;
use std::os::raw::c_void;
use std::sync::Once;
use tauri::{AppHandle, Emitter};

static INIT: Once = Once::new();
static mut APP_HANDLE: Option<AppHandle> = None;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeEvent {
    pub volume: f32,
    pub muted: bool,
    pub output_device: Option<String>,
}

/// Callback function for volume changes
extern "C" fn volume_listener_callback(
    _object_id: AudioObjectID,
    _number_addresses: u32,
    _addresses: *const AudioObjectPropertyAddress,
    _client_data: *mut c_void,
) -> OSStatus {
    if let Some(handle) = unsafe { APP_HANDLE.as_ref() } {
        let event = get_current_volume_info();
        let _ = handle.emit("volume-changed", event);
    }
    0 // noErr
}

/// Register the volume watcher
pub fn register(app_handle: AppHandle) -> Result<(), String> {
    INIT.call_once(|| {
        unsafe {
            APP_HANDLE = Some(app_handle);
        }

        // Get default output device
        if let Ok(device_id) = audio::get_default_output_device() {
            // Listen for volume changes
            let volume_address = AudioObjectPropertyAddress {
                mSelector: kAudioHardwareServiceDeviceProperty_VirtualMainVolume,
                mScope: kAudioDevicePropertyScopeOutput,
                mElement: kAudioObjectPropertyElementMain,
            };

            unsafe {
                AudioObjectAddPropertyListener(
                    device_id,
                    &volume_address,
                    Some(volume_listener_callback),
                    std::ptr::null_mut(),
                );
            }

            // Listen for mute changes
            let mute_address = AudioObjectPropertyAddress {
                mSelector: kAudioDevicePropertyMute,
                mScope: kAudioDevicePropertyScopeOutput,
                mElement: kAudioObjectPropertyElementMain,
            };

            unsafe {
                AudioObjectAddPropertyListener(
                    device_id,
                    &mute_address,
                    Some(volume_listener_callback),
                    std::ptr::null_mut(),
                );
            }
        }

        // Listen for default device changes
        let device_address = AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDefaultOutputDevice,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };

        unsafe {
            AudioObjectAddPropertyListener(
                kAudioObjectSystemObject,
                &device_address,
                Some(device_changed_callback),
                std::ptr::null_mut(),
            );
        }
    });

    Ok(())
}

/// Callback for when the default output device changes
extern "C" fn device_changed_callback(
    _object_id: AudioObjectID,
    _number_addresses: u32,
    _addresses: *const AudioObjectPropertyAddress,
    _client_data: *mut c_void,
) -> OSStatus {
    // Re-register listeners for the new device
    if let Ok(device_id) = audio::get_default_output_device() {
        let volume_address = AudioObjectPropertyAddress {
            mSelector: kAudioHardwareServiceDeviceProperty_VirtualMainVolume,
            mScope: kAudioDevicePropertyScopeOutput,
            mElement: kAudioObjectPropertyElementMain,
        };

        unsafe {
            AudioObjectAddPropertyListener(
                device_id,
                &volume_address,
                Some(volume_listener_callback),
                std::ptr::null_mut(),
            );
        }

        let mute_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyMute,
            mScope: kAudioDevicePropertyScopeOutput,
            mElement: kAudioObjectPropertyElementMain,
        };

        unsafe {
            AudioObjectAddPropertyListener(
                device_id,
                &mute_address,
                Some(volume_listener_callback),
                std::ptr::null_mut(),
            );
        }
    }

    // Emit volume changed event for the new device
    if let Some(handle) = unsafe { APP_HANDLE.as_ref() } {
        let event = get_current_volume_info();
        let _ = handle.emit("volume-changed", event);
    }

    0 // noErr
}

/// Get current volume info
fn get_current_volume_info() -> VolumeEvent {
    let volume = audio::get_output_volume().unwrap_or(0.0) * 100.0;
    let muted = audio::is_muted().unwrap_or(false);
    let output_device = audio::get_output_device_name().ok();

    VolumeEvent {
        volume,
        muted,
        output_device,
    }
}
