//! System Monitor Watcher
//!
//! Monitors CPU and Memory usage using a timer-based approach.
//! Emits `cpu-changed` and `memory-changed` events at regular intervals.

use once_cell::sync::Lazy;
use serde::Serialize;
use std::sync::{Mutex, Once};
use std::time::Duration;
use sysinfo::System;
use tauri::{AppHandle, Emitter, async_runtime};

static INIT: Once = Once::new();
static SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| Mutex::new(System::new()));

// Interval for CPU/Memory monitoring (5 seconds)
const MONITOR_INTERVAL_SECS: u64 = 5;

#[derive(Debug, Clone, Serialize)]
pub struct CpuEvent {
    pub usage: f32,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryEvent {
    pub total: u64,
    pub used: u64,
    pub usage: f32,
}

/// Register the system monitor watcher
pub fn register(app_handle: AppHandle) -> Result<(), String> {
    INIT.call_once(|| {
        // Spawn a tokio task for periodic monitoring
        let handle = app_handle.clone();
        async_runtime::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(MONITOR_INTERVAL_SECS));

            loop {
                ticker.tick().await;

                // Get CPU info
                if let Ok(cpu_event) = get_cpu_info() {
                    let _ = handle.emit("cpu-changed", cpu_event);
                }

                // Get Memory info
                if let Ok(memory_event) = get_memory_info() {
                    let _ = handle.emit("memory-changed", memory_event);
                }
            }
        });
    });

    Ok(())
}

/// Get current CPU info
fn get_cpu_info() -> Result<CpuEvent, String> {
    let mut sys = SYSTEM.lock().map_err(|e| format!("Lock error: {}", e))?;
    sys.refresh_cpu_all();

    let usage = sys.global_cpu_usage();

    Ok(CpuEvent {
        usage,
        temperature: None, // Temperature requires additional macOS APIs
    })
}

/// Get current Memory info
fn get_memory_info() -> Result<MemoryEvent, String> {
    let mut sys = SYSTEM.lock().map_err(|e| format!("Lock error: {}", e))?;
    sys.refresh_memory();

    let total = sys.total_memory();
    let used = sys.used_memory();
    let usage = if total > 0 {
        (used as f32 / total as f32) * 100.0
    } else {
        0.0
    };

    Ok(MemoryEvent { total, used, usage })
}
