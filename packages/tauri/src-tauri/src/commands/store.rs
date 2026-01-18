use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{command, AppHandle, Emitter};

/// Global in-memory store for cross-window state sharing
static STORE: Lazy<Mutex<HashMap<String, Value>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Set a value in the shared store and broadcast to all windows
#[command]
pub fn store_set(app: AppHandle, key: String, value: Value) -> Result<(), String> {
    let mut store = STORE.lock().map_err(|e| e.to_string())?;
    store.insert(key.clone(), value.clone());

    // Broadcast to all windows
    let event_name = format!("store-changed:{}", key);
    app.emit(&event_name, value).map_err(|e| e.to_string())?;

    Ok(())
}

/// Get a value from the shared store
#[command]
pub fn store_get(key: String) -> Result<Option<Value>, String> {
    let store = STORE.lock().map_err(|e| e.to_string())?;
    Ok(store.get(&key).cloned())
}

/// Delete a value from the shared store
#[command]
pub fn store_delete(app: AppHandle, key: String) -> Result<(), String> {
    let mut store = STORE.lock().map_err(|e| e.to_string())?;
    store.remove(&key);

    // Broadcast deletion (null value indicates removal)
    let event_name = format!("store-changed:{}", key);
    app.emit(&event_name, Value::Null).map_err(|e| e.to_string())?;

    Ok(())
}

/// Get all keys in the store
#[command]
pub fn store_keys() -> Result<Vec<String>, String> {
    let store = STORE.lock().map_err(|e| e.to_string())?;
    Ok(store.keys().cloned().collect())
}
