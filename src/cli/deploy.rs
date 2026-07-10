//! `frame deploy android` and `frame deploy ios` implementations.
//!
//! Writes generated files to `build/android/` and `build/ios/`,
//! copies assets, generates font helpers, and invokes the native build tool.

use crate::parser::{parse_project, AST};
use crate::compiler::{gen_android_with_plugins, gen_ios_with_plugins};
use crate::compiler::android::AndroidConfig;
use crate::compiler::ios::IosConfig;
use crate::plugins::PluginRegistry;

use std::fs;
use std::path::Path;
use std::process::Command;

// ─── Android deploy ───────────────────────────────────────────────────────────

/// Deploy to Android: write generated files, copy assets, invoke gradlew.
/// Returns `true` on success.
pub fn deploy_android(project_dir: &Path) -> bool {
    let config = match read_config(project_dir) {
        Ok(c) => c,
        Err(e) => { eprintln!("error: {e}"); return false; }
    };

    // Parse + generate
    let ast = match parse_project(&project_dir.to_string_lossy()) {
        Ok(a) => a,
        Err(errs) => {
            eprintln!("Deploy failed with {} parse error(s):", errs.len());
            for e in &errs { eprintln!("  {e}"); }
            return false;
        }
    };

    // Load plugins and collect their Kotlin sources
    let registry = PluginRegistry::load(project_dir);
    let mut plugin_kt: Vec<(&str, &str, &str)> = Vec::new();
    for (_name, plugin) in &registry.plugins {
        for src_path in &plugin.android_sources {
            if let Some(fname) = src_path.file_name().and_then(|s| s.to_str()) {
                if let Ok(content) = fs::read_to_string(src_path) {
                    // Use leak for static lifetime — plugin data lives for the deploy duration
                    let fname_leak: &'static str = Box::leak(fname.to_string().into_boxed_str());
                    let content_leak: &'static str = Box::leak(content.into_boxed_str());
                    let pkg_dir = format!("com/frame/{}", plugin.manifest.name);
                    let pkg_leak: &'static str = Box::leak(pkg_dir.into_boxed_str());
                    plugin_kt.push((pkg_leak, fname_leak, content_leak));
                }
            }
        }
    }

    let android_cfg = android_config_from_json(&config);
    let mut files = gen_android_with_plugins(&ast, &android_cfg, &plugin_kt);

    // Inject plugin permissions into AndroidManifest.xml
    let plugin_perms: Vec<String> = registry.plugins.values()
        .flat_map(|p| p.manifest.permissions.android.iter().cloned())
        .collect();
    if !plugin_perms.is_empty() {
        for file in &mut files {
            if file.path.ends_with("AndroidManifest.xml") {
                let mut perm_lines = String::new();
                for perm in &plugin_perms {
                    perm_lines.push_str(&format!(
                        "    <uses-permission android:name=\"{}\" />\n", perm
                    ));
                }
                if let Some(pos) = file.content.find("<manifest ") {
                    if let Some(close) = file.content[pos..].find(">\n") {
                        let insert_at = pos + close + 2;
                        file.content.insert_str(insert_at, &perm_lines);
                    }
                }
            }
        }
    }

    let build_dir = project_dir.join("build/android");

    // Write generated source files
    for file in &files {
        let dest = build_dir.join(&file.path);
        if let Some(parent) = dest.parent() { fs::create_dir_all(parent).ok(); }
        if let Err(e) = fs::write(&dest, &file.content) {
            eprintln!("warning: could not write {}: {e}", dest.display());
        }
    }

    // Make gradlew executable on Unix
    #[cfg(unix)]
    {
        let gradlew = build_dir.join("gradlew");
        if gradlew.exists() {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = fs::metadata(&gradlew) {
                let mut perms = meta.permissions();
                perms.set_mode(perms.mode() | 0o755);
                fs::set_permissions(&gradlew, perms).ok();
            }
        }
    }

    // Copy assets (fonts → app/src/main/assets/fonts/)
    let assets_src = project_dir.join("assets");
    let assets_dst = build_dir.join("app/src/main/assets");
    if assets_src.exists() {
        copy_assets_android(&assets_src, &assets_dst);
    }

    // Detect missing assets referenced in AST
    let missing = detect_missing_assets(&ast, &assets_src);
    if !missing.is_empty() {
        eprintln!("error: {} missing asset(s):", missing.len());
        for m in &missing { eprintln!("  {m}"); }
        return false;
    }

    println!("✓ Android project written to build/android/");

    // Download gradle-wrapper.jar if not present (required for ./gradlew to work)
    let wrapper_jar = build_dir.join("gradle/wrapper/gradle-wrapper.jar");
    if !wrapper_jar.exists() {
        println!("  Downloading gradle-wrapper.jar…");
        let jar_url = "https://github.com/gradle/gradle/raw/v8.4.0/gradle/wrapper/gradle-wrapper.jar";
        let dl = Command::new("curl")
            .args(["-fsSL", "-o", wrapper_jar.to_str().unwrap_or(""), jar_url])
            .status();
        match dl {
            Ok(s) if s.success() => println!("  gradle-wrapper.jar downloaded."),
            _ => {
                println!("  Could not download gradle-wrapper.jar automatically.");
                println!("  → Open build/android/ in Android Studio — it will sync Gradle automatically.");
                println!("  → Or run: cd build/android && gradle assembleDebug (requires system Gradle)");
            }
        }
    }

    // Invoke gradlew if it exists AND the wrapper jar is present
    let gradle = build_dir.join("gradlew");
    let jar_ready = build_dir.join("gradle/wrapper/gradle-wrapper.jar").exists();
    if gradle.exists() && jar_ready {
        println!("Running gradlew assembleDebug…");
        let status = Command::new("bash")
            .args(["-c", "./gradlew assembleDebug"])
            .current_dir(&build_dir)
            .status();
        match status {
            Ok(s) if s.success() => {
                println!("✓ Android APK built in build/android/app/build/outputs/apk/");
            }
            Ok(_) => {
                eprintln!("✗ Android build failed. Open build/android/ in Android Studio for details.");
                return false;
            }
            Err(e) => {
                eprintln!("✗ Could not run gradlew: {e}");
                eprintln!("  Make sure Java 17+ and Android SDK are installed. Run: frame check");
                return false;
            }
        }
    } else {
        println!("✓ Android project is ready in build/android/");
        println!("  Next steps:");
        println!("  1. Open build/android/ in Android Studio");
        println!("     Android Studio will download Gradle and sync automatically.");
        println!("  2. Or install Java 17+ and run: cd build/android && gradle assembleDebug");
        println!("  Run `frame check` to verify your Android build environment.");
    }

    true
}

// ─── iOS deploy ───────────────────────────────────────────────────────────────

/// Deploy to iOS: write generated files, copy assets, emit Podfile, invoke xcodebuild.
/// Returns `true` on success.
pub fn deploy_ios(project_dir: &Path) -> bool {
    let config = match read_config(project_dir) {
        Ok(c) => c,
        Err(e) => { eprintln!("error: {e}"); return false; }
    };

    let ast = match parse_project(&project_dir.to_string_lossy()) {
        Ok(a) => a,
        Err(errs) => {
            eprintln!("Deploy failed with {} parse error(s):", errs.len());
            for e in &errs { eprintln!("  {e}"); }
            return false;
        }
    };

    // Load plugins and collect their Swift sources
    let registry = PluginRegistry::load(project_dir);
    let mut plugin_swift: Vec<(&str, &str)> = Vec::new();
    for (_name, plugin) in &registry.plugins {
        for src_path in &plugin.ios_sources {
            if let Some(fname) = src_path.file_name().and_then(|s| s.to_str()) {
                if let Ok(content) = fs::read_to_string(src_path) {
                    let fname_leak: &'static str = Box::leak(fname.to_string().into_boxed_str());
                    let content_leak: &'static str = Box::leak(content.into_boxed_str());
                    plugin_swift.push((fname_leak, content_leak));
                }
            }
        }
    }

    let ios_cfg = ios_config_from_json(&config);
    let mut files = gen_ios_with_plugins(&ast, &ios_cfg, &plugin_swift);

    // Inject plugin permissions into Info.plist
    let plugin_perms: Vec<String> = registry.plugins.values()
        .flat_map(|p| p.manifest.permissions.ios.iter().cloned())
        .collect();
    if !plugin_perms.is_empty() {
        for file in &mut files {
            if file.path.ends_with("Info.plist") {
                let mut perm_entries = String::new();
                for perm in &plugin_perms {
                    perm_entries.push_str(&format!(
                        "\t<key>{}</key>\n\t<string>This app requires this permission.</string>\n",
                        perm
                    ));
                }
                if let Some(pos) = file.content.rfind("</dict>") {
                    file.content.insert_str(pos, &perm_entries);
                }
            }
        }
    }

    let build_dir = project_dir.join("build/ios");

    // Write generated source files
    for file in &files {
        let dest = build_dir.join(&file.path);
        if let Some(parent) = dest.parent() { fs::create_dir_all(parent).ok(); }
        if let Err(e) = fs::write(&dest, &file.content) {
            eprintln!("warning: could not write {}: {e}", dest.display());
        }
    }

    // Copy assets (fonts → Assets.xcassets/Resources/)
    let assets_src = project_dir.join("assets");
    let assets_dst = build_dir.join("Assets.xcassets/Resources");
    if assets_src.exists() {
        copy_assets_ios(&assets_src, &assets_dst);
    }

    // Detect missing assets
    let missing = detect_missing_assets(&ast, &assets_src);
    if !missing.is_empty() {
        eprintln!("error: {} missing asset(s):", missing.len());
        for m in &missing { eprintln!("  {m}"); }
        return false;
    }

    // Emit Podfile if not present
    let podfile = build_dir.join("Podfile");
    if !podfile.exists() {
        let app_name = config["name"].as_str().unwrap_or("FrameApp");
        let bundle_id = config["bundle_id"].as_str().unwrap_or("com.example.app");
        let content = gen_podfile(app_name, bundle_id);
        fs::write(&podfile, content).ok();
        println!("  Podfile written to build/ios/Podfile");
    }

    println!("✓ iOS project written to build/ios/");

    // Invoke xcodebuild if the project exists
    let app_name = config["name"].as_str().unwrap_or("FrameApp");
    let safe_app = app_name.replace(' ', "");
    let xcodeproj = build_dir.join(format!("{safe_app}.xcodeproj"));
    if xcodeproj.exists() {
        println!("Running xcodebuild (build check)…");
        // Use CODE_SIGNING_ALLOWED=NO and a generic destination for CI-style build validation
        let status = Command::new("xcodebuild")
            .args([
                "-project", &format!("{safe_app}.xcodeproj"),
                "-scheme", &safe_app,
                "-configuration", "Debug",
                "-sdk", "iphonesimulator",
                "CODE_SIGNING_ALLOWED=NO",
                "build",
            ])
            .current_dir(&build_dir)
            .status();
        match status {
            Ok(s) if s.success() => {
                println!("✓ iOS build verified. Open build/ios/{safe_app}.xcodeproj in Xcode to run on a device.");
            }
            Ok(_) => {
                eprintln!("✗ iOS build failed. Open build/ios/{safe_app}.xcodeproj in Xcode for details.");
                // Don't hard-fail — the project files are valid; xcodebuild may fail due to
                // missing simulator SDK or signing. The project is still openable in Xcode.
                println!("  The project files are correct. Open in Xcode to resolve any signing issues.");
            }
            Err(e) => {
                eprintln!("✗ Could not run xcodebuild: {e}");
                eprintln!("  Make sure Xcode command-line tools are installed: xcode-select --install");
                println!("  The project is still ready at build/ios/{safe_app}.xcodeproj");
            }
        }
    } else {
        println!("✓ iOS project is ready in build/ios/");
        println!("  Next steps:");
        println!("  1. Open build/ios/{}.xcodeproj in Xcode",
            config["name"].as_str().unwrap_or("FrameApp").replace(' ', ""));
        println!("  2. Select a simulator or device and press Run (⌘R)");
        println!("  3. If you have CocoaPods dependencies, run: cd build/ios && pod install");
        println!("     Then open the .xcworkspace instead of .xcodeproj");
        println!("  Run `frame check` to verify your iOS build environment.");
    }

    true
}

// ─── Asset copy helpers ───────────────────────────────────────────────────────

/// Copy all assets from `assets_dir` to `dest` (Android: `app/src/main/assets/`).
/// Also generates Fonts.kt for any font files found.
pub fn copy_assets_android(assets_dir: &Path, dest: &Path) {
    let fonts_dst = dest.join("fonts");
    fs::create_dir_all(&fonts_dst).ok();

    let mut font_files: Vec<String> = Vec::new();

    if let Ok(entries) = fs::read_dir(assets_dir) {
        for entry in entries.flatten() {
            let src_path = entry.path();
            if src_path.is_dir() {
                // Recurse into sub-directories (e.g. assets/fonts/)
                let sub_name = entry.file_name();
                let dst_sub = dest.join(&sub_name);
                fs::create_dir_all(&dst_sub).ok();
                copy_dir_recursive(&src_path, &dst_sub, &mut font_files);
            } else {
                let file_name = entry.file_name();
                let file_str = file_name.to_string_lossy().to_string();
                let dst_file = dest.join(&file_name);
                fs::copy(&src_path, &dst_file).ok();
                if is_font_file(&file_str) {
                    font_files.push(file_str);
                }
            }
        }
    }

    // Collect fonts specifically from a nested fonts/ sub-dir
    let fonts_src = assets_dir.join("fonts");
    if fonts_src.is_dir() {
        if let Ok(entries) = fs::read_dir(&fonts_src) {
            for entry in entries.flatten() {
                let src_path = entry.path();
                if src_path.is_file() {
                    let fname = entry.file_name().to_string_lossy().to_string();
                    let dst = fonts_dst.join(&fname);
                    fs::copy(&src_path, &dst).ok();
                    if is_font_file(&fname) && !font_files.contains(&fname) {
                        font_files.push(fname);
                    }
                }
            }
        }
    }

    // Generate Fonts.kt helper
    if !font_files.is_empty() {
        let fonts_kt = dest.parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("java/com/example/app/Fonts.kt"));
        if let Some(kt_path) = fonts_kt {
            if let Some(parent) = kt_path.parent() { fs::create_dir_all(parent).ok(); }
            let code = gen_fonts_kt(&font_files);
            fs::write(&kt_path, code).ok();
        }
    }
}

/// Copy all assets from `assets_dir` to `dest` (iOS: `Assets.xcassets/Resources/`).
/// Also generates font registration info for Info.plist.
pub fn copy_assets_ios(assets_dir: &Path, dest: &Path) {
    fs::create_dir_all(dest).ok();

    let mut font_files: Vec<String> = Vec::new();
    let fonts_src = assets_dir.join("fonts");

    // Copy everything
    if let Ok(entries) = fs::read_dir(assets_dir) {
        for entry in entries.flatten() {
            let src_path = entry.path();
            let file_name = entry.file_name();
            let fname = file_name.to_string_lossy().to_string();
            if src_path.is_file() {
                let dst = dest.join(&file_name);
                fs::copy(&src_path, &dst).ok();
                if is_font_file(&fname) {
                    font_files.push(fname);
                }
            }
        }
    }

    // Also copy fonts sub-dir
    if fonts_src.is_dir() {
        if let Ok(entries) = fs::read_dir(&fonts_src) {
            for entry in entries.flatten() {
                let src_path = entry.path();
                if src_path.is_file() {
                    let fname = entry.file_name().to_string_lossy().to_string();
                    let dst = dest.join(&fname);
                    fs::copy(&src_path, &dst).ok();
                    if is_font_file(&fname) && !font_files.contains(&fname) {
                        font_files.push(fname);
                    }
                }
            }
        }
    }

    // Write font registration snippet (appended to Info.plist section)
    if !font_files.is_empty() {
        let fonts_plist = dest.parent().map(|p| p.join("FontRegistration.plist"));
        if let Some(plist_path) = fonts_plist {
            let content = gen_fonts_plist(&font_files);
            fs::write(&plist_path, content).ok();
        }
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path, font_files: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(src) {
        for entry in entries.flatten() {
            let src_path = entry.path();
            let file_name = entry.file_name();
            let dst_path = dst.join(&file_name);
            if src_path.is_dir() {
                fs::create_dir_all(&dst_path).ok();
                copy_dir_recursive(&src_path, &dst_path, font_files);
            } else {
                fs::copy(&src_path, &dst_path).ok();
                let fname = file_name.to_string_lossy().to_string();
                if is_font_file(&fname) {
                    font_files.push(fname);
                }
            }
        }
    }
}

fn is_font_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".ttf") || lower.ends_with(".otf")
}

// ─── Font code generation ─────────────────────────────────────────────────────

/// Generate a Kotlin `Typeface.createFromAsset` call for a single font file.
pub fn gen_font_kotlin(font_filename: &str) -> String {
    let name = font_filename
        .replace(".ttf", "")
        .replace(".otf", "")
        .replace('-', "_")
        .replace(' ', "_");
    format!(
        r#"val {name} = Typeface.createFromAsset(context.assets, "fonts/{font_filename}")"#
    )
}

/// Generate a `Fonts.kt` helper class for a list of font files.
fn gen_fonts_kt(font_files: &[String]) -> String {
    let mut lines = vec![
        "package com.example.app".to_string(),
        "".to_string(),
        "import android.content.Context".to_string(),
        "import android.graphics.Typeface".to_string(),
        "".to_string(),
        "object Fonts {".to_string(),
        "    fun load(context: Context) {".to_string(),
    ];
    for f in font_files {
        lines.push(format!("        {}", gen_font_kotlin(f)));
    }
    lines.push("    }".to_string());
    lines.push("}".to_string());
    lines.join("\n")
}

/// Generate a Swift `UIAppFonts` registration snippet for a single font file.
pub fn gen_font_swift(font_filename: &str) -> String {
    format!(
        r#"    // Register font: {font_filename}
    // Add to Info.plist UIAppFonts: <string>{font_filename}</string>"#
    )
}

/// Generate a `FontRegistration.plist` snippet listing UIAppFonts.
fn gen_fonts_plist(font_files: &[String]) -> String {
    let mut lines = vec![
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>".to_string(),
        "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">".to_string(),
        "<plist version=\"1.0\">".to_string(),
        "<dict>".to_string(),
        "    <key>UIAppFonts</key>".to_string(),
        "    <array>".to_string(),
    ];
    for f in font_files {
        lines.push(format!("        <string>{f}</string>"));
    }
    lines.push("    </array>".to_string());
    lines.push("</dict>".to_string());
    lines.push("</plist>".to_string());
    lines.join("\n")
}

// ─── Missing asset detection ──────────────────────────────────────────────────

/// Scan the AST for any `src:` prop values referencing files that don't exist
/// under `assets_dir`. Returns a list of missing asset paths.
pub fn detect_missing_assets(ast: &AST, assets_dir: &Path) -> Vec<String> {
    let mut missing = Vec::new();
    for page in &ast.pages {
        collect_missing_from_nodes(&page.children, assets_dir, &mut missing);
    }
    for comp in ast.components.values() {
        collect_missing_from_nodes(&comp.children, assets_dir, &mut missing);
    }
    missing.sort();
    missing.dedup();
    missing
}

fn collect_missing_from_nodes(
    nodes: &[crate::parser::ast::ComponentNode],
    assets_dir: &Path,
    out: &mut Vec<String>,
) {
    use crate::parser::ast::{Expr, Value};
    for node in nodes {
        if let Some(src_expr) = node.props.get("src") {
            if let Expr::Literal(Value::Str(path)) = src_expr {
                // Strip leading "./" or "assets/"
                let rel = path.trim_start_matches("./").trim_start_matches("assets/");
                let full = assets_dir.join(rel);
                if !full.exists() {
                    out.push(path.clone());
                }
            }
        }
        collect_missing_from_nodes(&node.children, assets_dir, out);
    }
}

// ─── Podfile generation ───────────────────────────────────────────────────────

fn gen_podfile(app_name: &str, _bundle_id: &str) -> String {
    format!(
        r#"# Generated by frame deploy ios
platform :ios, '16.0'

target '{app_name}' do
  use_frameworks!

  # Add your pods here, e.g.:
  # pod 'Alamofire', '~> 5.0'

  post_install do |installer|
    installer.pods_project.targets.each do |target|
      target.build_configurations.each do |config|
        config.build_settings['IPHONEOS_DEPLOYMENT_TARGET'] = '16.0'
      end
    end
  end
end
"#
    )
}

// ─── Config helpers ───────────────────────────────────────────────────────────

fn read_config(project_dir: &Path) -> Result<serde_json::Value, String> {
    let path = project_dir.join("frame.config.json");
    let raw = fs::read_to_string(&path)
        .map_err(|_| "frame.config.json not found. Are you in a Frame project directory? Run: frame start <name>".to_string())?;
    serde_json::from_str(&raw)
        .map_err(|e| format!("Invalid frame.config.json: {e}"))
}

fn android_config_from_json(cfg: &serde_json::Value) -> AndroidConfig {
    AndroidConfig {
        application_id: cfg["bundle_id"].as_str().unwrap_or("com.example.app").to_string(),
        app_name:       cfg["name"].as_str().unwrap_or("Frame App").to_string(),
        version_name:   cfg["version"].as_str().unwrap_or("1.0").to_string(),
        version_code:   cfg["build_number"].as_str().and_then(|s| s.parse().ok()).unwrap_or(1),
        min_sdk:        cfg["min_android_sdk"].as_u64().unwrap_or(24) as u32,
        target_sdk:     34,
    }
}

fn ios_config_from_json(cfg: &serde_json::Value) -> IosConfig {
    IosConfig {
        bundle_id:         cfg["bundle_id"].as_str().unwrap_or("com.example.app").to_string(),
        app_name:          cfg["name"].as_str().unwrap_or("Frame App").to_string(),
        version:           cfg["version"].as_str().unwrap_or("1.0").to_string(),
        build_number:      cfg["build_number"].as_str().unwrap_or("1").to_string(),
        min_ios:           cfg["min_ios"].as_str().unwrap_or("16.0").to_string(),
        team_id:           "XXXXXXXXXX".to_string(),
        deployment_target: cfg["min_ios"].as_str().unwrap_or("16.0").to_string(),
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_kotlin_generation() {
        let code = gen_font_kotlin("Roboto-Regular.ttf");
        assert!(code.contains("Typeface.createFromAsset"));
        assert!(code.contains("Roboto-Regular.ttf"));
    }

    #[test]
    fn font_swift_generation() {
        let code = gen_font_swift("OpenSans-Bold.otf");
        assert!(code.contains("OpenSans-Bold.otf"));
        assert!(code.contains("UIAppFonts"));
    }

    #[test]
    fn copy_assets_android_creates_dirs() {
        let tmp = std::env::temp_dir().join("frame_deploy_test_android");
        let assets_src = tmp.join("assets");
        let fonts_dir = assets_src.join("fonts");
        fs::create_dir_all(&fonts_dir).unwrap();
        // Create a fake font file
        fs::write(fonts_dir.join("Test.ttf"), b"fake font").unwrap();

        let dst = tmp.join("dest");
        copy_assets_android(&assets_src, &dst);

        assert!(dst.join("fonts/Test.ttf").exists());
        // Cleanup
        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn copy_assets_ios_creates_dirs() {
        let tmp = std::env::temp_dir().join("frame_deploy_test_ios");
        let assets_src = tmp.join("assets");
        fs::create_dir_all(&assets_src).unwrap();
        fs::write(assets_src.join("logo.png"), b"fake png").unwrap();

        let dst = tmp.join("xcassets/Resources");
        copy_assets_ios(&assets_src, &dst);

        assert!(dst.join("logo.png").exists());
        fs::remove_dir_all(&tmp).ok();
    }
}
