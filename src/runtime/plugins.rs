//! Native plugin bridge stubs.
//!
//! Platform-specific implementations are conditionally compiled.
//! Full plugin system: Task 19.

// Android native bridge
#[cfg(target_os = "android")]
use jni::JNIEnv;

// macOS/iOS native bridge
#[cfg(target_os = "macos")]
#[allow(unused_imports)]
use objc::{msg_send, sel, sel_impl};

/// Open the device camera (stub for non-mobile targets).
pub fn camera() -> String {
    #[cfg(target_os = "android")]
    { "Camera opened (android)".to_string() }
    #[cfg(not(target_os = "android"))]
    { "Camera accessed (desktop/sim)".to_string() }
}

/// Get device location (stub for non-mobile targets).
pub fn location() -> String {
    #[cfg(target_os = "android")]
    { "Location accessed (android)".to_string() }
    #[cfg(not(target_os = "android"))]
    { "Location accessed (desktop/sim)".to_string() }
}

/// Send a local notification (stub for non-mobile targets).
pub fn notification(msg: &str) -> String {
    let _ = msg;
    #[cfg(target_os = "android")]
    { "Notification sent (android)".to_string() }
    #[cfg(not(target_os = "android"))]
    { format!("Notification: {}", msg) }
}
