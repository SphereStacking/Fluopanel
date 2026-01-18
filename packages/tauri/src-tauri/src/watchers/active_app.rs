//! Active Application Watcher
//!
//! Monitors frontmost application changes using NSWorkspace notifications.
//! Emits `active-app-changed` event when the user switches to a different app.

use objc2::rc::Retained;
use objc2::{define_class, msg_send, sel, ClassType};
use objc2_app_kit::NSWorkspace;
use objc2_foundation::{NSNotification, NSNotificationName, NSObject, NSObjectProtocol};
use serde::Serialize;
use std::sync::Once;
use tauri::{AppHandle, Emitter};

static INIT: Once = Once::new();
static mut APP_HANDLE: Option<AppHandle> = None;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveAppEvent {
    pub name: String,
    pub bundle_id: Option<String>,
    pub pid: Option<i32>,
}

/// Register the active application watcher
pub fn register(app_handle: AppHandle) -> Result<(), String> {
    INIT.call_once(|| {
        // Store app handle for callback
        unsafe {
            APP_HANDLE = Some(app_handle);
        }

        // Define observer class
        define_class!(
            #[unsafe(super(NSObject))]
            #[name = "ActiveAppObserver"]
            #[ivars = ()]
            struct ActiveAppObserver;

            unsafe impl NSObjectProtocol for ActiveAppObserver {}

            impl ActiveAppObserver {
                #[unsafe(method(appDidActivate:))]
                fn app_did_activate(&self, notification: &NSNotification) {
                    if let Some(handle) = unsafe { APP_HANDLE.as_ref() } {
                        // Get the activated app info from notification userInfo
                        let event = get_frontmost_app_info();
                        let _ = handle.emit("active-app-changed", event);
                    }
                }
            }
        );

        // Create observer instance
        let observer: Retained<ActiveAppObserver> =
            unsafe { msg_send![ActiveAppObserver::class(), new] };

        // Get workspace notification center (not default center)
        let workspace = NSWorkspace::sharedWorkspace();
        let notification_center = workspace.notificationCenter();

        // Register for app activation notification
        let notification_name =
            NSNotificationName::from_str("NSWorkspaceDidActivateApplicationNotification");

        unsafe {
            notification_center.addObserver_selector_name_object(
                &*observer,
                sel!(appDidActivate:),
                Some(&*notification_name),
                None,
            );
        }

        // Prevent observer from being deallocated
        std::mem::forget(observer);
    });

    Ok(())
}

/// Get current frontmost application info
fn get_frontmost_app_info() -> ActiveAppEvent {
    let workspace = NSWorkspace::sharedWorkspace();
    let front_app = workspace.frontmostApplication();

    match front_app {
        Some(app) => {
            let name = app
                .localizedName()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            let bundle_id = app.bundleIdentifier().map(|s| s.to_string());
            let pid = Some(app.processIdentifier());

            ActiveAppEvent {
                name,
                bundle_id,
                pid,
            }
        }
        None => ActiveAppEvent {
            name: "Unknown".to_string(),
            bundle_id: None,
            pid: None,
        },
    }
}
