//! Font management CLI commands (plan §4).
//!
//! - Directory scanning (`assets/fonts/`) — Phase 4a
//! - Font registry + typography integration — Phase 4b
//! - Google Fonts auto-download on deploy — Phase 4d

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// ─── FontEntry ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FontEntry {
    pub family: String,
    pub file_name: String,
    pub path: PathBuf,
    pub format: FontFormat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FontFormat {
    Ttf,
    Otf,
}

impl FontFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            FontFormat::Ttf => "ttf",
            FontFormat::Otf => "otf",
        }
    }
}

// ─── FontRegistry ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct FontRegistry {
    /// font_family → list of files/weights
    pub fonts: HashMap<String, Vec<FontEntry>>,
}

impl FontRegistry {
    /// Scan `assets/fonts/` and build the registry (Phase 4a).
    pub fn scan(project_dir: &Path) -> Self {
        let fonts_dir = project_dir.join("assets").join("fonts");
        if !fonts_dir.exists() {
            return FontRegistry::default();
        }

        let mut registry = FontRegistry::default();
        if let Ok(dir) = fs::read_dir(&fonts_dir) {
            for entry in dir.flatten() {
                let path = entry.path();
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let format = match ext {
                    "ttf" => FontFormat::Ttf,
                    "otf" => FontFormat::Otf,
                    _ => continue,
                };

                let file_name = path.file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_default();

                // Family name: filename minus extension, snake_case → Title Case
                let stem = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                let family = stem_to_family_name(stem);

                registry.fonts.entry(family.clone())
                    .or_default()
                    .push(FontEntry {
                        family,
                        file_name,
                        path,
                        format,
                    });
            }
        }
        registry
    }

    /// Returns true if font_family is registered.
    pub fn has_family(&self, name: &str) -> bool {
        self.fonts.contains_key(name)
    }
}

fn stem_to_family_name(stem: &str) -> String {
    // Split on hyphens, underscores, and CamelCase boundaries
    let parts: Vec<String> = stem.split(|c: char| c == '-' || c == '_')
        .flat_map(|s| split_camel_case(s))
        .filter(|s| !s.is_empty())
        .collect();

    if parts.is_empty() {
        return stem.to_string();
    }

    // Check if the last part is a weight name
    let weights = ["thin", "extralight", "light", "regular", "medium",
                    "semibold", "bold", "extrabold", "black",
                    "thinitalic", "extralightitalic", "lightitalic",
                    "italic", "mediumitalic", "semibolditalic",
                    "bolditalic", "extrabolditalic", "blackitalic"];

    let last_lower = parts.last().unwrap().to_lowercase();
    let family_parts: Vec<&str> = if weights.contains(&last_lower.as_str()) {
        parts[..parts.len() - 1].iter().map(|s| s.as_str()).collect()
    } else {
        parts.iter().map(|s| s.as_str()).collect()
    };

    // Capitalize each word
    family_parts.iter()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().to_string() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn split_camel_case(s: &str) -> Vec<String> {
    if s.is_empty() {
        return vec![];
    }
    // Detect if this is already lowercase-separated or mixed case
    let has_upper = s.chars().any(|c| c.is_uppercase());
    if !has_upper {
        return vec![s.to_string()];
    }
    let mut result = Vec::new();
    let mut current = String::new();
    for (i, c) in s.char_indices() {
        if c.is_uppercase() && i > 0 {
            result.push(current.clone());
            current.clear();
        }
        current.push(c);
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

// ─── Google Fonts support (Phase 4d) ───────────────────────────────────────────

/// Check if a font family is available on Google Fonts and download it.
/// Returns the path to the downloaded font file.
pub fn download_google_font(family: &str, project_dir: &Path) -> Result<PathBuf, String> {
    let cache_dir = project_dir.join("assets").join("fonts").join(".google-fonts");
    fs::create_dir_all(&cache_dir).map_err(|e| format!("Cannot create cache dir: {e}"))?;

    let safe_name = family.to_lowercase().replace(' ', "-");
    let target_path = cache_dir.join(format!("{safe_name}.ttf"));

    if target_path.exists() {
        return Ok(target_path);
    }

    // Download from Google Fonts API
    let url = format!("https://fonts.google.com/download?family={}", url_encode(family));
    let response = download_url(&url).map_err(|e| format!("Failed to download font '{family}': {e}"))?;
    fs::write(&target_path, &response).map_err(|e| format!("Cannot write font file: {e}"))?;

    println!("  Downloaded Google Font: {family}");
    Ok(target_path)
}

fn url_encode(s: &str) -> String {
    s.replace(' ', "+")
}

fn download_url(url: &str) -> Result<Vec<u8>, String> {
    // Use curl to download
    let output = std::process::Command::new("curl")
        .arg("-s")
        .arg("-L")
        .arg(url)
        .output()
        .map_err(|e| format!("curl failed: {e}"))?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(format!("curl returned {}", output.status))
    }
}

// ─── Deploy integration ────────────────────────────────────────────────────────

/// Copy all registered fonts to platform-specific locations (Phase 4b deploy).
pub fn copy_fonts_for_deploy(
    registry: &FontRegistry,
    project_dir: &Path,
    target: &str,
) -> Result<(), String> {
    let dest_dir = match target {
        "android" => project_dir.join("build").join("android").join("app").join("src").join("main").join("res").join("font"),
        "ios" => project_dir.join("build").join("ios").join("Fonts"),
        other => return Err(format!("Unknown target: {other}")),
    };
    fs::create_dir_all(&dest_dir).map_err(|e| format!("Cannot create {dest_dir:?}: {e}"))?;

    for (_family, entries) in &registry.fonts {
        for entry in entries {
            let dest = dest_dir.join(&entry.file_name);
            fs::copy(&entry.path, &dest).map_err(|e| {
                format!("Cannot copy font {}: {e}", entry.file_name)
            })?;
        }
    }
    Ok(())
}

// ─── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stem_to_family_simple() {
        assert_eq!(stem_to_family_name("Inter"), "Inter");
        assert_eq!(stem_to_family_name("Roboto"), "Roboto");
    }

    #[test]
    fn test_stem_to_family_with_weight() {
        assert_eq!(stem_to_family_name("Inter-Bold"), "Inter");
        assert_eq!(stem_to_family_name("OpenSans-Regular"), "Open Sans");
        assert_eq!(stem_to_family_name("RobotoMono-Light"), "Roboto Mono");
        assert_eq!(stem_to_family_name("Inter"), "Inter");
        assert_eq!(stem_to_family_name("open_sans_regular"), "Open Sans");
    }

    #[test]
    fn test_font_format_detection() {
        let p = Path::new("test.ttf");
        assert!(p.extension().map(|e| e == "ttf").unwrap_or(false));
    }

    #[test]
    fn test_font_registry_scan_nonexistent() {
        let dir = std::env::temp_dir().join("frame_font_test_empty");
        let reg = FontRegistry::scan(&dir);
        assert!(reg.fonts.is_empty());
    }

    #[test]
    fn test_font_registry_scan_with_files() {
        let dir = std::env::temp_dir().join("frame_font_test_files");
        let fonts_dir = dir.join("assets").join("fonts");
        fs::create_dir_all(&fonts_dir).unwrap();
        fs::write(fonts_dir.join("Inter-Bold.ttf"), "mock ttf data").unwrap();
        fs::write(fonts_dir.join("Inter-Regular.ttf"), "mock ttf data").unwrap();
        fs::write(fonts_dir.join("RobotoMono-Light.otf"), "mock otf data").unwrap();

        let reg = FontRegistry::scan(&dir);
        assert!(reg.has_family("Inter"), "Should have Inter family, has: {:?}", reg.fonts.keys().collect::<Vec<_>>());
        assert!(reg.has_family("Roboto Mono"), "Should have Roboto Mono family");
        assert_eq!(reg.fonts.get("Inter").unwrap().len(), 2);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_copy_fonts_for_deploy() {
        let dir = std::env::temp_dir().join("frame_font_deploy_test");
        let fonts_dir = dir.join("assets").join("fonts");
        fs::create_dir_all(&fonts_dir).unwrap();

        let font_path = fonts_dir.join("TestFont-Regular.ttf");
        fs::write(&font_path, "mock font").unwrap();

        let reg = FontRegistry::scan(&dir);
        assert!(reg.has_family("Test Font"));

        copy_fonts_for_deploy(&reg, &dir, "android").unwrap();
        let android_dest = dir.join("build").join("android").join("app").join("src").join("main").join("res").join("font");
        assert!(android_dest.join("TestFont-Regular.ttf").exists());

        copy_fonts_for_deploy(&reg, &dir, "ios").unwrap();
        let ios_dest = dir.join("build").join("ios").join("Fonts");
        assert!(ios_dest.join("TestFont-Regular.ttf").exists());

        fs::remove_dir_all(&dir).ok();
    }
}
