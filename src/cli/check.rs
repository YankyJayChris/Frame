//! `frame check` — environment diagnostics (like Flutter Doctor).
//!
//! Checks all tools required to build Frame apps and reports
//! their status with actionable fix instructions.

use std::process::Command;

/// Severity of a check result.
#[derive(Debug, Clone, PartialEq)]
pub enum CheckStatus {
    Ok,
    Warning,
    Error,
}

/// A single environment check result.
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub detail: String,
    pub fix: Option<String>,
}

impl CheckResult {
    fn ok(name: &str, detail: &str) -> Self {
        CheckResult { name: name.to_string(), status: CheckStatus::Ok,
            detail: detail.to_string(), fix: None }
    }
    fn warn(name: &str, detail: &str, fix: &str) -> Self {
        CheckResult { name: name.to_string(), status: CheckStatus::Warning,
            detail: detail.to_string(), fix: Some(fix.to_string()) }
    }
    fn err(name: &str, detail: &str, fix: &str) -> Self {
        CheckResult { name: name.to_string(), status: CheckStatus::Error,
            detail: detail.to_string(), fix: Some(fix.to_string()) }
    }
}

// ─── Tool detection helpers ───────────────────────────────────────────────────

fn which(cmd: &str) -> Option<String> {
    let output = Command::new("which").arg(cmd).output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn cmd_version(cmd: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(cmd).args(args).output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = if stdout.is_empty() { stderr } else { stdout };
    Some(combined.lines().next()?.trim().to_string())
}

fn env_var(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

// ─── Individual checks ────────────────────────────────────────────────────────

fn check_rust() -> CheckResult {
    match cmd_version("rustc", &["--version"]) {
        Some(v) => CheckResult::ok("Rust toolchain", &v),
        None    => CheckResult::err("Rust toolchain", "rustc not found",
            "Install Rust: https://rustup.rs"),
    }
}

fn check_cargo() -> CheckResult {
    match cmd_version("cargo", &["--version"]) {
        Some(v) => CheckResult::ok("Cargo", &v),
        None    => CheckResult::err("Cargo", "cargo not found",
            "Install Rust (includes cargo): https://rustup.rs"),
    }
}

fn check_java() -> CheckResult {
    match cmd_version("java", &["-version"]) {
        Some(v) => {
            // Java 17+ required for Android Gradle 8
            let is_17_plus = v.contains("17") || v.contains("18") || v.contains("19")
                || v.contains("20") || v.contains("21") || v.contains("22");
            if is_17_plus {
                CheckResult::ok("Java JDK", &v)
            } else {
                CheckResult::warn("Java JDK", &format!("{} (JDK 17+ required)", v),
                    "Install JDK 17+: https://adoptium.net")
            }
        }
        None => CheckResult::err("Java JDK", "java not found (required for Android builds)",
            "Install JDK 17: https://adoptium.net  or  brew install openjdk@17"),
    }
}

fn check_android_sdk() -> CheckResult {
    let sdk_root = env_var("ANDROID_HOME")
        .or_else(|| env_var("ANDROID_SDK_ROOT"));

    match sdk_root {
        Some(path) => {
            let tools_path = std::path::Path::new(&path).join("tools").join("bin");
            let platform_tools = std::path::Path::new(&path).join("platform-tools").join("adb");
            if platform_tools.exists() {
                CheckResult::ok("Android SDK", &format!("Found at {path}"))
            } else {
                CheckResult::warn("Android SDK", &format!("ANDROID_HOME={path} but platform-tools missing"),
                    "Open Android Studio → SDK Manager → install platform-tools")
            }
        }
        None => CheckResult::err("Android SDK",
            "ANDROID_HOME not set",
            "1. Install Android Studio: https://developer.android.com/studio\n   \
             2. Open SDK Manager and install SDK 34 + NDK\n   \
             3. Add to shell: export ANDROID_HOME=$HOME/Library/Android/sdk"),
    }
}

fn check_android_ndk() -> CheckResult {
    let sdk_root = env_var("ANDROID_HOME").or_else(|| env_var("ANDROID_SDK_ROOT"));
    if let Some(path) = sdk_root {
        let ndk_path = std::path::Path::new(&path).join("ndk");
        if ndk_path.exists() {
            // List NDK versions
            let versions: Vec<String> = std::fs::read_dir(&ndk_path).ok()
                .map(|entries| entries.filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().to_string()).collect())
                .unwrap_or_default();
            if versions.is_empty() {
                return CheckResult::warn("Android NDK", "NDK directory exists but no versions installed",
                    "Open Android Studio → SDK Manager → SDK Tools → NDK (Side by side)");
            }
            return CheckResult::ok("Android NDK", &format!("Versions: {}", versions.join(", ")));
        }
        return CheckResult::warn("Android NDK", "NDK not installed (required for native plugins)",
            "Android Studio → SDK Manager → SDK Tools → NDK (Side by side)");
    }
    CheckResult::warn("Android NDK", "Cannot check (ANDROID_HOME not set)", "Set ANDROID_HOME first")
}

fn check_gradle() -> CheckResult {
    match cmd_version("gradle", &["--version"]) {
        Some(v) => {
            let is_8 = v.contains("Gradle 8.");
            if is_8 {
                CheckResult::ok("Gradle", &v)
            } else {
                CheckResult::warn("Gradle", &format!("{} (Gradle 8.x recommended)", v),
                    "frame build uses the Gradle wrapper (gradlew) — no system install needed")
            }
        }
        None => CheckResult::warn("Gradle", "Not found in PATH (optional — Frame uses wrapper)",
            "frame deploy android uses ./gradlew; no system Gradle needed"),
    }
}

fn check_xcode() -> CheckResult {
    #[cfg(target_os = "macos")]
    {
        match cmd_version("xcodebuild", &["-version"]) {
            Some(v) => {
                // Xcode 15+ recommended
                CheckResult::ok("Xcode", &v)
            }
            None => CheckResult::err("Xcode", "xcodebuild not found (required for iOS builds)",
                "Install Xcode from the App Store: https://apps.apple.com/app/xcode/id497799835\n   \
                 Then: sudo xcode-select --switch /Applications/Xcode.app"),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        CheckResult::warn("Xcode", "iOS builds require macOS with Xcode",
            "Build iOS apps on a macOS machine with Xcode installed")
    }
}

fn check_cocoapods() -> CheckResult {
    #[cfg(target_os = "macos")]
    {
        match cmd_version("pod", &["--version"]) {
            Some(v) => CheckResult::ok("CocoaPods", &v),
            None    => CheckResult::warn("CocoaPods", "Not installed (required for iOS plugins)",
                "sudo gem install cocoapods  OR  brew install cocoapods"),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        CheckResult::warn("CocoaPods", "Only needed on macOS for iOS builds", "")
    }
}

fn check_adb() -> CheckResult {
    match cmd_version("adb", &["version"]) {
        Some(v) => CheckResult::ok("ADB (Android Debug Bridge)", &v),
        None    => CheckResult::warn("ADB", "Not in PATH (needed to run on Android devices)",
            "Add $ANDROID_HOME/platform-tools to your PATH"),
    }
}

fn check_simulator() -> CheckResult {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("xcrun").args(["simctl", "list", "devices", "available"])
            .output();
        match output {
            Ok(o) if o.status.success() => {
                let out = String::from_utf8_lossy(&o.stdout);
                let count = out.lines().filter(|l| l.contains("(Booted)") || l.contains("iPhone")).count();
                CheckResult::ok("iOS Simulator", &format!("{} simulator(s) available", count))
            }
            _ => CheckResult::warn("iOS Simulator", "xcrun simctl failed",
                "Open Xcode → Window → Devices and Simulators"),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        CheckResult::warn("iOS Simulator", "Only available on macOS", "")
    }
}

fn check_frame_config() -> CheckResult {
    if std::path::Path::new("frame.config.json").exists() {
        // Try to parse it
        match std::fs::read_to_string("frame.config.json") {
            Ok(content) => {
                if content.contains("bundle_id") {
                    CheckResult::ok("frame.config.json", "Found and valid")
                } else {
                    CheckResult::warn("frame.config.json", "Missing required field 'bundle_id'",
                        "Add: \"bundle_id\": \"com.yourcompany.appname\"")
                }
            }
            Err(e) => CheckResult::err("frame.config.json", &format!("Cannot read: {e}"), "Check file permissions"),
        }
    } else {
        CheckResult::warn("frame.config.json", "Not found (run from project root)",
            "Run: cd <your-project-dir>  OR  frame start <name>")
    }
}

// ─── Public entry point ───────────────────────────────────────────────────────

/// Run all environment checks and print results.
/// Returns `true` if all required checks passed (no errors).
pub fn run_check(target: &str, _fix: bool) -> bool {
    println!();
    println!("Frame Doctor — checking your development environment");
    println!("{}", "─".repeat(55));

    let mut checks: Vec<CheckResult> = Vec::new();

    // Always check
    checks.push(check_rust());
    checks.push(check_cargo());
    checks.push(check_frame_config());

    let check_android = target == "all" || target == "android";
    let check_ios     = target == "all" || target == "ios";

    if check_android {
        println!();
        println!("[Android]");
        checks.push(check_java());
        checks.push(check_android_sdk());
        checks.push(check_android_ndk());
        checks.push(check_gradle());
        checks.push(check_adb());
    }

    if check_ios {
        println!();
        println!("[iOS]");
        checks.push(check_xcode());
        checks.push(check_cocoapods());
        checks.push(check_simulator());
    }

    println!();
    println!("[Summary]");
    println!();

    let mut all_ok = true;

    for result in &checks {
        let (icon, label) = match result.status {
            CheckStatus::Ok      => ("✓", "\x1b[32m"),  // green
            CheckStatus::Warning => ("!", "\x1b[33m"),  // yellow
            CheckStatus::Error   => ("✗", "\x1b[31m"),  // red
        };
        let reset = "\x1b[0m";
        println!("  {label}{icon}{reset}  {} — {}", result.name, result.detail);

        if let Some(fix) = &result.fix {
            println!("       {} Fix: {}", " ".repeat(result.name.len()), fix);
        }

        if result.status == CheckStatus::Error {
            all_ok = false;
        }
    }

    println!();
    if all_ok {
        println!("✓ All checks passed! You're ready to build Frame apps.");
    } else {
        println!("✗ Some issues need attention before you can build.");
        println!("  Run: frame check --fix  to attempt automatic fixes.");
    }
    println!();

    all_ok
}

/// Attempt to auto-fix common issues (called with --fix flag).
pub fn run_fix(target: &str) {
    println!("frame check --fix is not yet supported for automatic installs.");
    println!("Please follow the fix instructions above for each failing check.");
    println!();
    println!("Quick links:");
    println!("  Rust:          https://rustup.rs");
    println!("  JDK 17:        https://adoptium.net");
    println!("  Android Studio: https://developer.android.com/studio");
    if target == "all" || target == "ios" {
        println!("  Xcode:         https://apps.apple.com/app/xcode/id497799835");
        println!("  CocoaPods:     sudo gem install cocoapods");
    }
}
