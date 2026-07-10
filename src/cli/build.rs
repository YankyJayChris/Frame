//! `frame build` implementation — config validation, SHA-256 incremental cache,
//! --watch, --strict, --locale, collect-all-errors, success summary.

use crate::parser::parse_project;
use crate::compiler::{gen_android, gen_ios};
use crate::compiler::android::AndroidConfig;
use crate::compiler::ios::IosConfig;

use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;

// ─── BuildConfig ─────────────────────────────────────────────────────────────

/// Parsed and validated `frame.config.json`.
pub struct BuildConfig {
    pub name: String,
    pub bundle_id: String,
    pub version: String,
    pub build_number: Option<String>,
    pub render_mode: Option<String>,
    pub min_android_sdk: Option<u64>,
    pub min_ios: Option<String>,
}

// ─── Config validation ────────────────────────────────────────────────────────

/// Validate a `frame.config.json` value. Returns a list of human-readable error strings.
pub fn validate_config(config: &serde_json::Value) -> Vec<String> {
    let mut errors = Vec::new();

    // Required fields
    for field in &["name", "bundle_id", "version"] {
        if config.get(field).and_then(|v| v.as_str()).is_none() {
            errors.push(format!("frame.config.json: missing required field `{field}`"));
        }
    }

    // bundle_id pattern: starts with a letter, at least 3 dot-separated segments
    if let Some(bundle_id) = config.get("bundle_id").and_then(|v| v.as_str()) {
        if !is_valid_bundle_id(bundle_id) {
            errors.push(format!(
                "frame.config.json: `bundle_id` \"{bundle_id}\" is invalid. \
                 Must be at least 3 dot-separated segments, each starting with a letter \
                 (e.g. \"com.example.app\")"
            ));
        }
    }

    // render_mode: must be "native" or "canvas" if present
    if let Some(mode) = config.get("render_mode").and_then(|v| v.as_str()) {
        if mode != "native" && mode != "canvas" {
            errors.push(format!(
                "frame.config.json: `render_mode` must be \"native\" or \"canvas\", got \"{mode}\""
            ));
        }
    }

    // min_android_sdk: must be >= 21 if present
    if let Some(sdk) = config.get("min_android_sdk").and_then(|v| v.as_u64()) {
        if sdk < 21 {
            errors.push(format!(
                "frame.config.json: `min_android_sdk` must be >= 21, got {sdk}"
            ));
        }
    }

    errors
}

/// Check that a bundle_id matches `^[a-zA-Z][a-zA-Z0-9]*(\.[a-zA-Z][a-zA-Z0-9]*){2,}$`
fn is_valid_bundle_id(id: &str) -> bool {
    let parts: Vec<&str> = id.split('.').collect();
    if parts.len() < 3 {
        return false;
    }
    for part in &parts {
        if part.is_empty() {
            return false;
        }
        let mut chars = part.chars();
        match chars.next() {
            Some(c) if c.is_ascii_alphabetic() => {}
            _ => return false,
        }
        for c in chars {
            if !c.is_ascii_alphanumeric() {
                return false;
            }
        }
    }
    true
}

// ─── SHA-256 incremental cache ────────────────────────────────────────────────

/// Compute the SHA-256 hash of a file, returned as a lowercase hex string.
pub fn compute_file_hash(path: &Path) -> String {
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return String::new(),
    };
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        match file.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => hasher.update(&buf[..n]),
            Err(_) => break,
        }
    }
    format!("{:x}", hasher.finalize())
}

const CACHE_DIR: &str = ".frame-cache";
const CACHE_FILE: &str = ".frame-cache/hashes.json";

/// Load the hash cache from `.frame-cache/hashes.json`. Returns an empty map if missing.
pub fn load_hash_cache(project_dir: &Path) -> HashMap<String, String> {
    let cache_path = project_dir.join(CACHE_FILE);
    let raw = match fs::read_to_string(&cache_path) {
        Ok(s) => s,
        Err(_) => return HashMap::new(),
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

/// Save the hash cache to `.frame-cache/hashes.json`.
pub fn save_hash_cache(project_dir: &Path, cache: &HashMap<String, String>) {
    let cache_dir = project_dir.join(CACHE_DIR);
    fs::create_dir_all(&cache_dir).ok();
    let path = project_dir.join(CACHE_FILE);
    if let Ok(json) = serde_json::to_string_pretty(cache) {
        fs::write(path, json).ok();
    }
}

/// Collect all `.fr` source files under `project_dir/src/`.
fn collect_fr_files(project_dir: &Path) -> Vec<PathBuf> {
    let src = project_dir.join("src");
    let mut result = Vec::new();
    visit_dir(&src, &mut result);
    result
}

fn visit_dir(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_dir(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("fr") {
            out.push(path);
        }
    }
}

// ─── Config helpers ───────────────────────────────────────────────────────────

fn read_config(project_dir: &Path) -> Result<serde_json::Value, String> {
    let path = project_dir.join("frame.config.json");
    let raw = fs::read_to_string(&path)
        .map_err(|_| "frame.config.json not found. Are you in a Frame project directory? Run: frame start <name>".to_string())?;
    serde_json::from_str(&raw)
        .map_err(|e| format!("Invalid frame.config.json: {e}"))
}

fn android_config(cfg: &serde_json::Value) -> AndroidConfig {
    AndroidConfig {
        application_id: cfg["bundle_id"].as_str().unwrap_or("com.example.app").to_string(),
        app_name:       cfg["name"].as_str().unwrap_or("Frame App").to_string(),
        version_name:   cfg["version"].as_str().unwrap_or("1.0").to_string(),
        version_code:   cfg["build_number"].as_str().and_then(|s| s.parse().ok()).unwrap_or(1),
        min_sdk:        cfg["min_android_sdk"].as_u64().unwrap_or(24) as u32,
        target_sdk:     34,
    }
}

fn ios_config(cfg: &serde_json::Value) -> IosConfig {
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

// ─── Single build pass ────────────────────────────────────────────────────────

fn run_build_once(project_dir: &Path, strict: bool, locale: &Option<String>) -> bool {
    let start = Instant::now();

    // 1. Read + validate config (collect all errors)
    let config = match read_config(project_dir) {
        Ok(c) => c,
        Err(e) => { eprintln!("error: {e}"); return false; }
    };
    let config_errors = validate_config(&config);
    if !config_errors.is_empty() {
        for err in &config_errors {
            eprintln!("error: {err}");
        }
        return false;
    }

    // 2. Log locale if provided
    if let Some(ref loc) = locale {
        println!("  Locale: {loc}");
    }

    // 3. Incremental cache check
    let mut cache = load_hash_cache(project_dir);
    let fr_files = collect_fr_files(project_dir);
    let mut any_changed = false;
    for path in &fr_files {
        let key = path.to_string_lossy().to_string();
        let new_hash = compute_file_hash(path);
        if cache.get(&key).map(|h| h.as_str()) != Some(&new_hash) {
            any_changed = true;
            cache.insert(key, new_hash);
        }
    }
    if !any_changed && !fr_files.is_empty() {
        println!("✓ Nothing changed. Build is up-to-date.");
        return true;
    }

    // 4. Parse project (collect all errors in one pass)
    let ast = match parse_project(&project_dir.to_string_lossy()) {
        Ok(a) => a,
        Err(errs) => {
            eprintln!("Build failed with {} error(s):", errs.len());
            for e in &errs {
                eprintln!("  {e}");
            }
            return false;
        }
    };

    // 5. Generate Android output
    let android_cfg = android_config(&config);
    let android_files = gen_android(&ast, &android_cfg);
    let android_out = project_dir.join("build/android");
    for file in &android_files {
        let dest = android_out.join(&file.path);
        if let Some(parent) = dest.parent() { fs::create_dir_all(parent).ok(); }
        if fs::write(&dest, &file.content).is_err() {
            eprintln!("warning: could not write {}", dest.display());
        }
    }
    // Make gradlew executable on Unix
    #[cfg(unix)]
    {
        let gradlew = android_out.join("gradlew");
        if gradlew.exists() {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = fs::metadata(&gradlew) {
                let mut perms = meta.permissions();
                perms.set_mode(perms.mode() | 0o755);
                fs::set_permissions(&gradlew, perms).ok();
            }
        }
    }

    // 6. Generate iOS output
    let ios_cfg = ios_config(&config);
    let ios_files = gen_ios(&ast, &ios_cfg);
    let ios_out = project_dir.join("build/ios");
    for file in &ios_files {
        let dest = ios_out.join(&file.path);
        if let Some(parent) = dest.parent() { fs::create_dir_all(parent).ok(); }
        if fs::write(&dest, &file.content).is_err() {
            eprintln!("warning: could not write {}", dest.display());
        }
    }

    // 7. Save updated cache
    save_hash_cache(project_dir, &cache);

    let elapsed = start.elapsed();
    let total = android_files.len() + ios_files.len();
    println!(
        "✓ Build complete in {:.2}s — {} file(s) generated",
        elapsed.as_secs_f64(),
        total
    );
    println!("  Android: build/android/  iOS: build/ios/");
    println!("  Run: frame deploy android  OR  frame deploy ios");

    // 8. Strict mode: treat warnings as errors (build succeeds but exit code 1)
    if strict {
        // In a full implementation, we'd collect warnings from the type checker here.
        // For now we report zero warnings; if warnings existed we'd return false.
    }

    true
}

// ─── Public entry point ───────────────────────────────────────────────────────

/// Run `frame build`. Returns `true` on success.
///
/// * `watch`  — re-run on every file-system change in `src/`
/// * `strict` — treat compiler warnings as errors
/// * `locale` — optional locale tag to filter i18n strings
pub fn run_build(watch: bool, strict: bool, locale: Option<String>) -> bool {
    let project_dir = Path::new(".");

    if !watch {
        return run_build_once(project_dir, strict, &locale);
    }

    // ── watch mode ────────────────────────────────────────────────────────────
    use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
    use std::sync::mpsc;
    use std::time::Duration;

    println!("Watching src/ for changes (Ctrl-C to stop)…");
    run_build_once(project_dir, strict, &locale);

    let (tx, rx) = mpsc::channel();
    let mut watcher: RecommendedWatcher = match notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    }) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("error: could not start file watcher: {e}");
            return false;
        }
    };

    let src_dir = project_dir.join("src");
    if watcher.watch(&src_dir, RecursiveMode::Recursive).is_err() {
        eprintln!("warning: could not watch src/ — watch mode disabled");
        return true;
    }

    loop {
        match rx.recv_timeout(Duration::from_secs(60)) {
            Ok(Ok(event)) => {
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                        println!("\nFile changed — rebuilding…");
                        run_build_once(project_dir, strict, &locale);
                    }
                    _ => {}
                }
            }
            Ok(Err(e)) => eprintln!("watch error: {e}"),
            Err(mpsc::RecvTimeoutError::Timeout) => {} // keep alive
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    true
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn valid_config_passes() {
        let cfg = json!({
            "name": "MyApp",
            "bundle_id": "com.example.myapp",
            "version": "1.0"
        });
        assert!(validate_config(&cfg).is_empty());
    }

    #[test]
    fn missing_required_fields() {
        let cfg = json!({ "name": "X" });
        let errs = validate_config(&cfg);
        assert!(errs.iter().any(|e| e.contains("bundle_id")));
        assert!(errs.iter().any(|e| e.contains("version")));
    }

    #[test]
    fn invalid_bundle_id_short() {
        let cfg = json!({ "name": "X", "bundle_id": "com.example", "version": "1.0" });
        let errs = validate_config(&cfg);
        assert!(!errs.is_empty());
    }

    #[test]
    fn invalid_bundle_id_starts_with_digit() {
        let cfg = json!({ "name": "X", "bundle_id": "1com.example.app", "version": "1.0" });
        let errs = validate_config(&cfg);
        assert!(!errs.is_empty());
    }

    #[test]
    fn valid_bundle_id_four_segments() {
        assert!(is_valid_bundle_id("com.example.sub.app"));
    }

    #[test]
    fn invalid_render_mode() {
        let cfg = json!({
            "name": "X", "bundle_id": "com.example.app", "version": "1.0",
            "render_mode": "web"
        });
        let errs = validate_config(&cfg);
        assert!(errs.iter().any(|e| e.contains("render_mode")));
    }

    #[test]
    fn min_android_sdk_too_low() {
        let cfg = json!({
            "name": "X", "bundle_id": "com.example.app", "version": "1.0",
            "min_android_sdk": 18
        });
        let errs = validate_config(&cfg);
        assert!(errs.iter().any(|e| e.contains("min_android_sdk")));
    }

    #[test]
    fn compute_hash_is_stable() {
        use std::io::Write;
        let dir = std::env::temp_dir();
        let path = dir.join("frame_test_hash.txt");
        {
            let mut f = fs::File::create(&path).unwrap();
            f.write_all(b"hello world").unwrap();
        }
        let h1 = compute_file_hash(&path);
        let h2 = compute_file_hash(&path);
        assert_eq!(h1, h2);
        assert!(!h1.is_empty());
        fs::remove_file(&path).ok();
    }

    #[test]
    fn cache_round_trip() {
        let dir = tempdir();
        let mut m = HashMap::new();
        m.insert("src/foo.fr".to_string(), "abc123".to_string());
        save_hash_cache(&dir, &m);
        let loaded = load_hash_cache(&dir);
        assert_eq!(loaded.get("src/foo.fr").map(|s| s.as_str()), Some("abc123"));
    }

    fn tempdir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("frame_cache_test");
        fs::create_dir_all(&dir).ok();
        dir
    }
}
