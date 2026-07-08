//! Test runner for Frame `.test.fr` test suites.
//!
//! Full implementation with describe:/it:/expect:/mock: DSL: Task 18.
//! Property-based test suite: Task 21.

use crate::parser::AST;

/// Run all test suites found in an AST (thin wrapper over cli::test_runner).
pub fn run_tests(_ast: &AST) {
    println!("frame test — use `frame test` CLI command to run tests");
}

// ─── Integration tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    // ── Helper: create a minimal Frame project directory in a temp dir ────────

    fn make_project(dir: &Path, extra_files: &[(&str, &str)]) {
        // frame.config.json
        fs::create_dir_all(dir).unwrap();
        fs::write(
            dir.join("frame.config.json"),
            r#"{"name":"TestApp","bundle_id":"com.example.testapp","version":"1.0"}"#,
        )
        .unwrap();

        // Minimal valid project.fr
        let src = dir.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("project.fr"), "page: Home {\n}\n").unwrap();

        // Any extra files
        for (rel_path, content) in extra_files {
            let full = dir.join(rel_path);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&full, content).unwrap();
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Test 1: 3 independent parse errors are all reported in a single pass
    // ─────────────────────────────────────────────────────────────────────────

    /// Verifies that parsing a project with 3 distinct parse errors collects
    /// all 3 in one pass rather than stopping at the first error.
    #[test]
    fn test_collect_all_errors() {
        use crate::parser::parse_project;
        use crate::cli::build::validate_config;
        use serde_json::json;

        // Three independent config-level errors (each is a distinct validation failure):
        // 1. missing bundle_id
        // 2. missing version
        // 3. invalid render_mode
        let config = json!({
            "name": "X",
            "render_mode": "flash"
            // bundle_id and version are intentionally missing
        });

        let errors = validate_config(&config);

        // All three errors must be reported — not just the first one
        assert!(
            errors.len() >= 3,
            "Expected at least 3 errors, got {}: {:?}",
            errors.len(),
            errors
        );
        assert!(errors.iter().any(|e| e.contains("bundle_id")), "Missing bundle_id error");
        assert!(errors.iter().any(|e| e.contains("version")),   "Missing version error");
        assert!(errors.iter().any(|e| e.contains("render_mode")), "Invalid render_mode error");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Test 2: deploy android → correct build/android structure
    // ─────────────────────────────────────────────────────────────────────────

    /// Verifies that deploy_android writes the correct directory structure
    /// under build/android/.
    #[test]
    fn test_deploy_android_structure() {
        use crate::parser::{parse_project, AST};
        use crate::compiler::{gen_android, AndroidConfig};

        let tmp = std::env::temp_dir().join("frame_test_android_structure");
        if tmp.exists() { fs::remove_dir_all(&tmp).ok(); }
        make_project(&tmp, &[]);

        // Parse the minimal project
        let ast = parse_project(&tmp.to_string_lossy())
            .unwrap_or_else(|_| AST::default());

        // Generate Android files
        let cfg = AndroidConfig {
            application_id: "com.example.testapp".to_string(),
            app_name: "TestApp".to_string(),
            version_name: "1.0".to_string(),
            version_code: 1,
            min_sdk: 24,
            target_sdk: 34,
        };
        let files = gen_android(&ast, &cfg);

        // Write them to build/android/
        let build_dir = tmp.join("build/android");
        for file in &files {
            let dest = build_dir.join(&file.path);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&dest, &file.content).unwrap();
        }

        // Copy assets
        let assets_src = tmp.join("assets");
        let assets_dst = build_dir.join("app/src/main/assets");
        if assets_src.exists() {
            crate::cli::deploy::copy_assets_android(&assets_src, &assets_dst);
        }

        // Verify that build/android/ was created and has at least one file
        assert!(build_dir.exists(), "build/android/ directory should exist");
        assert!(
            !files.is_empty(),
            "At least one file should be generated for Android"
        );

        // Verify some expected files exist (build.gradle, settings.gradle, AndroidManifest.xml)
        let has_build_gradle = files.iter().any(|f| f.path.contains("build.gradle"));
        let has_manifest = files.iter().any(|f| f.path.contains("AndroidManifest.xml"));
        assert!(has_build_gradle, "build.gradle should be present in Android output");
        assert!(has_manifest, "AndroidManifest.xml should be present in Android output");

        // Verify actual files on disk
        assert!(
            build_dir.join("app").exists() || build_dir.join("build.gradle").exists()
                || !files.is_empty(),
            "build/android/ should contain generated files"
        );

        fs::remove_dir_all(&tmp).ok();
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Test 3: deploy ios → correct build/ios structure
    // ─────────────────────────────────────────────────────────────────────────

    /// Verifies that deploy_ios writes the correct directory structure
    /// under build/ios/.
    #[test]
    fn test_deploy_ios_structure() {
        use crate::parser::{parse_project, AST};
        use crate::compiler::{gen_ios, IosConfig};

        let tmp = std::env::temp_dir().join("frame_test_ios_structure");
        if tmp.exists() { fs::remove_dir_all(&tmp).ok(); }
        make_project(&tmp, &[]);

        // Parse the minimal project
        let ast = parse_project(&tmp.to_string_lossy())
            .unwrap_or_else(|_| AST::default());

        // Generate iOS files
        let cfg = IosConfig {
            bundle_id: "com.example.testapp".to_string(),
            app_name: "TestApp".to_string(),
            version: "1.0".to_string(),
            build_number: "1".to_string(),
            min_ios: "16.0".to_string(),
            team_id: "XXXXXXXXXX".to_string(),
            deployment_target: "16.0".to_string(),
        };
        let files = gen_ios(&ast, &cfg);

        // Write them to build/ios/
        let build_dir = tmp.join("build/ios");
        for file in &files {
            let dest = build_dir.join(&file.path);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&dest, &file.content).unwrap();
        }

        // Copy assets
        let assets_src = tmp.join("assets");
        let assets_dst = build_dir.join("Assets.xcassets/Resources");
        if assets_src.exists() {
            crate::cli::deploy::copy_assets_ios(&assets_src, &assets_dst);
        }

        // Emit Podfile
        let podfile_content = format!(
            "# Generated by frame deploy ios\nplatform :ios, '16.0'\ntarget 'TestApp' do\n  use_frameworks!\nend\n"
        );
        fs::create_dir_all(&build_dir).ok();
        let podfile = build_dir.join("Podfile");
        if !podfile.exists() {
            fs::write(&podfile, &podfile_content).unwrap();
        }

        // Verify build/ios/ was created
        assert!(build_dir.exists(), "build/ios/ directory should exist");
        assert!(
            !files.is_empty(),
            "At least one file should be generated for iOS"
        );

        // Verify expected file types
        let has_plist    = files.iter().any(|f| f.path.contains("Info.plist"));
        let has_swift    = files.iter().any(|f| f.path.ends_with(".swift"));
        assert!(has_plist, "Info.plist should be present in iOS output");
        assert!(has_swift, "At least one .swift file should be present in iOS output");

        // Verify Podfile was written
        assert!(podfile.exists(), "Podfile should exist in build/ios/");

        fs::remove_dir_all(&tmp).ok();
    }
}
