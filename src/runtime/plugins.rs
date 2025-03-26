#[cfg(target_os = "android")]
use jni::JNIEnv;
#[cfg(target_os = "ios")]
use objc::{msg_send, sel, sel_impl};

// Camera plugin
pub fn camera() -> String {
    #[cfg(target_os = "android")]
    {
        let env = JNIEnv::from_raw(unsafe { jni::sys::JNI_GetCreatedJavaVM() }).unwrap();
        let activity = env.get_static_field("android/app/Activity", "currentActivity", "Landroid/app/Activity;").unwrap();
        let _ = env.call_method(activity.l().unwrap(), "startCamera", "()V", &[]).unwrap();
        "Camera opened".to_string()
    }
    #[cfg(target_os = "ios")]
    {
        unsafe {
            let ui_image_picker: *mut objc::runtime::Object = msg_send![class!(UIImagePickerController), alloc];
            let _: () = msg_send![ui_image_picker, init];
            "Camera opened".to_string()
        }
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    "Camera accessed (desktop)".to_string()
}

// Location plugin
pub fn location() -> String {
    #[cfg(target_os = "android")]
    {
        let env = JNIEnv::from_raw(unsafe { jni::sys::JNI_GetCreatedJavaVM() }).unwrap();
        let location_manager = env.get_static_field("android/location/LocationManager", "GPS_PROVIDER", "Ljava/lang/String;").unwrap();
        "Location accessed".to_string()
    }
    #[cfg(target_os = "ios")]
    {
        unsafe {
            let location_manager: *mut objc::runtime::Object = msg_send![class!(CLLocationManager), alloc];
            let _: () = msg_send![location_manager, init];
            "Location accessed".to_string()
        }
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    "Location accessed (desktop)".to_string()
}

// Notification plugin
pub fn notification(msg: &str) -> String {
    #[cfg(target_os = "android")]
    {
        let env = JNIEnv::from_raw(unsafe { jni::sys::JNI_GetCreatedJavaVM() }).unwrap();
        let notification_manager = env.get_static_field("android/app/NotificationManager", "IMPORTANCE_DEFAULT", "I").unwrap();
        "Notification sent".to_string()
    }
    #[cfg(target_os = "ios")]
    {
        unsafe {
            let notification_center: *mut objc::runtime::Object = msg_send![class!(UNUserNotificationCenter), currentNotificationCenter];
            let _: () = msg_send![notification_center, requestAuthorizationWithOptions:0 completionHandler:nil];
            "Notification sent".to_string()
        }
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        println!("Notification: {}", msg);
        "Notification sent (desktop)".to_string()
    }
}