//! Icon management CLI commands (plan §3b, §3c).
//!
//! - `frame icon add <path>` — register an SVG icon in the project
//! - Icon directory scanning (`assets/icons/`) happens at compile time

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

// ─── IconManifest ──────────────────────────────────────────────────────────────

/// The project's icon registry, persisted as `frame-icons.json`.
#[derive(Debug, Clone, Default)]
pub struct IconManifest {
    pub icons: HashMap<String, IconEntry>,
}

#[derive(Debug, Clone)]
pub struct IconEntry {
    pub name: String,
    pub source: IconSource,
    pub svg_path_data: Option<String>,
}

#[derive(Debug, Clone)]
pub enum IconSource {
    BuiltIn,
    UserAdded,
    Plugin,
}

// ─── In-memory cache ───────────────────────────────────────────────────────────

static ICON_MANIFEST: OnceLock<IconManifest> = OnceLock::new();

pub fn get_icon_manifest() -> &'static IconManifest {
    ICON_MANIFEST.get_or_init(|| {
        // Try to load from project or return empty
        IconManifest::default()
    })
}

// ─── SVG helpers ───────────────────────────────────────────────────────────────

/// Extract the `d` attribute from the first `<path>` element in an SVG.
fn extract_svg_path_data(svg_content: &str) -> Option<String> {
    // Simple extraction: find d="..." in <path ... d="..."/>
    if let Some(path_start) = svg_content.find("<path") {
        let rest = &svg_content[path_start..];
        if let Some(d_start) = rest.find("d=\"") {
            let after_d = &rest[d_start + 3..];
            if let Some(d_end) = after_d.find('"') {
                return Some(after_d[..d_end].to_string());
            }
        }
    }
    None
}

/// Quick SVG validation: checks for XML declaration or <svg tag
fn validate_svg(content: &str) -> Result<(), String> {
    if content.contains("<svg") || content.contains("<?xml") {
        // Check basic structure
        if let Some(path_data) = extract_svg_path_data(content) {
            if path_data.is_empty() {
                return Err("SVG has no <path d=\"...\"> element".to_string());
            }
            Ok(())
        } else {
            Err("SVG has no <path> element with a `d` attribute".to_string())
        }
    } else {
        Err("File does not appear to be a valid SVG".to_string())
    }
}

// ─── Manifest persistence ──────────────────────────────────────────────────────

fn manifest_path(project_dir: &Path) -> PathBuf {
    project_dir.join("frame-icons.json")
}

pub(crate) fn load_manifest(project_dir: &Path) -> IconManifest {
    let path = manifest_path(project_dir);
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        IconManifest::default()
    }
}

fn save_manifest(manifest: &IconManifest, project_dir: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(manifest).map_err(|e| e.to_string())?;
    fs::write(manifest_path(project_dir), &json).map_err(|e| e.to_string())
}

impl serde::Serialize for IconManifest {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.icons.len()))?;
        for (name, entry) in &self.icons {
            map.serialize_entry(name, &entry)?;
        }
        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for IconManifest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let icons: HashMap<String, IconEntry> = HashMap::deserialize(deserializer)?;
        Ok(IconManifest { icons })
    }
}

impl serde::Serialize for IconEntry {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("IconEntry", 2)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("svg_path_data", &self.svg_path_data)?;
        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for IconEntry {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct IconEntryHelper {
            name: String,
            svg_path_data: Option<String>,
        }
        let helper = IconEntryHelper::deserialize(deserializer)?;
        Ok(IconEntry {
            name: helper.name,
            source: IconSource::UserAdded,
            svg_path_data: helper.svg_path_data,
        })
    }
}

// ─── Directory scanning (§3c) ──────────────────────────────────────────────────

/// Scan `assets/icons/` for SVG files and add them to a manifest.
pub fn scan_icons_dir(project_dir: &Path) -> Vec<IconEntry> {
    let icons_dir = project_dir.join("assets").join("icons");
    if !icons_dir.exists() {
        return Vec::new();
    }
    let mut entries = Vec::new();
    if let Ok(dir) = fs::read_dir(&icons_dir) {
        for entry in dir.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "svg").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    let name = path.file_stem().unwrap().to_string_lossy().to_string();
                    let svg_path_data = extract_svg_path_data(&content);
                    entries.push(IconEntry {
                        name,
                        source: IconSource::UserAdded,
                        svg_path_data,
                    });
                }
            }
        }
    }
    entries
}

// ─── CLI handler (§3b) ─────────────────────────────────────────────────────────

/// `frame icon add <path> [--name <name>]`
pub fn run_icon_add(svg_path: &str, name: Option<&str>, project_dir: &Path) -> Result<(), String> {
    let path = Path::new(svg_path);
    if !path.exists() {
        return Err(format!("File not found: {svg_path}"));
    }

    let content = fs::read_to_string(path).map_err(|e| format!("Cannot read file: {e}"))?;

    // Validate SVG
    validate_svg(&content)?;

    // Extract data
    let svg_path_data = extract_svg_path_data(&content);
    let icon_name = name
        .map(|n| n.to_string())
        .unwrap_or_else(|| {
            path.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        });

    // Load manifest, add icon, save
    let mut manifest = load_manifest(project_dir);
    manifest.icons.insert(icon_name.clone(), IconEntry {
        name: icon_name.clone(),
        source: IconSource::UserAdded,
        svg_path_data,
    });
    save_manifest(&manifest, project_dir)?;

    println!("✓ Icon '{}' registered ({})", icon_name, svg_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_svg_path_data() {
        let svg = r#"<svg viewBox="0 0 24 24"><path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/></svg>"#;
        assert_eq!(extract_svg_path_data(svg), Some("M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z".to_string()));
    }

    #[test]
    fn test_validate_svg_ok() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><path d="M0 0h24v24H0z"/></svg>"#;
        assert!(validate_svg(svg).is_ok());
    }

    #[test]
    fn test_validate_svg_no_path() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><circle cx="12" cy="12" r="10"/></svg>"#;
        assert!(validate_svg(svg).is_err());
    }

    #[test]
    fn test_scan_icons_dir_empty() {
        let dir = std::env::temp_dir().join("frame_icon_test_empty");
        let entries = scan_icons_dir(&dir);
        assert!(entries.is_empty());
    }

    #[test]
    fn test_scan_icons_dir_with_files() {
        let dir = std::env::temp_dir().join("frame_icon_test_files");
        let icons_dir = dir.join("assets").join("icons");
        fs::create_dir_all(&icons_dir).unwrap();
        fs::write(icons_dir.join("home.svg"), r#"<svg><path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/></svg>"#).unwrap();
        fs::write(icons_dir.join("settings.svg"), r#"<svg><path d="M12 15a3 3 0 100-6 3 3 0 000 6z"/></svg>"#).unwrap();

        let entries = scan_icons_dir(&dir);
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|e| e.name == "home"));
        assert!(entries.iter().any(|e| e.name == "settings"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_icon_add_and_manifest() {
        let dir = std::env::temp_dir().join("frame_icon_add_test");
        fs::create_dir_all(&dir).unwrap();
        let svg_path = dir.join("test_icon.svg");
        fs::write(&svg_path, r#"<svg><path d="M0 0h24v24H0z"/></svg>"#).unwrap();

        run_icon_add(&svg_path.to_string_lossy(), Some("custom_name"), &dir).unwrap();
        let manifest = load_manifest(&dir);
        assert!(manifest.icons.contains_key("custom_name"));
        fs::remove_dir_all(&dir).ok();
    }
}
