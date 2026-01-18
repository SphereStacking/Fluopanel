//! Core Audio wrapper for volume control
//!
//! Uses native macOS Core Audio APIs instead of osascript for better performance.

#![cfg(target_os = "macos")]

use coreaudio_sys::*;
use std::os::raw::c_void;

/// Get the default output audio device ID
pub fn get_default_output_device() -> Result<AudioObjectID, String> {
    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioHardwarePropertyDefaultOutputDevice,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain,
    };

    let mut device_id: AudioObjectID = 0;
    let mut size = std::mem::size_of::<AudioObjectID>() as u32;

    let status = unsafe {
        AudioObjectGetPropertyData(
            kAudioObjectSystemObject,
            &property_address,
            0,
            std::ptr::null(),
            &mut size,
            &mut device_id as *mut AudioObjectID as *mut c_void,
        )
    };

    if status == 0 {
        Ok(device_id)
    } else {
        Err(format!("Failed to get default output device: {}", status))
    }
}

/// Get the master volume of the default output device (0.0 - 1.0)
pub fn get_output_volume() -> Result<f32, String> {
    let device_id = get_default_output_device()?;

    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioHardwareServiceDeviceProperty_VirtualMainVolume,
        mScope: kAudioDevicePropertyScopeOutput,
        mElement: kAudioObjectPropertyElementMain,
    };

    let mut volume: f32 = 0.0;
    let mut size = std::mem::size_of::<f32>() as u32;

    let status = unsafe {
        AudioObjectGetPropertyData(
            device_id,
            &property_address,
            0,
            std::ptr::null(),
            &mut size,
            &mut volume as *mut f32 as *mut c_void,
        )
    };

    if status == 0 {
        Ok(volume)
    } else {
        // Fallback: try per-channel volume
        get_channel_volume(device_id)
    }
}

/// Get volume from the first available channel
fn get_channel_volume(device_id: AudioObjectID) -> Result<f32, String> {
    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyVolumeScalar,
        mScope: kAudioDevicePropertyScopeOutput,
        mElement: 1, // Channel 1 (left)
    };

    let mut volume: f32 = 0.0;
    let mut size = std::mem::size_of::<f32>() as u32;

    let status = unsafe {
        AudioObjectGetPropertyData(
            device_id,
            &property_address,
            0,
            std::ptr::null(),
            &mut size,
            &mut volume as *mut f32 as *mut c_void,
        )
    };

    if status == 0 {
        Ok(volume)
    } else {
        Err(format!("Failed to get volume: {}", status))
    }
}

/// Set the master volume of the default output device (0.0 - 1.0)
pub fn set_output_volume(volume: f32) -> Result<(), String> {
    let device_id = get_default_output_device()?;
    let volume = volume.clamp(0.0, 1.0);

    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioHardwareServiceDeviceProperty_VirtualMainVolume,
        mScope: kAudioDevicePropertyScopeOutput,
        mElement: kAudioObjectPropertyElementMain,
    };

    let status = unsafe {
        AudioObjectSetPropertyData(
            device_id,
            &property_address,
            0,
            std::ptr::null(),
            std::mem::size_of::<f32>() as u32,
            &volume as *const f32 as *const c_void,
        )
    };

    if status == 0 {
        Ok(())
    } else {
        // Fallback: set per-channel volume
        set_channel_volume(device_id, volume)
    }
}

/// Set volume on both channels
fn set_channel_volume(device_id: AudioObjectID, volume: f32) -> Result<(), String> {
    for channel in 1..=2 {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyVolumeScalar,
            mScope: kAudioDevicePropertyScopeOutput,
            mElement: channel,
        };

        let status = unsafe {
            AudioObjectSetPropertyData(
                device_id,
                &property_address,
                0,
                std::ptr::null(),
                std::mem::size_of::<f32>() as u32,
                &volume as *const f32 as *const c_void,
            )
        };

        if status != 0 && channel == 1 {
            return Err(format!("Failed to set volume: {}", status));
        }
    }
    Ok(())
}

/// Check if the default output device is muted
pub fn is_muted() -> Result<bool, String> {
    let device_id = get_default_output_device()?;

    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyMute,
        mScope: kAudioDevicePropertyScopeOutput,
        mElement: kAudioObjectPropertyElementMain,
    };

    let mut muted: u32 = 0;
    let mut size = std::mem::size_of::<u32>() as u32;

    let status = unsafe {
        AudioObjectGetPropertyData(
            device_id,
            &property_address,
            0,
            std::ptr::null(),
            &mut size,
            &mut muted as *mut u32 as *mut c_void,
        )
    };

    if status == 0 {
        Ok(muted != 0)
    } else {
        // Some devices don't support mute property, assume not muted
        Ok(false)
    }
}

/// Set mute state of the default output device
pub fn set_muted(muted: bool) -> Result<(), String> {
    let device_id = get_default_output_device()?;

    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyMute,
        mScope: kAudioDevicePropertyScopeOutput,
        mElement: kAudioObjectPropertyElementMain,
    };

    let mute_value: u32 = if muted { 1 } else { 0 };

    let status = unsafe {
        AudioObjectSetPropertyData(
            device_id,
            &property_address,
            0,
            std::ptr::null(),
            std::mem::size_of::<u32>() as u32,
            &mute_value as *const u32 as *const c_void,
        )
    };

    if status == 0 {
        Ok(())
    } else {
        Err(format!("Failed to set mute: {}", status))
    }
}

/// Get the name of the default output device
pub fn get_output_device_name() -> Result<String, String> {
    let device_id = get_default_output_device()?;

    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyDeviceNameCFString,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain,
    };

    let mut name_ref: core_foundation_sys::string::CFStringRef = std::ptr::null();
    let mut size = std::mem::size_of::<core_foundation_sys::string::CFStringRef>() as u32;

    let status = unsafe {
        AudioObjectGetPropertyData(
            device_id,
            &property_address,
            0,
            std::ptr::null(),
            &mut size,
            &mut name_ref as *mut _ as *mut c_void,
        )
    };

    if status == 0 && !name_ref.is_null() {
        let name = unsafe { cfstring_to_string(name_ref) };
        Ok(name)
    } else {
        Err(format!("Failed to get device name: {}", status))
    }
}

/// Convert CFString to Rust String
unsafe fn cfstring_to_string(cf_string: core_foundation_sys::string::CFStringRef) -> String {
    use core_foundation_sys::string::*;

    let length = CFStringGetLength(cf_string);
    let max_size = CFStringGetMaximumSizeForEncoding(length, kCFStringEncodingUTF8) + 1;
    let mut buffer = vec![0u8; max_size as usize];

    if CFStringGetCString(
        cf_string,
        buffer.as_mut_ptr() as *mut i8,
        max_size,
        kCFStringEncodingUTF8,
    ) != 0
    {
        let c_str = std::ffi::CStr::from_ptr(buffer.as_ptr() as *const i8);
        c_str.to_string_lossy().into_owned()
    } else {
        String::new()
    }
}
