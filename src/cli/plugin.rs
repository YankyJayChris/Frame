//! Plugin CLI command implementations for the Frame framework.
//!
//! Provides: plugin_add, plugin_remove, plugin_install, plugin_list,
//!           plugin_create, plugin_publish

use crate::plugins::{
    FrameConfig, FrameLock, PluginManifest, PluginRegistry,
    AndroidPluginInfo, IosPluginInfo, PluginPermissions,
    parse_semver, parse_semver_range, semver_compatible, sha256_checksum,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ─── frame plugin add <name> ──────────────────────────────────────────────────

/// Install a plugin into `frame_modules/<name>/`.
///
/// Supports:
/// - `@user/repo` — clones from `https://github.com/user/repo.git` (Phase 6b)
/// - `plugin-name` — local stub if not found in registry
pub fn plugin_add(name: &str, project_root: &Path) -> bool {
    let modules_dir = project_root.join("frame_modules");
    let plugin_dir = modules_dir.join(name);

    if plugin_dir.exists() {
        println!("Plugin '{}' is already installed at {:?}", name, plugin_dir);
        return true;
    }

    // Phase 6b: Handle @user/repo format — git clone
    if name.starts_with('@') {
        return install_github_plugin(name, &modules_dir, project_root);
    }

    // Try to download from registry
    let version = try_fetch_plugin_version(name).unwrap_or_else(|| "1.0.0".to_string());

    // Scaffold the directory structure
    if let Err(e) = scaffold_plugin_dir(&plugin_dir, name, &version) {
        eprintln!("Error creating plugin directory: {}", e);
        return false;
    }

    println!("Installed plugin '{}' v{} → frame_modules/{}/", name, version, name);

    // Update frame.config.json
    let mut config = FrameConfig::load(project_root).unwrap_or_default();
    config.plugins.entry(name.to_string())
        .or_insert_with(|| format!("^{}", version));
    if let Err(e) = config.save(project_root) {
        eprintln!("Warning: could not update frame.config.json: {}", e);
    }

    // Write frame.lock
    let mut lock = FrameLock::load(project_root);
    let checksum = compute_plugin_checksum(&plugin_dir);
    lock.set_plugin(name, &version, &checksum);
    if let Err(e) = lock.save(project_root) {
        eprintln!("Warning: could not write frame.lock: {}", e);
    }

    true
}

/// Attempt to look up the latest version from the Frame Plugin Registry.
/// Returns None if the registry is unreachable.
fn try_fetch_plugin_version(_name: &str) -> Option<String> {
    // No live registry available in this environment; return None gracefully.
    // In production this would perform an HTTP GET to the registry API.
    None
}

fn scaffold_plugin_dir(dir: &Path, name: &str, version: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(dir.join("src"))?;
    std::fs::create_dir_all(dir.join("android"))?;
    std::fs::create_dir_all(dir.join("ios"))?;

    // plugin.json
    let manifest = PluginManifest {
        name: name.to_string(),
        version: version.to_string(),
        description: format!("{} plugin for Frame", name),
        permissions: PluginPermissions::default(),
        dependencies: HashMap::new(),
        android: Some(AndroidPluginInfo {
            class: format!("{}Plugin", pascal_case(name)),
            package: format!("com.frame.{}", name.replace('_', "").to_lowercase()),
        }),
        ios: Some(IosPluginInfo {
            class: format!("{}Plugin", pascal_case(name)),
        }),
    };
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(dir.join("plugin.json"), json)?;

    // src/index.fr
    std::fs::write(dir.join("src").join("index.fr"), format!(
        "// {} plugin entry point\n// Define exported components here.\n", name
    ))?;

    // android/Plugin.kt stub
    let class = format!("{}Plugin", pascal_case(name));
    let pkg = format!("com.frame.{}", name.replace('_', "").to_lowercase());
    std::fs::write(dir.join("android").join("Plugin.kt"), format!(
        "package {pkg}\n\nclass {class} {{\n    fun init() {{ }}\n}}\n"
    ))?;

    // ios/Plugin.swift stub
    std::fs::write(dir.join("ios").join("Plugin.swift"), format!(
        "import Foundation\n\nclass {class} {{\n    func setup() {{ }}\n}}\n"
    ))?;

    Ok(())
}

fn compute_plugin_checksum(plugin_dir: &Path) -> String {
    // Hash all files in the plugin directory for reproducibility
    let mut hasher_input: Vec<u8> = Vec::new();
    collect_file_bytes(plugin_dir, &mut hasher_input);
    if hasher_input.is_empty() {
        "sha256:empty".to_string()
    } else {
        sha256_checksum(&hasher_input)
    }
}

fn collect_file_bytes(dir: &Path, out: &mut Vec<u8>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        paths.sort();
        for path in paths {
            if path.is_file() {
                if let Ok(bytes) = std::fs::read(&path) {
                    out.extend_from_slice(&bytes);
                }
            } else if path.is_dir() {
                collect_file_bytes(&path, out);
            }
        }
    }
}

// ─── frame plugin remove <name> ───────────────────────────────────────────────

pub fn plugin_remove(name: &str, project_root: &Path) -> bool {
    let plugin_dir = project_root.join("frame_modules").join(name);
    if !plugin_dir.exists() {
        eprintln!("Plugin '{}' is not installed.", name);
        return false;
    }

    if let Err(e) = std::fs::remove_dir_all(&plugin_dir) {
        eprintln!("Error removing plugin directory: {}", e);
        return false;
    }

    // Update frame.config.json
    let mut config = FrameConfig::load(project_root).unwrap_or_default();
    config.plugins.remove(name);
    if let Err(e) = config.save(project_root) {
        eprintln!("Warning: could not update frame.config.json: {}", e);
    }

    // Update frame.lock
    let mut lock = FrameLock::load(project_root);
    lock.plugins.remove(name);
    if let Err(e) = lock.save(project_root) {
        eprintln!("Warning: could not write frame.lock: {}", e);
    }

    println!("Removed plugin '{}'.", name);
    true
}

// ─── frame plugin install (alias: frame install) ──────────────────────────────

pub fn plugin_install(project_root: &Path) -> bool {
    let config = match FrameConfig::load(project_root) {
        Some(c) => c,
        None => {
            eprintln!("No frame.config.json found in {:?}.", project_root);
            return false;
        }
    };

    if config.plugins.is_empty() {
        println!("No plugins declared in frame.config.json.");
        return true;
    }

    let mut all_ok = true;
    for (name, range_str) in &config.plugins {
        let plugin_dir = project_root.join("frame_modules").join(name);
        if plugin_dir.exists() {
            // Check if installed version satisfies declared range
            let registry = PluginRegistry::load(project_root);
            if let Some(installed) = registry.plugins.get(name) {
                if let Some(range) = parse_semver_range(range_str) {
                    let ver = installed.version();
                    if semver_compatible(&ver, &range) {
                        println!("  ✓ {} ({}) — already satisfied", name, ver);
                        continue;
                    } else {
                        println!("  ↻ {} — upgrading (installed: {}, required: {})", name, ver, range_str);
                        let _ = std::fs::remove_dir_all(&plugin_dir);
                    }
                }
            }
        }
        println!("  + Installing {}…", name);
        if !plugin_add(name, project_root) {
            eprintln!("  ✗ Failed to install '{}'.", name);
            all_ok = false;
        }
    }

    // Check for dependency conflicts after install
    let registry = PluginRegistry::load(project_root);
    let conflicts = registry.detect_conflicts();
    for conflict in &conflicts {
        eprintln!("⚠ {}", conflict);
        all_ok = false;
    }

    if all_ok {
        println!("All plugins installed successfully.");
    }
    all_ok
}

// ─── frame plugin list ────────────────────────────────────────────────────────

pub fn plugin_list(project_root: &Path) -> bool {
    let config = FrameConfig::load(project_root).unwrap_or_default();
    let registry = PluginRegistry::load(project_root);

    if config.plugins.is_empty() && registry.plugins.is_empty() {
        println!("No plugins installed.");
        return true;
    }

    println!("{:<20} {:<12} {:<12} {}", "Name", "Installed", "Required", "Status");
    println!("{}", "-".repeat(60));

    // Merge keys from config and registry
    let mut names: Vec<String> = config.plugins.keys().cloned()
        .chain(registry.plugins.keys().cloned())
        .collect();
    names.sort();
    names.dedup();

    for name in &names {
        let installed_ver = registry.plugins.get(name.as_str())
            .map(|p| p.manifest.version.clone())
            .unwrap_or_else(|| "—".to_string());
        let required_range = config.plugins.get(name.as_str())
            .cloned()
            .unwrap_or_else(|| "—".to_string());
        let status = if registry.plugins.contains_key(name.as_str()) {
            "✓ installed"
        } else {
            "✗ missing"
        };
        println!("{:<20} {:<12} {:<12} {}", name, installed_ver, required_range, status);
    }
    true
}

// ─── frame plugin create <name> ───────────────────────────────────────────────

pub fn plugin_create(name: &str, project_root: &Path) -> bool {
    let plugin_dir = project_root.join("frame_modules").join(name);
    if plugin_dir.exists() {
        eprintln!("Plugin '{}' already exists at {:?}.", name, plugin_dir);
        return false;
    }

    if let Err(e) = scaffold_plugin_dir(&plugin_dir, name, "0.1.0") {
        eprintln!("Error creating plugin scaffold: {}", e);
        return false;
    }

    // Write README.md
    let readme = format!(
        "# {name}\n\nA Frame plugin.\n\n## Installation\n\n```\nframe plugin add {name}\n```\n\n## Usage\n\n```fr\nimport {{ {class} }} \"{name}\"\n```\n",
        name = name,
        class = pascal_case(name),
    );
    if let Err(e) = std::fs::write(plugin_dir.join("README.md"), readme) {
        eprintln!("Warning: could not write README.md: {}", e);
    }

    println!("Plugin '{}' scaffolded at frame_modules/{}/", name, name);
    println!("  frame_modules/{}/plugin.json", name);
    println!("  frame_modules/{}/src/index.fr", name);
    println!("  frame_modules/{}/android/Plugin.kt", name);
    println!("  frame_modules/{}/ios/Plugin.swift", name);
    println!("  frame_modules/{}/README.md", name);
    true
}

// ─── frame plugin publish ─────────────────────────────────────────────────────

pub fn plugin_publish(project_root: &Path) -> bool {
    // Validate plugin.json in the current directory (or project_root)
    let manifest_path = project_root.join("plugin.json");
    if !manifest_path.exists() {
        eprintln!(
            "No plugin.json found. Run `frame plugin create <name>` first, \
             or run this command from your plugin directory."
        );
        return false;
    }

    let content = match std::fs::read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(e) => { eprintln!("Could not read plugin.json: {}", e); return false; }
    };
    let manifest: PluginManifest = match serde_json::from_str(&content) {
        Ok(m) => m,
        Err(e) => { eprintln!("Invalid plugin.json: {}", e); return false; }
    };

    if manifest.name.is_empty() {
        eprintln!("plugin.json: 'name' is required.");
        return false;
    }
    if manifest.version.is_empty() {
        eprintln!("plugin.json: 'version' is required.");
        return false;
    }
    if parse_semver(&manifest.version).is_none() {
        eprintln!("plugin.json: 'version' must be a valid semver (e.g. 1.0.0).");
        return false;
    }

    // Check credentials
    let cred_path = dirs_cred_path();
    if !cred_path.exists() {
        eprintln!(
            "No auth token found at {:?}.\n\
             Please authenticate first: frame login",
            cred_path
        );
        return false;
    }

    println!(
        "Validating plugin '{}' v{}…",
        manifest.name, manifest.version
    );
    println!(
        "Note: The Frame Plugin Registry is not yet online. \
         Publishing will be available in a future release."
    );
    true
}

fn dirs_cred_path() -> PathBuf {
    let home = std::env::var("HOME")
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".frame").join("credentials")
}

// ─── GitHub git clone (Phase 6b) ──────────────────────────────────────────────

/// Install a plugin from a GitHub repo: `@user/repo` or `@user/repo@v1.2.3`.
fn install_github_plugin(name: &str, modules_dir: &Path, project_root: &Path) -> bool {
    // Parse @user/repo or @user/repo@v1.2.3
    let at_trimmed = name.trim_start_matches('@');
    let (repo_spec, tag) = if let Some(idx) = at_trimmed.rfind('@') {
        let (repo, tag) = at_trimmed.split_at(idx);
        (repo, Some(&tag[1..]))
    } else {
        (at_trimmed, None)
    };

    // Determine clone URL: user/repo → github.com/user/repo
    let repo_dir_name = repo_spec.replace('/', "_");
    let plugin_dir = modules_dir.join(&repo_dir_name);

    if plugin_dir.exists() {
        println!("Plugin '{name}' is already installed at {:?}", plugin_dir);
        return true;
    }

    let clone_url = format!("https://github.com/{}.git", repo_spec);
    println!("Cloning {}{} ...", clone_url,
             tag.map(|t| format!(" (tag: {t})")).unwrap_or_default());

    let mut cmd = std::process::Command::new("git");
    cmd.arg("clone").arg(&clone_url).arg(&plugin_dir);
    if let Some(t) = tag {
        cmd.arg("--branch").arg(t);
    }
    cmd.arg("--depth").arg("1");

    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Error: git clone failed: {e}");
            return false;
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error: git clone failed: {stderr}");
        // Clean up partial directory
        let _ = std::fs::remove_dir_all(&plugin_dir);
        return false;
    }

    println!("✓ Cloned plugin '{name}'");

    // Update frame.config.json
    let mut config = FrameConfig::load(project_root).unwrap_or_default();
    config.plugins.entry(name.to_string())
        .or_insert_with(|| tag.map(|t| t.to_string()).unwrap_or_else(|| "latest".to_string()));
    if let Err(e) = config.save(project_root) {
        eprintln!("Warning: could not update frame.config.json: {e}");
    }

    // Compute checksum and write frame.lock
    let checksum = compute_plugin_checksum(&plugin_dir);
    let mut lock = FrameLock::load(project_root);
    let version = tag.unwrap_or("0.0.0").trim_start_matches('v');
    lock.set_plugin(name, version, &checksum);
    if let Err(e) = lock.save(project_root) {
        eprintln!("Warning: could not update frame.lock: {e}");
    }

    true
}

// ─── Utility ──────────────────────────────────────────────────────────────────

pub fn pascal_case(s: &str) -> String {
    s.split('_')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}
