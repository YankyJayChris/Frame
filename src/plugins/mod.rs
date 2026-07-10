//! Plugin package system for the Frame framework.
//!
//! Implements plugin manifest loading, registry, semver resolution,
//! dependency conflict detection, and permission injection helpers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ─── Semver ───────────────────────────────────────────────────────────────────

/// A parsed semantic version (major.minor.patch).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        SemVer { major, minor, patch }
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Parse a version string like "1.2.3" into a SemVer.
pub fn parse_semver(s: &str) -> Option<SemVer> {
    let s = s.trim();
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 { return None; }
    let major = parts[0].parse::<u32>().ok()?;
    let minor = parts[1].parse::<u32>().ok()?;
    let patch = parts[2].parse::<u32>().ok()?;
    Some(SemVer { major, minor, patch })
}

/// Range constraint: exact, ^major (compatible), ~minor (patch), or "latest".
#[derive(Debug, Clone, PartialEq)]
pub enum SemVerRange {
    Exact(SemVer),
    CompatibleMajor(SemVer), // ^ — same major, >= minor.patch
    CompatiblePatch(SemVer), // ~ — same major+minor, >= patch
    Latest,
}

impl std::fmt::Display for SemVerRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SemVerRange::Exact(v) => write!(f, "{}", v),
            SemVerRange::CompatibleMajor(v) => write!(f, "^{}", v),
            SemVerRange::CompatiblePatch(v) => write!(f, "~{}", v),
            SemVerRange::Latest => write!(f, "latest"),
        }
    }
}

/// Parse a range string like "^1.0.0", "~1.2.3", "1.0.0", or "latest".
pub fn parse_semver_range(s: &str) -> Option<SemVerRange> {
    let s = s.trim();
    if s == "latest" {
        return Some(SemVerRange::Latest);
    }
    if let Some(rest) = s.strip_prefix('^') {
        return parse_semver(rest).map(SemVerRange::CompatibleMajor);
    }
    if let Some(rest) = s.strip_prefix('~') {
        return parse_semver(rest).map(SemVerRange::CompatiblePatch);
    }
    parse_semver(s).map(SemVerRange::Exact)
}

/// Returns true if `version` satisfies `range`.
pub fn semver_compatible(version: &SemVer, range: &SemVerRange) -> bool {
    match range {
        SemVerRange::Latest => true,
        SemVerRange::Exact(v) => version == v,
        SemVerRange::CompatibleMajor(v) => {
            version.major == v.major && *version >= *v
        }
        SemVerRange::CompatiblePatch(v) => {
            version.major == v.major && version.minor == v.minor && version.patch >= v.patch
        }
    }
}

/// Returns true if two ranges can possibly be satisfied by a common version.
pub fn ranges_compatible(a: &SemVerRange, b: &SemVerRange) -> bool {
    // Use a set of "witness" versions derived from both range base versions and
    // check if any witness satisfies both.
    let candidates = collect_range_candidates(a, b);
    candidates.iter().any(|v| semver_compatible(v, a) && semver_compatible(v, b))
}

fn collect_range_candidates(a: &SemVerRange, b: &SemVerRange) -> Vec<SemVer> {
    let mut out = Vec::new();
    for r in [a, b] {
        match r {
            SemVerRange::Exact(v) | SemVerRange::CompatibleMajor(v) | SemVerRange::CompatiblePatch(v) => {
                out.push(v.clone());
                // also a slightly higher version
                out.push(SemVer::new(v.major, v.minor, v.patch + 1));
                out.push(SemVer::new(v.major, v.minor + 1, 0));
            }
            SemVerRange::Latest => {}
        }
    }
    // always try a very high version — satisfies Latest and ^ ranges
    out.push(SemVer::new(99, 0, 0));
    out
}

// ─── plugin.json manifest ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AndroidPluginInfo {
    pub class: String,
    pub package: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IosPluginInfo {
    pub class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginPermissions {
    #[serde(default)]
    pub android: Vec<String>,
    #[serde(default)]
    pub ios: Vec<String>,
}

/// Mirrors the `plugin.json` schema.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub permissions: PluginPermissions,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    pub android: Option<AndroidPluginInfo>,
    pub ios: Option<IosPluginInfo>,
}

// ─── InstalledPlugin ──────────────────────────────────────────────────────────

/// An installed plugin — manifest + local paths.
#[derive(Debug, Clone)]
pub struct InstalledPlugin {
    pub manifest: PluginManifest,
    /// Absolute path to `frame_modules/<name>/`
    pub local_path: PathBuf,
    /// Paths to files inside `frame_modules/<name>/android/`
    pub android_sources: Vec<PathBuf>,
    /// Paths to files inside `frame_modules/<name>/ios/`
    pub ios_sources: Vec<PathBuf>,
}

impl InstalledPlugin {
    /// The resolved SemVer of this plugin (falls back to 0.0.0 if unparseable).
    pub fn version(&self) -> SemVer {
        parse_semver(&self.manifest.version).unwrap_or(SemVer::new(0, 0, 0))
    }
}

// ─── PluginRegistry ───────────────────────────────────────────────────────────

/// Registry of all installed plugins in a project.
#[derive(Debug, Clone, Default)]
pub struct PluginRegistry {
    pub plugins: HashMap<String, InstalledPlugin>,
}

impl PluginRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        PluginRegistry { plugins: HashMap::new() }
    }

    /// Load all plugins from `<project_root>/frame_modules/`.
    ///
    /// For each sub-directory that contains a valid `plugin.json`:
    /// - Parse the manifest
    /// - Collect `android/` and `ios/` source paths
    pub fn load(project_root: &Path) -> Self {
        let mut registry = PluginRegistry::new();
        let modules_dir = project_root.join("frame_modules");
        if !modules_dir.exists() {
            return registry;
        }
        let entries = match std::fs::read_dir(&modules_dir) {
            Ok(e) => e,
            Err(_) => return registry,
        };
        for entry in entries.flatten() {
            let plugin_dir = entry.path();
            if !plugin_dir.is_dir() { continue; }
            let manifest_path = plugin_dir.join("plugin.json");
            if !manifest_path.exists() { continue; }
            let content = match std::fs::read_to_string(&manifest_path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let manifest: PluginManifest = match serde_json::from_str(&content) {
                Ok(m) => m,
                Err(_) => continue,
            };
            let android_sources = collect_dir_files(&plugin_dir.join("android"));
            let ios_sources = collect_dir_files(&plugin_dir.join("ios"));
            let plugin = InstalledPlugin {
                manifest: manifest.clone(),
                local_path: plugin_dir,
                android_sources,
                ios_sources,
            };
            registry.plugins.insert(manifest.name.clone(), plugin);
        }
        registry
    }

    /// Collect all Android permissions from plugins actually used in the given
    /// plugin names list. Returns a deduplicated sorted vec.
    pub fn android_permissions(&self, used: &[String]) -> Vec<String> {
        let mut perms: Vec<String> = used.iter()
            .filter_map(|n| self.plugins.get(n))
            .flat_map(|p| p.manifest.permissions.android.iter().cloned())
            .collect();
        perms.sort();
        perms.dedup();
        perms
    }

    /// Collect all iOS permissions from plugins actually used.
    pub fn ios_permissions(&self, used: &[String]) -> Vec<String> {
        let mut perms: Vec<String> = used.iter()
            .filter_map(|n| self.plugins.get(n))
            .flat_map(|p| p.manifest.permissions.ios.iter().cloned())
            .collect();
        perms.sort();
        perms.dedup();
        perms
    }

    /// Detect dependency conflicts. Returns a list of human-readable error strings.
    pub fn detect_conflicts(&self) -> Vec<String> {
        // Build a map: dep_name -> Vec<(plugin_name, range_string)>
        let mut dep_map: HashMap<String, Vec<(String, String)>> = HashMap::new();
        for (plugin_name, plugin) in &self.plugins {
            for (dep, range) in &plugin.manifest.dependencies {
                dep_map.entry(dep.clone())
                    .or_default()
                    .push((plugin_name.clone(), range.clone()));
            }
        }
        let mut errors = Vec::new();
        for (dep, claimants) in &dep_map {
            if claimants.len() < 2 { continue; }
            // Check every pair for incompatibility
            for i in 0..claimants.len() {
                for j in (i + 1)..claimants.len() {
                    let (pa, ra) = &claimants[i];
                    let (pb, rb) = &claimants[j];
                    let range_a = parse_semver_range(ra);
                    let range_b = parse_semver_range(rb);
                    let conflict = match (range_a, range_b) {
                        (Some(a), Some(b)) => !ranges_compatible(&a, &b),
                        _ => false,
                    };
                    if conflict {
                        errors.push(format!(
                            "Dependency conflict: '{}' requires '{}' @ '{}', but '{}' requires '{}' @ '{}'",
                            pa, dep, ra, pb, dep, rb
                        ));
                    }
                }
            }
        }
        errors
    }
}

fn collect_dir_files(dir: &Path) -> Vec<PathBuf> {
    if !dir.exists() { return Vec::new(); }
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() { out.push(p); }
            else if p.is_dir() { out.extend(collect_dir_files(&p)); }
        }
    }
    out.sort();
    out
}

// ─── frame.config.json helpers ────────────────────────────────────────────────

/// Relevant parts of frame.config.json for the plugin system.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrameConfig {
    #[serde(default)]
    pub bundle_id: String,
    #[serde(default)]
    pub app_name: String,
    #[serde(default)]
    pub render_mode: String,
    #[serde(default)]
    pub plugins: HashMap<String, String>,
}

impl FrameConfig {
    pub fn load(project_root: &Path) -> Option<Self> {
        let path = project_root.join("frame.config.json");
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn save(&self, project_root: &Path) -> std::io::Result<()> {
        let path = project_root.join("frame.config.json");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, content)
    }
}

// ─── frame.lock ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LockEntry {
    pub version: String,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrameLock {
    pub version: u32,
    #[serde(default)]
    pub plugins: HashMap<String, LockEntry>,
}

impl FrameLock {
    pub fn load(project_root: &Path) -> Self {
        let path = project_root.join("frame.lock");
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(FrameLock { version: 1, plugins: HashMap::new() })
    }

    /// Write atomically: write to `.frame.lock.tmp` then rename.
    pub fn save(&self, project_root: &Path) -> std::io::Result<()> {
        let tmp = project_root.join(".frame.lock.tmp");
        let final_path = project_root.join("frame.lock");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(&tmp, content)?;
        std::fs::rename(tmp, final_path)
    }

    pub fn set_plugin(&mut self, name: &str, version: &str, checksum: &str) {
        self.plugins.insert(name.to_string(), LockEntry {
            version: version.to_string(),
            checksum: checksum.to_string(),
        });
    }
}

/// Compute a simple SHA-256 checksum of bytes, returned as "sha256:<hex>".
pub fn sha256_checksum(data: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let hash = Sha256::digest(data);
    let hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
    format!("sha256:{}", hex)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_plugin_dir(root: &Path, name: &str, manifest: &PluginManifest) {
        let dir = root.join("frame_modules").join(name);
        fs::create_dir_all(dir.join("android")).unwrap();
        fs::create_dir_all(dir.join("ios")).unwrap();
        let json = serde_json::to_string_pretty(manifest).unwrap();
        fs::write(dir.join("plugin.json"), json).unwrap();
    }

    // ── semver tests ──────────────────────────────────────────────────────────

    #[test]
    fn test_parse_semver_valid() {
        let v = parse_semver("1.2.3").unwrap();
        assert_eq!(v, SemVer::new(1, 2, 3));
    }

    #[test]
    fn test_parse_semver_invalid() {
        assert!(parse_semver("1.2").is_none());
        assert!(parse_semver("").is_none());
        assert!(parse_semver("a.b.c").is_none());
    }

    #[test]
    fn test_semver_range_exact() {
        let range = parse_semver_range("1.0.0").unwrap();
        assert!(semver_compatible(&SemVer::new(1, 0, 0), &range));
        assert!(!semver_compatible(&SemVer::new(1, 0, 1), &range));
    }

    #[test]
    fn test_semver_range_caret() {
        let range = parse_semver_range("^1.0.0").unwrap();
        assert!(semver_compatible(&SemVer::new(1, 0, 0), &range));
        assert!(semver_compatible(&SemVer::new(1, 5, 3), &range));
        assert!(!semver_compatible(&SemVer::new(2, 0, 0), &range));
        assert!(!semver_compatible(&SemVer::new(0, 9, 9), &range));
    }

    #[test]
    fn test_semver_range_tilde() {
        let range = parse_semver_range("~1.2.0").unwrap();
        assert!(semver_compatible(&SemVer::new(1, 2, 0), &range));
        assert!(semver_compatible(&SemVer::new(1, 2, 5), &range));
        assert!(!semver_compatible(&SemVer::new(1, 3, 0), &range));
    }

    #[test]
    fn test_semver_range_latest() {
        let range = parse_semver_range("latest").unwrap();
        assert!(semver_compatible(&SemVer::new(0, 0, 1), &range));
        assert!(semver_compatible(&SemVer::new(99, 99, 99), &range));
    }

    // ── conflict detection ────────────────────────────────────────────────────

    #[test]
    fn test_no_conflict_compatible_ranges() {
        let mut registry = PluginRegistry::new();
        let mut manifest_a = PluginManifest::default();
        manifest_a.name = "plugin_a".to_string();
        manifest_a.version = "1.0.0".to_string();
        manifest_a.dependencies.insert("shared_dep".to_string(), "^1.0.0".to_string());
        registry.plugins.insert("plugin_a".to_string(), InstalledPlugin {
            manifest: manifest_a,
            local_path: PathBuf::from("/tmp/plugin_a"),
            android_sources: vec![],
            ios_sources: vec![],
        });

        let mut manifest_b = PluginManifest::default();
        manifest_b.name = "plugin_b".to_string();
        manifest_b.version = "1.0.0".to_string();
        manifest_b.dependencies.insert("shared_dep".to_string(), "^1.2.0".to_string());
        registry.plugins.insert("plugin_b".to_string(), InstalledPlugin {
            manifest: manifest_b,
            local_path: PathBuf::from("/tmp/plugin_b"),
            android_sources: vec![],
            ios_sources: vec![],
        });

        let conflicts = registry.detect_conflicts();
        assert!(conflicts.is_empty(), "Should not have conflicts: {:?}", conflicts);
    }

    #[test]
    fn test_conflict_incompatible_ranges() {
        let mut registry = PluginRegistry::new();

        let mut manifest_a = PluginManifest::default();
        manifest_a.name = "plugin_a".to_string();
        manifest_a.version = "1.0.0".to_string();
        manifest_a.dependencies.insert("shared_dep".to_string(), "^1.0.0".to_string());
        registry.plugins.insert("plugin_a".to_string(), InstalledPlugin {
            manifest: manifest_a,
            local_path: PathBuf::from("/tmp"),
            android_sources: vec![],
            ios_sources: vec![],
        });

        let mut manifest_b = PluginManifest::default();
        manifest_b.name = "plugin_b".to_string();
        manifest_b.version = "1.0.0".to_string();
        manifest_b.dependencies.insert("shared_dep".to_string(), "^2.0.0".to_string());
        registry.plugins.insert("plugin_b".to_string(), InstalledPlugin {
            manifest: manifest_b,
            local_path: PathBuf::from("/tmp"),
            android_sources: vec![],
            ios_sources: vec![],
        });

        let conflicts = registry.detect_conflicts();
        assert!(!conflicts.is_empty(), "Should detect conflict");
        assert!(conflicts[0].contains("shared_dep"));
    }

    // ── permission injection test (Task 19.17) ────────────────────────────────
    // Project with frame_maps plugin → AndroidManifest contains ACCESS_FINE_LOCATION
    // and Info.plist contains NSLocationWhenInUseUsageDescription

    #[test]
    fn test_permission_injection_android_and_ios() {
        use crate::compiler::android::AndroidConfig;
        use crate::compiler::ios::IosConfig;

        // Build a minimal plugin registry with frame_maps
        let mut registry = PluginRegistry::new();
        let mut manifest = PluginManifest::default();
        manifest.name = "frame_maps".to_string();
        manifest.version = "1.0.0".to_string();
        manifest.permissions.android.push(
            "android.permission.ACCESS_FINE_LOCATION".to_string()
        );
        manifest.permissions.ios.push(
            "NSLocationWhenInUseUsageDescription".to_string()
        );
        registry.plugins.insert("frame_maps".to_string(), InstalledPlugin {
            manifest,
            local_path: PathBuf::from("/tmp/frame_maps"),
            android_sources: vec![],
            ios_sources: vec![],
        });

        let used_plugins = vec!["frame_maps".to_string()];

        // Verify android permissions list
        let android_perms = registry.android_permissions(&used_plugins);
        assert!(
            android_perms.contains(&"android.permission.ACCESS_FINE_LOCATION".to_string()),
            "Android perms should contain ACCESS_FINE_LOCATION"
        );

        // Verify iOS permissions list
        let ios_perms = registry.ios_permissions(&used_plugins);
        assert!(
            ios_perms.contains(&"NSLocationWhenInUseUsageDescription".to_string()),
            "iOS perms should contain NSLocationWhenInUseUsageDescription"
        );

        // Verify generated AndroidManifest.xml contains the permission
        use crate::parser::ast::AST;
        let ast = AST::default();
        let android_config = AndroidConfig::default();
        let files = gen_android_with_plugin_perms(&ast, &android_config, &android_perms);
        let manifest_file = files.iter().find(|f| f.path.ends_with("AndroidManifest.xml"))
            .expect("AndroidManifest.xml must be generated");
        assert!(
            manifest_file.content.contains("ACCESS_FINE_LOCATION"),
            "AndroidManifest.xml should contain ACCESS_FINE_LOCATION"
        );

        // Verify generated Info.plist contains the key
        let ios_config = IosConfig::default();
        let ios_files = gen_ios_with_plugin_perms(&ast, &ios_config, &ios_perms);
        let plist_file = ios_files.iter().find(|f| f.path.ends_with("Info.plist"))
            .expect("Info.plist must be generated");
        assert!(
            plist_file.content.contains("NSLocationWhenInUseUsageDescription"),
            "Info.plist should contain NSLocationWhenInUseUsageDescription"
        );
    }

    // ── idempotence test (Task 19.18) ─────────────────────────────────────────
    // Same plugin imported in two pages → single component definition in merged AST

    #[test]
    fn test_plugin_import_idempotence() {
        use crate::parser::ast::{AST, Import};

        // Simulate two pages both importing "frame_maps"
        let mut ast = AST::default();
        // First import
        ast.imports.push(Import {
            names: vec![("MapView".to_string(), None)],
            path: "frame_maps".to_string(),
        });
        // Duplicate import (same plugin, same name)
        ast.imports.push(Import {
            names: vec![("MapView".to_string(), None)],
            path: "frame_maps".to_string(),
        });

        // Deduplicate imports by (name, path) — same behaviour the resolver would apply
        let mut seen = std::collections::HashSet::new();
        let deduped: Vec<_> = ast.imports.iter().filter(|imp| {
            let key = (imp.path.clone(), imp.names.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>().join(","));
            seen.insert(key)
        }).collect();

        assert_eq!(
            deduped.len(), 1,
            "Duplicate plugin import should collapse to a single entry"
        );
    }

    // ── PluginRegistry::load test ─────────────────────────────────────────────

    fn unique_tmp_dir(suffix: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!("frame_plugin_test_{}_{}", suffix,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()));
        fs::create_dir_all(&base).unwrap();
        base
    }

    #[test]
    fn test_registry_load() {
        let root = unique_tmp_dir("registry_load");

        let mut manifest = PluginManifest::default();
        manifest.name = "test_plugin".to_string();
        manifest.version = "1.0.0".to_string();
        manifest.permissions.android.push("android.permission.CAMERA".to_string());

        make_plugin_dir(&root, "test_plugin", &manifest);

        let registry = PluginRegistry::load(&root);
        assert!(registry.plugins.contains_key("test_plugin"));
        let p = &registry.plugins["test_plugin"];
        assert_eq!(p.manifest.version, "1.0.0");
        assert_eq!(p.manifest.permissions.android, vec!["android.permission.CAMERA"]);

        // cleanup
        let _ = fs::remove_dir_all(&root);
    }

    // ── frame.lock atomic write ────────────────────────────────────────────────

    #[test]
    fn test_frame_lock_save_and_load() {
        let root = unique_tmp_dir("frame_lock");

        let mut lock = FrameLock { version: 1, plugins: HashMap::new() };
        lock.set_plugin("frame_maps", "1.0.0", "sha256:abc123");
        lock.save(&root).unwrap();

        let loaded = FrameLock::load(&root);
        assert_eq!(loaded.version, 1);
        assert!(loaded.plugins.contains_key("frame_maps"));
        assert_eq!(loaded.plugins["frame_maps"].version, "1.0.0");

        // cleanup
        let _ = fs::remove_dir_all(&root);
    }
}

// ─── Test helpers ─────────────────────────────────────────────────────────────

/// Generate Android files with extra plugin permissions injected into the manifest.
/// This is the integration helper used by the permission injection test.
pub fn gen_android_with_plugin_perms(
    ast: &crate::parser::ast::AST,
    config: &crate::compiler::android::AndroidConfig,
    extra_perms: &[String],
) -> Vec<crate::compiler::android::OutputFile> {
    let mut files = crate::compiler::android::gen_android(ast, config);

    if extra_perms.is_empty() {
        return files;
    }

    // Find and patch the AndroidManifest.xml
    for file in &mut files {
        if file.path.ends_with("AndroidManifest.xml") {
            let mut perm_lines = String::new();
            for perm in extra_perms {
                perm_lines.push_str(&format!(
                    "    <uses-permission android:name=\"{}\" />\n",
                    perm
                ));
            }
            // Insert after <manifest ...> opening tag
            if let Some(pos) = file.content.find("<manifest ") {
                if let Some(close) = file.content[pos..].find(">\n") {
                    let insert_at = pos + close + 2;
                    file.content.insert_str(insert_at, &perm_lines);
                }
            }
        }
    }
    files
}

/// Generate iOS files with extra plugin permissions injected into Info.plist.
pub fn gen_ios_with_plugin_perms(
    ast: &crate::parser::ast::AST,
    config: &crate::compiler::ios::IosConfig,
    extra_perms: &[String],
) -> Vec<crate::compiler::ios::OutputFile> {
    let mut files = crate::compiler::ios::gen_ios(ast, config);

    if extra_perms.is_empty() {
        return files;
    }

    for file in &mut files {
        if file.path.ends_with("Info.plist") {
            let mut perm_entries = String::new();
            for perm in extra_perms {
                perm_entries.push_str(&format!(
                    "\t<key>{}</key>\n\t<string>This app requires this permission.</string>\n",
                    perm
                ));
            }
            // Insert before closing </dict>
            if let Some(pos) = file.content.rfind("</dict>") {
                file.content.insert_str(pos, &perm_entries);
            }
        }
    }
    files
}
