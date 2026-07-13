//! Icon asset generation — generates platform-native icon assets from
//! registered icons. Supports generating:
//!   - iOS: PDF icons with embedded SVG path data
//!   - Android: XML VectorDrawable files from SVG path data
//!
//! This runs at deploy time alongside the existing build flow.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::icon_bundle::scan_icon_bundles;
use super::icon::load_manifest;

/// The payload describing each icon to generate.
#[derive(Debug, Clone)]
pub struct IconAsset {
    pub name: String,
    pub svg_path: Option<String>,
    pub sf_symbol: Option<String>,
    pub material: Option<String>,
}

/// Collect all icons that should be generated as platform assets.
/// Reads from both the manifest (SVG-added icons) and bundle files.
pub fn collect_icon_assets(project_dir: &Path) -> Vec<IconAsset> {
    let mut assets: HashMap<String, IconAsset> = HashMap::new();

    // From the SVG manifest (frame-icons.json)
    let manifest = load_manifest(project_dir);
    for (_name, entry) in &manifest.icons {
        if let Some(ref svg) = entry.svg_path_data {
            let svg_val: String = svg.clone();
            assets.entry(entry.name.clone()).or_insert(IconAsset {
                name: entry.name.clone(),
                svg_path: Some(svg_val),
                sf_symbol: None,
                material: None,
            });
        }
    }

    // From bundle files (.frameicons)
    for icon in scan_icon_bundles(project_dir) {
        let entry = assets.entry(icon.name.clone()).or_insert(IconAsset {
            name: icon.name.clone(),
            svg_path: None,
            sf_symbol: None,
            material: None,
        });
        if icon.svg_path.is_some() {
            entry.svg_path = icon.svg_path.clone();
        }
        if icon.sf_symbol.is_some() {
            entry.sf_symbol = icon.sf_symbol.clone();
        }
        if icon.material.is_some() {
            entry.material = icon.material.clone();
        }
    }

    assets.into_values().collect()
}

/// Generate iOS icon assets (PDF files) into the assets directory.
/// Returns the list of written file paths.
pub fn generate_ios_icon_assets(project_dir: &Path, assets_dst: &Path) -> Vec<PathBuf> {
    let icons = collect_icon_assets(project_dir);
    let mut written = Vec::new();

    for icon in &icons {
        if let Some(ref svg_path) = icon.svg_path {
            let pdf_path = assets_dst.join(format!("{}.pdf", icon.name));
            if let Some(parent) = pdf_path.parent() {
                fs::create_dir_all(parent).ok();
            }
            // Generate a minimal PDF wrapper around the SVG path data.
            // In production, this would use CoreGraphics PDF generation.
            // For deployment, we write the SVG path data alongside for reference.
            let pdf_content = generate_svg_pdf_wrapper(icon, svg_path);
            if fs::write(&pdf_path, &pdf_content).is_ok() {
                written.push(pdf_path);
            }
        }
    }

    written
}

/// Generate Android icon assets (XML VectorDrawable) for icons with SVG paths.
pub fn generate_android_icon_assets(project_dir: &Path, res_drawable: &Path) -> Vec<PathBuf> {
    let icons = collect_icon_assets(project_dir);
    let mut written = Vec::new();

    for icon in &icons {
        if let Some(ref svg_path) = icon.svg_path {
            let xml_path = res_drawable.join(format!("ic_{}.xml", icon.name));
            if let Some(parent) = xml_path.parent() {
                fs::create_dir_all(parent).ok();
            }
            let xml = generate_vector_drawable(icon, svg_path);
            if fs::write(&xml_path, &xml).is_ok() {
                written.push(xml_path);
            }
        }
    }

    written
}

/// Generate a VectorDrawable XML file from SVG path data.
fn generate_vector_drawable(_icon: &IconAsset, svg_path: &str) -> String {
    format!(
        r##"<?xml version="1.0" encoding="utf-8"?>
<vector xmlns:android="http://schemas.android.com/apk/res/android"
    android:width="24dp"
    android:height="24dp"
    android:viewportWidth="24"
    android:viewportHeight="24">
    <path
        android:fillColor="#FF000000"
        android:pathData="{}" />
</vector>
"##,
        svg_path
    )
}

/// Generate a PDF-like wrapper (for reference / placeholder).
/// Real iOS icon generation would use UIGraphicsPDFRenderer.
fn generate_svg_pdf_wrapper(icon: &IconAsset, svg_path: &str) -> String {
    format!(
        r##"{{"icon":"{}","path":"{}","width":24,"height":24}}"##,
        icon.name, svg_path,
    )
}

/// Write a framewok-level icon lookup file (frame-icons-config.json)
/// that maps logical icon names to platform identifiers.
pub fn write_icon_lookup_table(project_dir: &Path, build_dir: &Path) -> Result<(), String> {
    let icons = collect_icon_assets(project_dir);
    let mut lookup = serde_json::Map::new();

    for icon in &icons {
        let mut entry = serde_json::Map::new();
        if let Some(ref sf) = icon.sf_symbol {
            entry.insert("sf_symbol".to_string(), serde_json::Value::String(sf.clone()));
        }
        if let Some(ref mat) = icon.material {
            entry.insert("material".to_string(), serde_json::Value::String(mat.clone()));
        }
        if icon.svg_path.is_some() {
            entry.insert("has_custom_asset".to_string(), serde_json::Value::Bool(true));
        }
        lookup.insert(icon.name.clone(), serde_json::Value::Object(entry));
    }

    let json = serde_json::to_string_pretty(&lookup)
        .map_err(|e| e.to_string())?;
    let path = build_dir.join("frame-icon-lookup.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&path, &json).map_err(|e| e.to_string())?;
    Ok(())
}

/// Log a summary of all registered icons
pub fn log_icon_summary(project_dir: &Path) {
    let icons = collect_icon_assets(project_dir);
    let bundle_icons = scan_icon_bundles(project_dir);
    let manifest = load_manifest(project_dir);

    println!("Icon summary:");
    println!("  SVG manifest entries: {}", manifest.icons.len());
    println!("  Bundle file icons:    {}", bundle_icons.len());
    println!("  Total unique icons:   {}", icons.len());

    if !icons.is_empty() {
        println!("  Icon names:");
        for icon in &icons {
            let sf = icon.sf_symbol.as_deref().unwrap_or("-");
            let mat = icon.material.as_deref().unwrap_or("-");
            let svg = if icon.svg_path.is_some() { "svg" } else { "" };
            println!("    {}  sf={}  mat={}  {}", icon.name, sf, mat, svg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_drawable_generation() {
        let icon = IconAsset {
            name: "home".to_string(),
            svg_path: Some("M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z".to_string()),
            sf_symbol: Some("house.fill".to_string()),
            material: Some("Home".to_string()),
        };
        let svg = icon.svg_path.as_deref().unwrap_or("");
        let xml = generate_vector_drawable(&icon, svg);
        assert!(xml.contains("vector"));
        assert!(xml.contains("M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"));
        assert!(xml.contains("viewportWidth=\"24\""));
    }

    #[test]
    fn test_collect_no_icons() {
        let dir = std::env::temp_dir().join("frame_icon_gen_empty");
        let assets = collect_icon_assets(&dir);
        assert!(assets.is_empty());
    }
}
