//! Icon bundle system — reads `.frameicons` bundle files containing
//! multiple icon definitions in a single file.
//!
//! This sits alongside the existing SVG-based icon system (`icon.rs`)
//! without modifying it. The bundle format supports multiple icon types:
//!   - `sf_symbol` – Apple SF Symbol name (iOS)
//!   - `material`  – Material Design Icon name (Android)
//!   - `svg_path`  – raw SVG path data for custom rendering
//!
//! Bundle files use a JSON-based format:
//! ```json
//! {
//!   "icons": [
//!     { "name": "home", "sf_symbol": "house.fill", "material": "Home" }
//!   ]
//! }
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// A single icon definition from a bundle file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleIcon {
    pub name: String,
    #[serde(default)]
    pub sf_symbol: Option<String>,
    #[serde(default)]
    pub material: Option<String>,
    #[serde(default)]
    pub svg_path: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

/// A `.frameicons` bundle file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconBundle {
    #[serde(default = "default_version")]
    pub version: String,
    pub icons: Vec<BundleIcon>,
}

fn default_version() -> String {
    "1.0".to_string()
}

impl IconBundle {
    /// Load a single `.frameicons` file.
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
        serde_json::from_str::<IconBundle>(&content)
            .map_err(|e| format!("Invalid icon bundle {}: {e}", path.display()))
    }

    /// Load all `.frameicons` files from a directory.
    pub fn load_all(dir: &Path) -> Vec<BundleIcon> {
        if !dir.exists() {
            return Vec::new();
        }
        let mut all = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "frameicons").unwrap_or(false) {
                    if let Ok(bundle) = Self::load(&path) {
                        all.extend(bundle.icons);
                    }
                }
            }
        }
        all
    }

    /// Save a bundle to a file.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| e.to_string())?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(path, &json).map_err(|e| e.to_string())
    }

    /// Merge another bundle's icons into this one (dedup by name).
    pub fn merge(&mut self, other: IconBundle) {
        for icon in other.icons {
            if !self.icons.iter().any(|i| i.name == icon.name) {
                self.icons.push(icon);
            }
        }
    }
}

/// Scan all bundled icons from `assets/icons/` (`.frameicons` files).
/// This is called alongside `scan_icons_dir` from `icon.rs`.
pub fn scan_icon_bundles(project_dir: &Path) -> Vec<BundleIcon> {
    let bundles_dir = project_dir.join("assets").join("icons");
    IconBundle::load_all(&bundles_dir)
}

/// Build a lookup map from bundled icon name to BundleIcon.
pub fn bundle_icon_map(project_dir: &Path) -> HashMap<String, BundleIcon> {
    let mut map = HashMap::new();
    for icon in scan_icon_bundles(project_dir) {
        map.entry(icon.name.clone()).or_insert_with(|| icon.clone());
    }
    map
}

/// Write a default `.frameicons` file with starter icons.
pub fn write_default_bundle(project_dir: &Path) -> Result<(), String> {
    let dir = project_dir.join("assets").join("icons");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("default.frameicons");

    if path.exists() {
        return Ok(()); // Don't overwrite
    }

    let bundle = IconBundle {
        version: "1.0".to_string(),
        icons: vec![
            BundleIcon {
                name: "add".to_string(),
                sf_symbol: Some("plus".to_string()),
                material: Some("Add".to_string()),
                svg_path: None,
                category: Some("actions".to_string()),
                tags: Some(vec!["action".to_string(), "ui".to_string()]),
            },
            BundleIcon {
                name: "settings".to_string(),
                sf_symbol: Some("gearshape".to_string()),
                material: Some("Settings".to_string()),
                svg_path: None,
                category: Some("actions".to_string()),
                tags: Some(vec!["action".to_string(), "ui".to_string()]),
            },
            BundleIcon {
                name: "search".to_string(),
                sf_symbol: Some("magnifyingglass".to_string()),
                material: Some("Search".to_string()),
                svg_path: None,
                category: Some("actions".to_string()),
                tags: Some(vec!["action".to_string()]),
            },
        ],
    };
    bundle.save(&path)
}

/// `frame icon load-bundle <path>` — register all icons from a bundle file.
pub fn run_icon_load_bundle(bundle_path: &str, project_dir: &Path) -> Result<(), String> {
    let path = Path::new(bundle_path);
    if !path.exists() {
        return Err(format!("Bundle file not found: {bundle_path}"));
    }

    let bundle = IconBundle::load(path)?;
    let count = bundle.icons.len();
    println!("Loaded {} icon(s) from {}", count, bundle_path);

    // Write to assets/icons/ as a project-local bundle
    let dest_dir = project_dir.join("assets").join("icons");
    fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;

    // Check for name collisions
    let existing = scan_icon_bundles(project_dir);
    let existing_names: Vec<&str> = existing.iter().map(|i| i.name.as_str()).collect();
    let mut collision_count = 0;
    for icon in &bundle.icons {
        if existing_names.contains(&icon.name.as_str()) {
            println!("  Warning: icon '{}' already exists, skipping", icon.name);
            collision_count += 1;
        }
    }

    if collision_count < count {
        let dest_path = dest_dir.join(format!("imported_{}.frameicons", 
            path.file_stem().unwrap_or_default().to_string_lossy()));
        let existing_content = if dest_path.exists() {
            let mut existing_bundle = IconBundle::load(&dest_path)?;
            existing_bundle.merge(bundle);
            serde_json::to_string_pretty(&existing_bundle).map_err(|e| e.to_string())?
        } else {
            serde_json::to_string_pretty(&bundle).map_err(|e| e.to_string())?
        };
        fs::write(&dest_path, &existing_content).map_err(|e| e.to_string())?;
        println!("  Written to {}", dest_path.display());
    }

    println!("✓ Bundle loaded: {count} icon(s), {collision_count} collision(s) skipped");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_bundle_roundtrip() {
        let bundle = IconBundle {
            version: "1.0".to_string(),
            icons: vec![
                BundleIcon {
                    name: "home".to_string(),
                    sf_symbol: Some("house.fill".to_string()),
                    material: Some("Home".to_string()),
                    svg_path: None,
                    category: Some("navigation".to_string()),
                    tags: Some(vec!["ui".to_string()]),
                },
            ],
        };

        let dir = std::env::temp_dir().join("frame_bundle_test");
        let path = dir.join("test.frameicons");
        bundle.save(&path).unwrap();
        let loaded = IconBundle::load(&path).unwrap();
        assert_eq!(loaded.icons.len(), 1);
        assert_eq!(loaded.icons[0].name, "home");
        assert_eq!(loaded.icons[0].sf_symbol.as_deref(), Some("house.fill"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_scan_empty_dir() {
        let dir = std::env::temp_dir().join("frame_bundle_empty");
        let icons = scan_icon_bundles(&dir);
        assert!(icons.is_empty());
    }

    #[test]
    fn test_merge_dedup() {
        let mut bundle = IconBundle {
            version: "1.0".to_string(),
            icons: vec![
                BundleIcon {
                    name: "home".to_string(),
                    sf_symbol: Some("house.fill".to_string()),
                    material: None,
                    svg_path: None,
                    category: None,
                    tags: None,
                },
            ],
        };
        let other = IconBundle {
            version: "1.0".to_string(),
            icons: vec![
                BundleIcon {
                    name: "home".to_string(), // duplicate
                    sf_symbol: Some("house".to_string()),
                    material: None,
                    svg_path: None,
                    category: None,
                    tags: None,
                },
                BundleIcon {
                    name: "search".to_string(),
                    sf_symbol: Some("magnifyingglass".to_string()),
                    material: None,
                    svg_path: None,
                    category: None,
                    tags: None,
                },
            ],
        };
        bundle.merge(other);
        assert_eq!(bundle.icons.len(), 2);
        // Original "home" kept, not overwritten
        assert_eq!(bundle.icons[0].sf_symbol.as_deref(), Some("house.fill"));
    }

    #[test]
    fn test_load_invalid_json() {
        let dir = std::env::temp_dir().join("frame_bundle_invalid");
        let path = dir.join("bad.frameicons");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, "not json").unwrap();
        let result = IconBundle::load(&path);
        assert!(result.is_err());
        std::fs::remove_dir_all(&dir).ok();
    }
}
