//! iOS / UIKit code generator for the Frame framework.
//!
//! Entry point: `gen_ios(ast, config) -> Vec<OutputFile>`
//! Mirrors android.rs — every built-in component has a UIKit mapping.

#![allow(unused_variables, dead_code)]
use crate::parser::ast::*;
use std::collections::HashMap;

// ─── Config / OutputFile ──────────────────────────────────────────────────────

/// iOS project configuration.
#[derive(Debug, Clone)]
pub struct IosConfig {
    pub bundle_id: String,
    pub app_name: String,
    pub version: String,
    pub build_number: String,
    pub min_ios: String,         // e.g. "16.0"
    pub team_id: String,
    pub deployment_target: String, // e.g. "16.0"
}

impl Default for IosConfig {
    fn default() -> Self {
        IosConfig {
            bundle_id: "com.example.frameapp".to_string(),
            app_name: "Frame App".to_string(),
            version: "1.0".to_string(),
            build_number: "1".to_string(),
            min_ios: "16.0".to_string(),
            team_id: "XXXXXXXXXX".to_string(),
            deployment_target: "16.0".to_string(),
        }
    }
}

/// A generated file with its relative path and content.
#[derive(Debug, Clone)]
pub struct OutputFile {
    pub path: String,
    pub content: String,
}

// ─── Public entry point ───────────────────────────────────────────────────────

/// Generate all iOS project files from an AST + config.
pub fn gen_ios(ast: &AST, config: &IosConfig) -> Vec<OutputFile> {
    gen_ios_with_plugins(ast, config, &[])
}

/// Like [`gen_ios`] but also accepts extra Swift source files from plugins.
/// Each entry is `(filename, source_content)`.
pub fn gen_ios_with_plugins(
    ast: &AST,
    config: &IosConfig,
    extra_swift_sources: &[(&str, &str)],
) -> Vec<OutputFile> {
    let mut files: Vec<OutputFile> = Vec::new();
    let bundle_id = &config.bundle_id;
    let app_name = &config.app_name;
    let safe_name = app_name.replace(' ', "");

    // ── Architecture detection ──────────────────────────────────────────────────
    // Mirrors the Android arch detection — same three buckets.
    let arch = detect_ios_arch(ast);
    // iOS subdirectory prefixes for each file category
    let screens_dir  = ios_screens_dir(arch);   // e.g. "Screens/" or "Presentation/Screens/"
    let comp_dir     = ios_comp_dir(arch);       // e.g. "Components/" or "Presentation/Components/"
    let stores_dir   = ios_stores_dir(arch);     // e.g. "Stores/" or "Data/Stores/"
    let models_dir   = ios_models_dir(arch);     // e.g. "Models/" or "Domain/Models/"

    // Detect features
    let uses_fetch        = ios_uses_fetch(ast);
    let uses_camera       = ios_uses_call(ast, "camera:capture");
    let uses_audio_record = ios_uses_call(ast, "audio:record");
    let uses_location     = ios_uses_call(ast, "location:get") || ios_uses_call(ast, "location:watch");
    let uses_location_bg  = ios_uses_call(ast, "location:background");
    let uses_notification = ios_uses_call(ast, "notification:send");
    let uses_bluetooth    = ios_uses_call(ast, "bluetooth:connect") || ios_uses_call(ast, "bluetooth:scan");
    let uses_contacts     = ios_uses_call(ast, "contacts:read") || ios_uses_call(ast, "contacts:write");
    let uses_calendar     = ios_uses_call(ast, "calendar:read") || ios_uses_call(ast, "calendar:write");
    let uses_photos       = ios_uses_call(ast, "storage:images");
    let uses_health       = ios_uses_call(ast, "health:steps");
    let uses_speech       = ios_uses_call(ast, "speech:recognize");
    let uses_face_id      = ios_uses_call(ast, "auth:biometric");
    let uses_http         = uses_fetch;

    // ── Xcode project file (project.pbxproj) ─────────────────────────────────
    // Collect all Swift source file names BEFORE building pbxproj.
    let mut swift_sources: Vec<String> = vec![
        "AppDelegate.swift".to_string(),
        "SceneDelegate.swift".to_string(),
        "MainViewController.swift".to_string(),
        "KeychainHelper.swift".to_string(),
        "UIColorExtension.swift".to_string(),
        "RouteHelper.swift".to_string(),
    ];
    // App lifecycle wiring file — only when :app {} hooks are declared
    if ast.on_launch.is_some() || ast.on_foreground.is_some() || ast.on_background.is_some() {
        swift_sources.push("FrameAppLifecycle.swift".to_string());
    }
    for page in &ast.pages {
        swift_sources.push(format!("{}{}ViewController.swift", screens_dir, page.name));
    }
    for name in ast.components.keys() {
        swift_sources.push(format!("{}{}View.swift", comp_dir, name));
    }
    for name in ast.stores.keys() {
        swift_sources.push(format!("{}{}Store.swift", stores_dir, name));
    }
    if uses_camera       { swift_sources.push("CameraHelper.swift".to_string()); }
    if uses_location     { swift_sources.push("LocationHelper.swift".to_string()); }
    if uses_notification { swift_sources.push("NotificationHelper.swift".to_string()); }

    // Plugin Swift sources — added to pbxproj and generated as output files
    for (fname, content) in extra_swift_sources {
        swift_sources.push(fname.to_string());
        files.push(OutputFile {
            path: fname.to_string(),
            content: content.to_string(),
        });
    }

    files.push(gen_xcodeproj_pbxproj(config, &swift_sources));
    files.push(gen_xcscheme(config));
    files.push(gen_xcworkspace_data(config));
    files.push(gen_gitignore_ios());

    // Project scaffolding
    files.push(gen_info_plist_full(config, uses_camera, uses_audio_record,
        uses_location, uses_location_bg, uses_notification, uses_bluetooth,
        uses_contacts, uses_calendar, uses_photos, uses_health,
        uses_speech, uses_face_id, uses_http));
    files.push(gen_app_delegate(bundle_id, app_name, ast));
    files.push(gen_scene_delegate(bundle_id));
    // Generate FrameApp lifecycle wiring if :app {} hooks are declared
    if ast.on_launch.is_some() || ast.on_foreground.is_some() || ast.on_background.is_some() {
        files.push(gen_frame_app_lifecycle(ast));
    }
    files.push(gen_assets_xcassets());
    files.push(gen_podfile(config, ast));

    // Entry point: main screen controller wiring
    files.push(gen_main_view_controller(ast, bundle_id));

    // Per-page ViewControllers — placed in arch-aware subdirectory
    for page in &ast.pages {
        files.push(gen_page_view_controller_at(page, ast, bundle_id, screens_dir));
    }

    // Custom components → UIView subclasses
    for (name, comp) in &ast.components {
        files.push(gen_component_view_at(name, comp, bundle_id, comp_dir));
    }

    // Store ObservableObjects
    for (name, store) in &ast.stores {
        files.push(gen_store_swift_at(name, store, bundle_id, stores_dir));
    }

    // :obj declarations → Swift Codable structs
    for obj in ast.objects.values() {
        let mut f = gen_obj_swift(obj);
        f.path = format!("{}{}", models_dir, f.path);
        swift_sources.push(f.path.clone());
        files.push(f);
    }

    // :enum declarations → Swift enums (plan §1b)
    for enum_def in ast.enums.values() {
        let mut f = gen_enum_swift(enum_def);
        f.path = format!("{}{}", models_dir, f.path);
        swift_sources.push(f.path.clone());
        files.push(f);
    }

    // :type aliases → Swift typealiases (plan §1c)
    for type_alias in ast.type_aliases.values() {
        let mut f = gen_type_alias_swift(type_alias);
        f.path = format!("{}{}", models_dir, f.path);
        swift_sources.push(f.path.clone());
        files.push(f);
    }

    // KeychainHelper (always generated — stores may need it)
    files.push(gen_keychain_helper());

    // UIColor hex extension — needed by any component with color: style
    files.push(gen_uicolor_extension());

    // Route helper (navigate() calls in Swift use this)
    files.push(gen_route_helper(ast));

    // Platform feature helpers
    if uses_camera        { files.push(gen_camera_helper_swift(bundle_id)); }
    if uses_location      { files.push(gen_location_helper_swift(bundle_id)); }
    if uses_notification  { files.push(gen_notification_helper_swift(bundle_id)); }

    files
}

// ─── iOS Architecture helpers ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
enum IosArch { Mvc, Clean, Flat }

fn detect_ios_arch(ast: &AST) -> IosArch {
    for imp in &ast.imports {
        if imp.path.contains("presentation/") || imp.path.contains("domain/") {
            return IosArch::Clean;
        }
        if imp.path.contains("views/") || imp.path.contains("controllers/") {
            return IosArch::Mvc;
        }
    }
    IosArch::Flat
}

fn ios_screens_dir(arch: IosArch) -> &'static str {
    match arch {
        IosArch::Mvc   => "Screens/",
        IosArch::Clean => "Presentation/Screens/",
        IosArch::Flat  => "",
    }
}

fn ios_comp_dir(arch: IosArch) -> &'static str {
    match arch {
        IosArch::Mvc   => "Components/",
        IosArch::Clean => "Presentation/Components/",
        IosArch::Flat  => "",
    }
}

fn ios_stores_dir(arch: IosArch) -> &'static str {
    match arch {
        IosArch::Mvc   => "Stores/",
        IosArch::Clean => "Data/Stores/",
        IosArch::Flat  => "",
    }
}

fn ios_models_dir(arch: IosArch) -> &'static str {
    match arch {
        IosArch::Mvc   => "Models/",
        IosArch::Clean => "Domain/Models/",
        IosArch::Flat  => "",
    }
}

/// Wrapper: generate a ViewController placed in `subdir`.
fn gen_page_view_controller_at(page: &Page, ast: &AST, bundle_id: &str, subdir: &str) -> OutputFile {
    let mut f = gen_page_view_controller(page, ast, bundle_id);
    if !subdir.is_empty() {
        f.path = format!("{}{}", subdir, f.path);
    }
    f
}

/// Wrapper: generate a component UIView subclass placed in `subdir`.
fn gen_component_view_at(name: &str, comp: &ComponentDef, bundle_id: &str, subdir: &str) -> OutputFile {
    let mut f = gen_component_view(name, comp, bundle_id);
    if !subdir.is_empty() {
        f.path = format!("{}{}", subdir, f.path);
    }
    f
}

/// Wrapper: generate an ObservableObject store placed in `subdir`.
fn gen_store_swift_at(name: &str, store: &StoreSlice, bundle_id: &str, subdir: &str) -> OutputFile {
    let mut f = gen_store_swift(name, store, bundle_id);
    if !subdir.is_empty() {
        f.path = format!("{}{}", subdir, f.path);
    }
    f
}

// ─── Xcode project file (project.pbxproj) ────────────────────────────────────
// Without this file the output directory cannot be opened in Xcode at all.
// We generate a minimal-but-complete single-target project referencing all Swift sources.

/// Generate the .xcodeproj/project.pbxproj file for the app.
/// Uses stable UUIDs derived from the app name so the file is deterministic.
fn gen_xcodeproj_pbxproj(config: &IosConfig, swift_sources: &[String]) -> OutputFile {
    let safe_name = config.app_name.replace(' ', "");
    let bundle_id = &config.bundle_id;
    let min_ios   = &config.min_ios;
    let team_id   = &config.team_id;

    // Generate deterministic-looking hex UUIDs from a base seed.
    // In a real project each file gets a unique UUID; we derive them
    // by hashing the filename with a simple deterministic scheme.
    fn uuid(seed: u64) -> String {
        let a = seed.wrapping_mul(0x9e3779b97f4a7c15u64);
        let b = (seed ^ 0xdeadbeef) as u32;
        format!("{a:016X}{b:08X}")
    }

    let project_uuid   = uuid(1);
    let target_uuid    = uuid(2);
    let build_cfg_list = uuid(3);
    let debug_cfg      = uuid(4);
    let release_cfg    = uuid(5);
    let sources_phase  = uuid(6);
    let resources_phase = uuid(7);
    let frameworks_phase = uuid(8);
    let main_group     = uuid(9);
    let products_group = uuid(10);
    let xcassets_ref   = uuid(11);
    let info_plist_ref = uuid(12);
    let product_ref    = uuid(14);

    // Build file references and source build phases for every Swift file
    let mut file_refs  = String::new();
    let mut build_files = String::new();
    let mut sources_list = String::new();

    for (i, src) in swift_sources.iter().enumerate() {
        let ref_id   = uuid(100 + i as u64);
        let build_id = uuid(200 + i as u64);
        file_refs.push_str(&format!(
            "\t\t{ref_id} /* {src} */ = {{isa = PBXFileReference; lastKnownFileType = sourcecode.swift; path = {src}; sourceTree = \"<group>\"; }};\n"
        ));
        build_files.push_str(&format!(
            "\t\t{build_id} /* {src} in Sources */ = {{isa = PBXBuildFile; fileRef = {ref_id} /* {src} */; }};\n"
        ));
        sources_list.push_str(&format!(
            "\t\t\t\t{build_id} /* {src} in Sources */,\n"
        ));
    }

    // Group children list (all source refs + xcassets + info plist + launch screen)
    let mut group_children = String::new();
    for (i, src) in swift_sources.iter().enumerate() {
        let ref_id = uuid(100 + i as u64);
        group_children.push_str(&format!("\t\t\t\t{ref_id} /* {src} */,\n"));
    }
    group_children.push_str(&format!("\t\t\t\t{xcassets_ref} /* Assets.xcassets */,\n"));
    group_children.push_str(&format!("\t\t\t\t{info_plist_ref} /* Info.plist */,\n"));
    // LaunchScreen.storyboard removed — using UILaunchScreen dict in Info.plist instead

    let xcassets_build_id = uuid(900);

    let content = format!(r#"// !$*UTF8*$!
{{
	archiveVersion = 1;
	classes = {{
	}};
	objectVersion = 56;
	objects = {{

/* Begin PBXBuildFile section */
{build_files}		{xcassets_build_id} /* Assets.xcassets in Resources */ = {{isa = PBXBuildFile; fileRef = {xcassets_ref} /* Assets.xcassets */; }};
/* End PBXBuildFile section */

/* Begin PBXFileReference section */
{file_refs}		{xcassets_ref} /* Assets.xcassets */ = {{isa = PBXFileReference; lastKnownFileType = folder.assetcatalog; path = Assets.xcassets; sourceTree = "<group>"; }};
		{info_plist_ref} /* Info.plist */ = {{isa = PBXFileReference; lastKnownFileType = text.plist.xml; path = {safe_name}/Info.plist; sourceTree = "<group>"; }};
		{product_ref} /* {safe_name}.app */ = {{isa = PBXFileReference; explicitFileType = wrapper.application; includeInIndex = 0; path = {safe_name}.app; sourceTree = BUILT_PRODUCTS_DIR; }};
/* End PBXFileReference section */

/* Begin PBXFrameworksBuildPhase section */
		{frameworks_phase} /* Frameworks */ = {{
			isa = PBXFrameworksBuildPhase;
			buildActionMask = 2147483647;
			files = (
			);
			runOnlyForDeploymentPostprocessing = 0;
		}};
/* End PBXFrameworksBuildPhase section */

/* Begin PBXGroup section */
		{main_group} = {{
			isa = PBXGroup;
			children = (
{group_children}			);
			sourceTree = "<group>";
		}};
		{products_group} /* Products */ = {{
			isa = PBXGroup;
			children = (
				{product_ref} /* {safe_name}.app */,
			);
			name = Products;
			sourceTree = "<group>";
		}};
/* End PBXGroup section */

/* Begin PBXNativeTarget section */
		{target_uuid} /* {safe_name} */ = {{
			isa = PBXNativeTarget;
			buildConfigurationList = {build_cfg_list} /* Build configuration list for PBXNativeTarget "{safe_name}" */;
			buildPhases = (
				{sources_phase} /* Sources */,
				{frameworks_phase} /* Frameworks */,
				{resources_phase} /* Resources */,
			);
			buildRules = (
			);
			dependencies = (
			);
			name = {safe_name};
			productName = {safe_name};
			productReference = {product_ref} /* {safe_name}.app */;
			productType = "com.apple.product-type.application";
		}};
/* End PBXNativeTarget section */

/* Begin PBXProject section */
		{project_uuid} /* Project object */ = {{
			isa = PBXProject;
			attributes = {{
				BuildIndependentTargetsInParallel = 1;
				LastSwiftUpdateCheck = 1500;
				LastUpgradeCheck = 1500;
				TargetAttributes = {{
					{target_uuid} = {{
						CreatedOnToolsVersion = 15.0;
						DevelopmentTeam = {team_id};
					}};
				}};
			}};
			buildConfigurationList = {build_cfg_list};
			compatibilityVersion = "Xcode 14.0";
			developmentRegion = en;
			hasScannedForEncodings = 0;
			knownRegions = (
				en,
				Base,
			);
			mainGroup = {main_group};
			productRefGroup = {products_group} /* Products */;
			projectDirPath = "";
			projectRoot = "";
			targets = (
				{target_uuid} /* {safe_name} */,
			);
		}};
/* End PBXProject section */

/* Begin PBXResourcesBuildPhase section */
		{resources_phase} /* Resources */ = {{
			isa = PBXResourcesBuildPhase;
			buildActionMask = 2147483647;
			files = (
				{xcassets_build_id} /* Assets.xcassets in Resources */,
			);
			runOnlyForDeploymentPostprocessing = 0;
		}};
/* End PBXResourcesBuildPhase section */

/* Begin PBXSourcesBuildPhase section */
		{sources_phase} /* Sources */ = {{
			isa = PBXSourcesBuildPhase;
			buildActionMask = 2147483647;
			files = (
{sources_list}			);
			runOnlyForDeploymentPostprocessing = 0;
		}};
/* End PBXSourcesBuildPhase section */

/* Begin XCBuildConfiguration section */
		{debug_cfg} /* Debug */ = {{
			isa = XCBuildConfiguration;
			buildSettings = {{
				ALWAYS_SEARCH_USER_PATHS = NO;
				ASSETCATALOG_COMPILER_GENERATE_SWIFT_ASSET_SYMBOL_EXTENSIONS = YES;
				CLANG_ANALYZER_NONNULL = YES;
				CODE_SIGN_STYLE = Automatic;
				CURRENT_PROJECT_VERSION = 1;
				DEVELOPMENT_TEAM = {team_id};
				GENERATE_INFOPLIST_FILE = NO;
				INFOPLIST_FILE = {safe_name}/Info.plist;
				IPHONEOS_DEPLOYMENT_TARGET = {min_ios};
				LD_RUNPATH_SEARCH_PATHS = (
					"$(inherited)",
					"@executable_path/Frameworks",
				);
				MARKETING_VERSION = 1.0;
				PRODUCT_BUNDLE_IDENTIFIER = {bundle_id};
				PRODUCT_NAME = "$(TARGET_NAME)";
				SWIFT_EMIT_LOC_STRINGS = YES;
				SWIFT_VERSION = 5.0;
				TARGETED_DEVICE_FAMILY = "1,2";
			}};
			name = Debug;
		}};
		{release_cfg} /* Release */ = {{
			isa = XCBuildConfiguration;
			buildSettings = {{
				ALWAYS_SEARCH_USER_PATHS = NO;
				ASSETCATALOG_COMPILER_GENERATE_SWIFT_ASSET_SYMBOL_EXTENSIONS = YES;
				CLANG_ANALYZER_NONNULL = YES;
				CODE_SIGN_STYLE = Automatic;
				CURRENT_PROJECT_VERSION = 1;
				DEVELOPMENT_TEAM = {team_id};
				GENERATE_INFOPLIST_FILE = NO;
				INFOPLIST_FILE = {safe_name}/Info.plist;
				IPHONEOS_DEPLOYMENT_TARGET = {min_ios};
				LD_RUNPATH_SEARCH_PATHS = (
					"$(inherited)",
					"@executable_path/Frameworks",
				);
				MARKETING_VERSION = 1.0;
				PRODUCT_BUNDLE_IDENTIFIER = {bundle_id};
				PRODUCT_NAME = "$(TARGET_NAME)";
				SWIFT_EMIT_LOC_STRINGS = YES;
				SWIFT_VERSION = 5.0;
				TARGETED_DEVICE_FAMILY = "1,2";
			}};
			name = Release;
		}};
/* End XCBuildConfiguration section */

/* Begin XCConfigurationList section */
		{build_cfg_list} /* Build configuration list for PBXNativeTarget "{safe_name}" */ = {{
			isa = XCConfigurationList;
			buildConfigurations = (
				{debug_cfg} /* Debug */,
				{release_cfg} /* Release */,
			);
			defaultConfigurationIsVisible = 0;
			defaultConfigurationName = Release;
		}};
/* End XCConfigurationList section */
	}};
	rootObject = {project_uuid} /* Project object */;
}}
"#);

    OutputFile {
        path: format!("{safe_name}.xcodeproj/project.pbxproj"),
        content,
    }
}

/// Generate the .xcscheme file so Xcode knows how to build and run the app.
fn gen_xcscheme(config: &IosConfig) -> OutputFile {
    let safe_name = config.app_name.replace(' ', "");
    fn uuid(seed: u64) -> String {
        let a = seed.wrapping_mul(0x9e3779b97f4a7c15u64);
        let b = (seed ^ 0xdeadbeef) as u32;
        format!("{a:016X}{b:08X}")
    }
    let target_uuid = uuid(2);
    let content = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<Scheme
   LastUpgradeVersion = "1500"
   version = "1.7">
   <BuildAction
      parallelizeBuildables = "YES"
      buildImplicitDependencies = "YES">
      <BuildActionEntries>
         <BuildActionEntry
            buildForTesting = "YES"
            buildForRunning = "YES"
            buildForProfiling = "YES"
            buildForArchiving = "YES"
            buildForAnalyzing = "YES">
            <BuildableReference
               BuildableIdentifier = "primary"
               BlueprintIdentifier = "{target_uuid}"
               BuildableName = "{safe_name}.app"
               BlueprintName = "{safe_name}"
               ReferencedContainer = "container:{safe_name}.xcodeproj">
            </BuildableReference>
         </BuildActionEntry>
      </BuildActionEntries>
   </BuildAction>
   <TestAction
      buildConfiguration = "Debug"
      selectedDebuggerIdentifier = "Xcode.DebuggerFoundation.Debugger.LLDB"
      selectedLauncherIdentifier = "Xcode.DebuggerFoundation.Launcher.LLDB"
      shouldUseLaunchSchemeArgsEnv = "YES"
      shouldAutocreateTestPlan = "YES">
   </TestAction>
   <LaunchAction
      buildConfiguration = "Debug"
      selectedDebuggerIdentifier = "Xcode.DebuggerFoundation.Debugger.LLDB"
      selectedLauncherIdentifier = "Xcode.DebuggerFoundation.Launcher.LLDB"
      launchStyle = "0"
      useCustomWorkingDirectory = "NO"
      ignoresPersistentStateOnLaunch = "NO"
      debugDocumentVersioning = "YES"
      debugServiceExtension = "internal"
      allowLocationSimulation = "YES">
      <BuildableProductRunnable
         runnableDebuggingMode = "0">
         <BuildableReference
            BuildableIdentifier = "primary"
            BlueprintIdentifier = "{target_uuid}"
            BuildableName = "{safe_name}.app"
            BlueprintName = "{safe_name}"
            ReferencedContainer = "container:{safe_name}.xcodeproj">
         </BuildableReference>
      </BuildableProductRunnable>
   </LaunchAction>
   <ProfileAction
      buildConfiguration = "Release"
      shouldUseLaunchSchemeArgsEnv = "YES"
      savedToolIdentifier = ""
      useCustomWorkingDirectory = "NO"
      debugDocumentVersioning = "YES">
   </ProfileAction>
   <AnalyzeAction
      buildConfiguration = "Debug">
   </AnalyzeAction>
   <ArchiveAction
      buildConfiguration = "Release"
      revealArchiveInOrganizer = "YES">
   </ArchiveAction>
</Scheme>
"#);
    OutputFile {
        path: format!("{safe_name}.xcodeproj/xcshareddata/xcschemes/{safe_name}.xcscheme"),
        content,
    }
}

/// Generate xcworkspace/contents.xcworkspacedata so `xcodebuild -workspace` works.
fn gen_xcworkspace_data(config: &IosConfig) -> OutputFile {
    let safe_name = config.app_name.replace(' ', "");
    OutputFile {
        path: format!("{safe_name}.xcodeproj/project.xcworkspace/contents.xcworkspacedata"),
        content: format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<Workspace
   version = "1.0">
   <FileRef
      location = "self:{safe_name}.xcodeproj">
   </FileRef>
</Workspace>
"#),
    }
}

/// Generate LaunchScreen.storyboard — required by Info.plist UILaunchStoryboardName key.
/// Uses a minimal but ibtool-compatible format. If ibtool rejects it, the project
/// still opens fine in Xcode and the storyboard can be recreated via the UI.
fn gen_launch_screen_storyboard(safe_name: &str) -> OutputFile {
    OutputFile {
        // Written to the {AppName}/ subfolder, consistent with Info.plist and
        // referenced via INFOPLIST_FILE path in the Xcode build settings.
        path: format!("{safe_name}/LaunchScreen.storyboard"),
        // This is a minimal valid storyboard that Xcode 14+ ibtool accepts.
        content: r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<document type="com.apple.InterfaceBuilder3.CocoaTouch.Storyboard.XIB" version="3.0" toolsVersion="32700.99.1234" targetRuntime="AppleCocoa" propertyAccessControl="none" useAutolayout="YES" launchScreen="YES" useTraitCollections="YES" useSafeAreas="YES" colorMatched="YES" initialViewController="01J-lp-oVM">
    <device id="retina6_12" orientation="portrait" appearance="light"/>
    <dependencies>
        <deployment identifier="iOS"/>
        <plugIn identifier="com.apple.InterfaceBuilder.IBCocoaTouchPlugin" version="22684.4"/>
        <capability name="Safe area layout guides" minToolsVersion="9.0"/>
        <capability name="documents saved in the Xcode 8 format" minToolsVersion="8.0"/>
    </dependencies>
    <scenes>
        <scene sceneID="EHf-IW-A2E">
            <objects>
                <viewController id="01J-lp-oVM" sceneMemberID="viewController">
                    <view key="view" contentMode="scaleToFill" id="Ze5-6b-2t3">
                        <rect key="frame" x="0.0" y="0.0" width="393" height="852"/>
                        <autoresizingMask key="autoresizingMask" widthSizable="YES" heightSizable="YES"/>
                        <viewLayoutGuide key="safeAreaLayoutGuide" id="Bcu-3y-fUS"/>
                        <color key="backgroundColor" systemColor="systemBackgroundColor"/>
                    </view>
                </viewController>
                <placeholder placeholderIdentifier="IBFirstResponder" id="iYj-Kq-Ea1" userLabel="First Responder" sceneMemberID="firstResponder"/>
            </objects>
            <point key="canvasLocation" x="53" y="375"/>
        </scene>
    </scenes>
</document>
"#
        .to_string(),
    }
}

/// .gitignore for the generated iOS project.
fn gen_gitignore_ios() -> OutputFile {
    OutputFile {
        path: ".gitignore".to_string(),
        content: r#"# Generated by Frame framework
.DS_Store
/.build
/Pods
xcuserdata/
*.xccheckout
*.moved-aside
DerivedData
*.hmap
*.ipa
*.xcuserstate
*.xcscmblueprint
"#
        .to_string(),
    }
}

// ─── Feature detection helpers ────────────────────────────────────────────────

fn ios_uses_fetch(ast: &AST) -> bool {
    ast.functions.values().any(|f| stmts_use_fetch(&f.body))
        || ast.stores.values().any(|s| s.actions.values().any(|f| stmts_use_fetch(&f.body)))
}

fn stmts_use_fetch(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|s| matches!(s, Stmt::WaitFetch(_)))
}

fn ios_uses_call(ast: &AST, func_name: &str) -> bool {
    ast.functions.values().any(|f| stmts_use_call(&f.body, func_name))
}

fn stmts_use_call(stmts: &[Stmt], name: &str) -> bool {
    stmts.iter().any(|s| match s {
        Stmt::Call(c) | Stmt::Wait(c) => c.func == name,
        _ => false,
    })
}

// ─── Info.plist ───────────────────────────────────────────────────────────────

// ─── Info.plist ───────────────────────────────────────────────────────────────

/// Full Info.plist generator covering every iOS privacy key category.
#[allow(clippy::too_many_arguments)]
fn gen_info_plist_full(
    config: &IosConfig,
    uses_camera: bool,
    uses_audio_record: bool,
    uses_location: bool,
    uses_location_bg: bool,
    uses_notification: bool,
    uses_bluetooth: bool,
    uses_contacts: bool,
    uses_calendar: bool,
    uses_photos: bool,
    uses_health: bool,
    uses_speech: bool,
    uses_face_id: bool,
    uses_http: bool,
) -> OutputFile {
    let mut extras = String::new();

    // ── Camera & Media ─────────────────────────────────────────────────────────
    if uses_camera {
        extras.push_str("\t<key>NSCameraUsageDescription</key>\n\t<string>This app uses the camera to capture photos and videos.</string>\n");
        extras.push_str("\t<key>NSPhotoLibraryAddUsageDescription</key>\n\t<string>This app saves photos and videos to your library.</string>\n");
    }
    if uses_audio_record {
        extras.push_str("\t<key>NSMicrophoneUsageDescription</key>\n\t<string>This app uses the microphone to record audio.</string>\n");
    }
    if uses_photos {
        extras.push_str("\t<key>NSPhotoLibraryUsageDescription</key>\n\t<string>This app accesses your photo library.</string>\n");
    }

    // ── Location ───────────────────────────────────────────────────────────────
    if uses_location {
        extras.push_str("\t<key>NSLocationWhenInUseUsageDescription</key>\n\t<string>This app uses your location while in use.</string>\n");
    }
    if uses_location_bg {
        extras.push_str("\t<key>NSLocationAlwaysAndWhenInUseUsageDescription</key>\n\t<string>This app uses your location in the background.</string>\n");
        extras.push_str("\t<key>NSLocationAlwaysUsageDescription</key>\n\t<string>This app uses your location in the background.</string>\n");
    }

    // ── Contacts & Calendar ────────────────────────────────────────────────────
    if uses_contacts {
        extras.push_str("\t<key>NSContactsUsageDescription</key>\n\t<string>This app accesses your contacts.</string>\n");
    }
    if uses_calendar {
        extras.push_str("\t<key>NSCalendarsUsageDescription</key>\n\t<string>This app accesses your calendar.</string>\n");
        extras.push_str("\t<key>NSRemindersUsageDescription</key>\n\t<string>This app accesses your reminders.</string>\n");
    }

    // ── Bluetooth ──────────────────────────────────────────────────────────────
    if uses_bluetooth {
        extras.push_str("\t<key>NSBluetoothAlwaysUsageDescription</key>\n\t<string>This app uses Bluetooth to connect to nearby devices.</string>\n");
        extras.push_str("\t<key>NSBluetoothPeripheralUsageDescription</key>\n\t<string>This app uses Bluetooth peripherals.</string>\n");
    }

    // ── Health & Activity ──────────────────────────────────────────────────────
    if uses_health {
        extras.push_str("\t<key>NSHealthShareUsageDescription</key>\n\t<string>This app reads health and activity data.</string>\n");
        extras.push_str("\t<key>NSHealthUpdateUsageDescription</key>\n\t<string>This app writes health and activity data.</string>\n");
        extras.push_str("\t<key>NSMotionUsageDescription</key>\n\t<string>This app tracks motion and fitness activity.</string>\n");
    }

    // ── Speech Recognition ─────────────────────────────────────────────────────
    if uses_speech {
        extras.push_str("\t<key>NSSpeechRecognitionUsageDescription</key>\n\t<string>This app uses speech recognition.</string>\n");
    }

    // ── Face ID / Biometrics ───────────────────────────────────────────────────
    if uses_face_id {
        extras.push_str("\t<key>NSFaceIDUsageDescription</key>\n\t<string>This app uses Face ID for authentication.</string>\n");
    }

    // ── Network ────────────────────────────────────────────────────────────────
    if uses_http {
        extras.push_str("\t<key>NSAppTransportSecurity</key>\n\t<dict>\n\t\t<key>NSAllowsArbitraryLoads</key>\n\t\t<true/>\n\t</dict>\n");
    }

    OutputFile {
        path: format!("{}/Info.plist", config.app_name.replace(' ', "")),
        content: format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleIdentifier</key>
	<string>{bundle_id}</string>
	<key>CFBundleShortVersionString</key>
	<string>{version}</string>
	<key>CFBundleVersion</key>
	<string>{build}</string>
	<key>CFBundleName</key>
	<string>{name}</string>
	<key>UILaunchScreen</key>
	<dict/>
	<key>MinimumOSVersion</key>
	<string>{min_ios}</string>
{extras}</dict>
</plist>
"#,
            bundle_id = config.bundle_id,
            version   = config.version,
            build     = config.build_number,
            name      = config.app_name,
            min_ios   = config.min_ios,
        ),
    }
}

// ─── AppDelegate.swift ────────────────────────────────────────────────────────

fn gen_app_delegate(bundle_id: &str, app_name: &str, ast: &AST) -> OutputFile {
    let has_lifecycle = ast.on_launch.is_some()
        || ast.on_foreground.is_some()
        || ast.on_background.is_some();
    let lifecycle_call = if has_lifecycle {
        "        frameRegisterAppLifecycle()\n"
    } else {
        ""
    };
    OutputFile {
        path: "AppDelegate.swift".to_string(),
        content: format!(r#"// AppDelegate.swift — {app_name}
import UIKit

@main
class AppDelegate: UIResponder, UIApplicationDelegate {{

    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {{
        // Restore persisted store values on app launch
        FrameStoreRegistry.shared.restoreAll()
        // Wire :app {{ }} lifecycle hooks
{lifecycle_call}        // Call the project-level on_launch hook (if declared in :app {{ }})
        FrameAppLifecycle.shared.onLaunch()
        return true
    }}

    func application(
        _ application: UIApplication,
        configurationForConnecting connectingSceneSession: UISceneSession,
        options: UIScene.ConnectionOptions
    ) -> UISceneConfiguration {{
        return UISceneConfiguration(name: "Default Configuration", sessionRole: connectingSceneSession.role)
    }}
}}

// ─── App-level lifecycle dispatcher ──────────────────────────────────────────
/// Generated stores register restore closures here.
/// The :app {{ }} block overrides onLaunch / onForeground / onBackground.
class FrameStoreRegistry {{
    static let shared = FrameStoreRegistry()
    private var restoreClosures: [() -> Void] = []
    func register(restore: @escaping () -> Void) {{ restoreClosures.append(restore) }}
    func restoreAll() {{ restoreClosures.forEach {{ $0() }} }}
}}

class FrameAppLifecycle {{
    static let shared = FrameAppLifecycle()
    var onLaunchHandler:     (() -> Void)?
    var onForegroundHandler: (() -> Void)?
    var onBackgroundHandler: (() -> Void)?
    func onLaunch()     {{ onLaunchHandler?() }}
    func onForeground() {{ onForegroundHandler?() }}
    func onBackground() {{ onBackgroundHandler?() }}
}}
"#, app_name = app_name, lifecycle_call = lifecycle_call),
    }
}

// ─── SceneDelegate.swift ──────────────────────────────────────────────────────

fn gen_scene_delegate(bundle_id: &str) -> OutputFile {
    OutputFile {
        path: "SceneDelegate.swift".to_string(),
        content: r#"import UIKit

class SceneDelegate: UIResponder, UIWindowSceneDelegate {
    var window: UIWindow?

    func scene(_ scene: UIScene, willConnectTo session: UISceneSession,
               options connectionOptions: UIScene.ConnectionOptions) {
        guard let windowScene = scene as? UIWindowScene else { return }
        window = UIWindow(windowScene: windowScene)
        let nav = UINavigationController(rootViewController: MainViewController())
        window?.rootViewController = nav
        window?.makeKeyAndVisible()
    }

    // ── App foreground / background ───────────────────────────────────────────

    func sceneWillEnterForeground(_ scene: UIScene) {
        // App is about to become active (from background or cold start).
        FrameAppLifecycle.shared.onForeground()
    }

    func sceneDidEnterBackground(_ scene: UIScene) {
        // App moved to background — save state, pause work.
        FrameAppLifecycle.shared.onBackground()
    }

    func sceneDidBecomeActive(_ scene: UIScene) {
        // Scene became fully active (interruption ended or app foregrounded).
        // This is the complement to sceneWillResignActive.
    }

    func sceneWillResignActive(_ scene: UIScene) {
        // About to lose focus (e.g. phone call, notification banner).
        // Use for pausing in-progress work.
    }
}
"#.to_string(),
    }
}

// ─── Assets.xcassets ─────────────────────────────────────────────────────────

fn gen_assets_xcassets() -> OutputFile {
    OutputFile {
        path: "Assets.xcassets/Contents.json".to_string(),
        content: r#"{
  "info": { "author": "frame", "version": 1 }
}
"#.to_string(),
    }
}

// ─── Podfile ──────────────────────────────────────────────────────────────────

fn gen_podfile(config: &IosConfig, ast: &AST) -> OutputFile {
    let uses_maps   = ast.pages.iter().any(|p| p.children.iter().any(|c| c.kind == "map_view"))
        || ast.components.values().any(|c| c.children.iter().any(|n| n.kind == "map_view"));
    let uses_lottie = ast.pages.iter().any(|p| p.children.iter().any(|c| c.kind == "lottie"))
        || ast.components.values().any(|c| c.children.iter().any(|n| n.kind == "lottie"));
    let uses_video  = ast.pages.iter().any(|p| p.children.iter().any(|c| c.kind == "video_player"));

    let mut pods = String::new();
    if uses_maps   { pods.push_str("  pod 'GoogleMaps'\n"); }
    if uses_lottie { pods.push_str("  pod 'lottie-ios'\n"); }
    if uses_video  { pods.push_str("  pod 'MobileVLCKit'\n"); }

    let safe_name = config.app_name.replace(' ', "");
    OutputFile {
        path: "Podfile".to_string(),
        content: format!(r#"platform :ios, '{min_ios}'
use_frameworks!

target '{safe_name}' do
{pods}end
"#,
            min_ios   = config.min_ios,
        ),
    }
}

// ─── MainViewController.swift ─────────────────────────────────────────────────

fn gen_main_view_controller(ast: &AST, bundle_id: &str) -> OutputFile {
    let first_vc = ast.pages.first()
        .map(|p| format!("{}ViewController()", p.name))
        .unwrap_or_else(|| "UIViewController()".to_string());

    let push_setup: String = ast.pages.iter().skip(1).map(|p| {
        format!("        // register route: {} -> {}ViewController\n", p.route, p.name)
    }).collect();

    OutputFile {
        path: "MainViewController.swift".to_string(),
        content: format!(r#"import UIKit

class MainViewController: UIViewController {{
    override func viewDidLoad() {{
        super.viewDidLoad()
        let root = {first_vc}
        navigationController?.pushViewController(root, animated: false)
{push_setup}    }}
}}
"#),
    }
}

// ─── Per-page ViewController ──────────────────────────────────────────────────

fn gen_page_view_controller(page: &Page, _ast: &AST, bundle_id: &str) -> OutputFile {
    let state_props = gen_swift_state_vars(&page.state);

    // Build typed init params from page.params (explicit) or route path segments
    let route_param_list: Vec<(String, String)> = if !page.params.is_empty() {
        page.params.iter().map(|(n, t)| {
            let (swift_t, _) = swift_type_default(t);
            (n.clone(), swift_t)
        }).collect()
    } else {
        page.route.split('/').filter(|s| s.starts_with(':'))
            .map(|s| (s.trim_start_matches(':').to_string(), "String".to_string()))
            .collect()
    };

    // Swift stored properties + init
    let param_props = route_param_list.iter()
        .map(|(n, t)| format!("    var {n}: {t}?\n"))
        .collect::<String>();
    let init_params = if route_param_list.is_empty() {
        String::new()
    } else {
        let params_str = route_param_list.iter()
            .map(|(n, t)| format!("{n}: {t}? = nil"))
            .collect::<Vec<_>>().join(", ");
        let assigns = route_param_list.iter()
            .map(|(n, _)| format!("        self.{n} = {n}\n"))
            .collect::<String>();
        format!("\n    init({params_str}) {{\n        super.init(nibName: nil, bundle: nil)\n{assigns}    }}\n\n    required init?(coder: NSCoder) {{ fatalError(\"init(coder:) not supported\") }}\n")
    };

    let before_appear = page.before_enter.as_ref()
        .map(|e| format!("        {}()\n", emit_swift_expr(e)))
        .unwrap_or_default();

    let on_mount = page.on_mount.as_ref()
        .map(|e| format!("        {}()\n", emit_swift_expr(e)))
        .unwrap_or_default();

    let before_disappear = {
        let bl = page.before_leave.as_ref()
            .map(|e| format!("        {}()\n", emit_swift_expr(e)))
            .unwrap_or_default();
        let um = page.on_unmount.as_ref()
            .map(|e| format!("        {}()\n", emit_swift_expr(e)))
            .unwrap_or_default();
        bl + &um
    };

    let (fg_bg_props, fg_bg_setup, fg_bg_teardown) =
        if page.on_foreground.is_some() || page.on_background.is_some() {
            let fg_handler = page.on_foreground.as_ref()
                .map(|e| format!("    @objc private func _onForeground() {{ {}() }}\n", emit_swift_expr(e)))
                .unwrap_or_default();
            let bg_handler = page.on_background.as_ref()
                .map(|e| format!("    @objc private func _onBackground() {{ {}() }}\n", emit_swift_expr(e)))
                .unwrap_or_default();
            let setup_fg = if page.on_foreground.is_some() {
                "        NotificationCenter.default.addObserver(self, selector: #selector(_onForeground),\n            name: UIApplication.willEnterForegroundNotification, object: nil)\n"
            } else { "" };
            let setup_bg = if page.on_background.is_some() {
                "        NotificationCenter.default.addObserver(self, selector: #selector(_onBackground),\n            name: UIApplication.didEnterBackgroundNotification, object: nil)\n"
            } else { "" };
            let teardown = "        NotificationCenter.default.removeObserver(self)\n";
            (
                format!("{fg_handler}{bg_handler}"),
                format!("{setup_fg}{setup_bg}"),
                teardown.to_string(),
            )
        } else {
            (String::new(), String::new(), String::new())
    };

    let has_fg_bg = !fg_bg_setup.is_empty();
    let setup_ui: String = page.children.iter()
        .map(|n| emit_uikit_view(n, "view", 2))
        .collect();

    OutputFile {
        path: format!("{}ViewController.swift", page.name),
        content: format!(r#"import UIKit

/// Route: {route}
class {name}ViewController: UIViewController {{
{param_props}{state_props}{init_params}
    override func viewDidLoad() {{
        super.viewDidLoad()
        view.backgroundColor = .systemBackground
        title = "{title}"
        setupUI()
{fg_bg_setup}    }}

    override func viewWillAppear(_ animated: Bool) {{
        super.viewWillAppear(animated)
{before_appear}    }}

    override func viewDidAppear(_ animated: Bool) {{
        super.viewDidAppear(animated)
{on_mount}    }}

    override func viewDidDisappear(_ animated: Bool) {{
        super.viewDidDisappear(animated)
{before_disappear}    }}

    override func viewWillDisappear(_ animated: Bool) {{
        super.viewWillDisappear(animated)
    }}
{fg_bg_props}
    private func setupUI() {{
{setup_ui}    }}
{deinit_block}}}
"#,
            name       = page.name,
            route      = page.route,
            title      = page.name,
            param_props = param_props,
            init_params = init_params,
            deinit_block = if has_fg_bg {
                format!("\n    deinit {{\n{fg_bg_teardown}    }}\n")
            } else {
                String::new()
            },
        ),
    }
}

fn gen_swift_state_vars(state: &HashMap<String, StateField>) -> String {
    let mut lines: Vec<String> = state.values().map(|f| {
        let (swift_type, default) = swift_type_default(&f.type_);
        let val = f.default.as_ref().map(|e| emit_swift_expr(e)).unwrap_or_else(|| default.to_string());
        format!("    @Published var {}: {} = {}", f.name, swift_type, val)
    }).collect();
    lines.sort();
    lines.join("\n") + if lines.is_empty() { "" } else { "\n" }
}

// ─── Custom component UIView subclasses ───────────────────────────────────────

fn gen_component_view(name: &str, comp: &ComponentDef, bundle_id: &str) -> OutputFile {
    let props_params: String = if comp.props.is_empty() {
        String::new()
    } else {
        let mut params: Vec<String> = comp.props.values().map(|p| {
            let (t, d) = swift_type_default(&p.type_);
            format!("    var {}: {} = {}", p.name, t, d)
        }).collect();
        params.sort();
        params.join("\n")
    };

    let setup_code: String = comp.children.iter()
        .map(|n| emit_uikit_view(n, "self", 2))
        .collect();

    OutputFile {
        path: format!("{}View.swift", name),
        content: format!(r#"import UIKit

class {name}View: UIView {{
{props_params}

    override init(frame: CGRect) {{
        super.init(frame: frame)
        setupUI()
    }}
    required init?(coder: NSCoder) {{ fatalError() }}

    private func setupUI() {{
{setup_code}    }}
}}
"#),
    }
}

// ─── Store ObservableObject ───────────────────────────────────────────────────

fn gen_store_swift(name: &str, store: &StoreSlice, bundle_id: &str) -> OutputFile {
    let mut content = String::new();
    content.push_str("import Foundation\nimport Combine\n\n");
    content.push_str(&format!("class {}Store: ObservableObject {{\n", name));

    // @Published fields
    let mut sorted_fields: Vec<_> = store.fields.values().collect();
    sorted_fields.sort_by_key(|f| &f.name);
    for field in &sorted_fields {
        let (t, d) = swift_type_default(&field.type_);
        let val = field.default.as_ref().map(|e| emit_swift_expr(e)).unwrap_or_else(|| d.to_string());
        content.push_str(&format!("    @Published var {}: {} = {}\n", field.name, t, val));
    }
    content.push('\n');

    // init: restore persisted
    let local_fields: Vec<_>  = store.persist.iter().filter(|(_, s)| **s == PersistStrategy::Local).collect();
    let secure_fields: Vec<_> = store.persist.iter().filter(|(_, s)| **s == PersistStrategy::Secure).collect();

    content.push_str("    init() {\n");
    // Register with FrameStoreRegistry so AppDelegate can trigger restore on launch
    content.push_str(&format!(
        "        FrameStoreRegistry.shared.register {{ [weak self] in self?.restorePersistedFields() }}\n"
    ));
    content.push_str("        restorePersistedFields()\n    }\n\n");
    content.push_str("    private func restorePersistedFields() {\n");
    if !local_fields.is_empty() {
        for (fname, _) in &local_fields {
            content.push_str(&format!(
                "        if let v = UserDefaults.standard.string(forKey: \"{fname}\") {{ {fname} = v }}\n"
            ));
        }
    }
    if !secure_fields.is_empty() {
        for (fname, _) in &secure_fields {
            content.push_str(&format!(
                "        if let v = KeychainHelper.read(key: \"{fname}\") {{ {fname} = v }}\n"
            ));
        }
    }
    content.push_str("    }\n\n");

    // Actions
    let mut sorted_actions: Vec<_> = store.actions.values().collect();
    sorted_actions.sort_by_key(|a| &a.name);
    for action in &sorted_actions {
        let params_str = action.params.iter()
            .map(|(pn, pt, default)| {
                let (t, _) = swift_type_default(pt);
                match default {
                    Some(d) => format!("{pn}: {t} = {}", emit_swift_expr(d)),
                    None => format!("{pn}: {t}"),
                }
            })
            .collect::<Vec<_>>().join(", ");
        let body: String = action.body.iter().map(|s| emit_swift_stmt(s, 2)).collect();
        if action.is_async {
            content.push_str(&format!("    func {}({}) async {{\n", action.name, params_str));
            content.push_str("        await MainActor.run {\n");
            content.push_str(&body);
            content.push_str("        }\n    }\n\n");
        } else {
            content.push_str(&format!("    func {}({}) {{\n", action.name, params_str));
            content.push_str(&body);
            content.push_str("    }\n\n");
        }
    }

    content.push('}');
    content.push('\n');

    OutputFile { path: format!("{}Store.swift", name), content }
}

// ─── Platform feature helpers ─────────────────────────────────────────────────

fn gen_camera_helper_swift(bundle_id: &str) -> OutputFile {
    OutputFile {
        path: "CameraHelper.swift".to_string(),
        content: r#"import UIKit

class CameraHelper: NSObject, UIImagePickerControllerDelegate, UINavigationControllerDelegate {
    var onCapture: ((UIImage?) -> Void)?

    func capture(from vc: UIViewController, onCapture: @escaping (UIImage?) -> Void) {
        self.onCapture = onCapture
        let picker = UIImagePickerController()
        picker.sourceType = .camera
        picker.delegate = self
        vc.present(picker, animated: true)
    }

    func imagePickerController(_ picker: UIImagePickerController,
                                didFinishPickingMediaWithInfo info: [UIImagePickerController.InfoKey: Any]) {
        let image = info[.originalImage] as? UIImage
        picker.dismiss(animated: true)
        onCapture?(image)
    }
    func imagePickerControllerDidCancel(_ picker: UIImagePickerController) {
        picker.dismiss(animated: true)
        onCapture?(nil)
    }
}
"#.to_string(),
    }
}

fn gen_location_helper_swift(bundle_id: &str) -> OutputFile {
    OutputFile {
        path: "LocationHelper.swift".to_string(),
        content: r#"import CoreLocation

class LocationHelper: NSObject, CLLocationManagerDelegate {
    private let manager = CLLocationManager()
    var onLocation: ((Double, Double) -> Void)?

    func getLocation(onLocation: @escaping (Double, Double) -> Void) {
        self.onLocation = onLocation
        manager.delegate = self
        manager.requestWhenInUseAuthorization()
        manager.startUpdatingLocation()
    }

    func locationManager(_ manager: CLLocationManager, didUpdateLocations locations: [CLLocation]) {
        guard let loc = locations.last else { return }
        onLocation?(loc.coordinate.latitude, loc.coordinate.longitude)
        manager.stopUpdatingLocation()
    }
}
"#.to_string(),
    }
}

fn gen_notification_helper_swift(bundle_id: &str) -> OutputFile {
    OutputFile {
        path: "NotificationHelper.swift".to_string(),
        content: r#"import UserNotifications

struct NotificationHelper {
    static func send(title: String, message: String, identifier: String = UUID().uuidString) {
        UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound, .badge]) { granted, _ in
            guard granted else { return }
            let content = UNMutableNotificationContent()
            content.title = title
            content.body = message
            let trigger = UNTimeIntervalNotificationTrigger(timeInterval: 0.1, repeats: false)
            let request = UNNotificationRequest(identifier: identifier, content: content, trigger: trigger)
            UNUserNotificationCenter.current().add(request)
        }
    }
}
"#.to_string(),
    }
}

// ─── UIKit view emitter ───────────────────────────────────────────────────────

/// Emit UIKit code that adds a component node to a parent view.
pub fn emit_uikit_view(node: &ComponentNode, parent: &str, indent: usize) -> String {
    let pad  = "    ".repeat(indent);
    let i1   = "    ".repeat(indent + 1);
    let body = match node.kind.as_str() {
        "text"             => emit_ios_text(node, parent, &pad, &i1),
        "button"           => emit_ios_button(node, parent, &pad, &i1),
        "image"            => emit_ios_image(node, parent, &pad, &i1),
        "icon"             => emit_ios_icon(node, parent, &pad, &i1),
        "row"              => emit_ios_stack(node, parent, &pad, &i1, indent, "horizontal"),
        "column"           => emit_ios_stack(node, parent, &pad, &i1, indent, "vertical"),
        "container"        => emit_ios_container(node, parent, &pad, &i1, indent),
        "stack"            => emit_ios_stack_absolute(node, parent, &pad, &i1, indent),
        "list"             => emit_ios_table_view(node, parent, &pad, &i1),
        "input"            => emit_ios_text_field(node, parent, &pad),
        "dropdown"         => emit_ios_picker(node, parent, &pad, &i1, indent),
        "form"             => emit_ios_stack(node, parent, &pad, &i1, indent, "vertical"),
        "app_bar"          => emit_ios_navigation_bar(node, parent, &pad, indent),
        "bottom_navigation_bar" => emit_ios_tab_bar(node, parent, &pad, &i1, indent),
        "scaffold"         => emit_ios_scaffold(node, parent, &pad, &i1, indent),
        "card"             => emit_ios_card(node, parent, &pad, &i1, indent),
        "divider"          => emit_ios_divider(parent, &pad),
        "spacer"           => emit_ios_spacer(node, parent, &pad),
        "modal"            => emit_ios_alert(node, parent, &pad),
        "scroll_view"      => emit_ios_scroll_view(node, parent, &pad, &i1, indent),
        "grid"             => emit_ios_collection_view(node, parent, &pad),
        // ── Feedback ──────────────────────────────────────────────────────
        "toast"            => emit_ios_toast(node, parent, &pad),
        "tooltip"          => emit_ios_tooltip(node, parent, &pad),
        "badge"            => emit_ios_badge(node, parent, &pad, &i1, indent),
        "progress_bar"     => emit_ios_progress_bar(node, parent, &pad),
        "progress_circle"  => emit_ios_progress_circle(node, parent, &pad),
        // ── Navigation ────────────────────────────────────────────────────
        "tab_bar"          => emit_ios_tab_bar_controller(node, parent, &pad, &i1, indent),
        "tab"              => emit_ios_tab_item(node, parent, &pad),
        "sidebar"          => emit_ios_sidebar(node, parent, &pad, &i1, indent),
        "floating_action_button" => emit_ios_fab(node, parent, &pad, indent),
        "bottom_sheet"     => emit_ios_bottom_sheet(node, parent, &pad, &i1, indent),
        // ── Inputs ────────────────────────────────────────────────────────
        "switch"           => emit_ios_switch(node, parent, &pad),
        "checkbox"         => emit_ios_checkbox(node, parent, &pad),
        "radio"            => emit_ios_radio(node, parent, &pad),
        "slider"           => emit_ios_slider(node, parent, &pad),
        "stepper"          => emit_ios_stepper(node, parent, &pad),
        "text_area"        => emit_ios_text_view(node, parent, &pad),
        "search_bar"       => emit_ios_search_bar(node, parent, &pad),
        "date_picker"      => emit_ios_date_picker(node, parent, &pad),
        "time_picker"      => emit_ios_time_picker(node, parent, &pad),
        "color_picker"     => emit_ios_color_picker(node, parent, &pad),
        "rating"           => emit_ios_rating(node, parent, &pad),
        "otp_input"        => emit_ios_otp_input(node, parent, &pad),
        // ── Display ───────────────────────────────────────────────────────
        "avatar"           => emit_ios_avatar(node, parent, &pad),
        "chip"             => emit_ios_chip(node, parent, &pad),
        "tag"              => emit_ios_tag(node, parent, &pad),
        "banner"           => emit_ios_banner(node, parent, &pad, &i1, indent),
        "table"            => emit_ios_table_view(node, parent, &pad, &i1),
        "accordion"        => emit_ios_accordion(node, parent, &pad, &i1, indent),
        "timeline"         => emit_ios_timeline(node, parent, &pad, &i1, indent),
        "skeleton"         => emit_ios_skeleton(node, parent, &pad),
        // ── Media ─────────────────────────────────────────────────────────
        "video_player"     => emit_ios_video_player(node, parent, &pad),
        "audio_player"     => emit_ios_audio_player(node, parent, &pad),
        "lottie"           => emit_ios_lottie(node, parent, &pad),
        "web_view"         => emit_ios_web_view(node, parent, &pad),
        "map_view"         => emit_ios_map_view(node, parent, &pad),
        "camera_view"      => emit_ios_camera_view(node, parent, &pad),
        "qr_scanner"       => emit_ios_qr_scanner(node, parent, &pad),
        // ── Misc ───────────────────────────────────────────────────────────
        "item"             => emit_ios_stack(node, parent, &pad, &i1, indent, "vertical"),
        "plugin"           => emit_ios_plugin(node, parent, &pad),
        // ── Gestures ──────────────────────────────────────────────────────
        "swipeable"        => emit_ios_swipeable(node, parent, &pad, &i1, indent),
        "draggable"        => emit_ios_draggable(node, parent, &pad, &i1, indent),
        "refresh"          => emit_ios_refresh(node, parent, &pad),
        "long_press"       => emit_ios_long_press(node, parent, &pad, &i1, indent),
        _ => {
            // User-defined custom component
            let var = to_var_name(&node.kind);
            let props_str = node.props.iter()
                .map(|(k, v)| format!("{}: {}", k, emit_swift_expr(v)))
                .collect::<Vec<_>>().join(", ");
            format!("{pad}let {var} = {}View({props_str})\n\
                     {pad}{parent}.addSubview({var})\n",
                    node.kind)
        }
    };

    // show_if wrapper
    if let Some(cond) = &node.show_if {
        let cond_str = emit_swift_expr(cond);
        format!("{pad}if {cond_str} {{\n{body}{pad}}}\n")
    } else {
        // Component-level lifecycle hooks emitted after view is added to hierarchy.
        // on_mount  → DispatchQueue.main.async (runs after viewDidLoad returns)
        // on_unmount → not directly expressible in UIKit here; documented as a comment.
        // on_update  → DispatchQueue.main.async with key comment for dependency.
        let mount_code = if let Some(h) = &node.events.on_mount {
            format!("{pad}DispatchQueue.main.async {{ {}() }} // on_mount\n", emit_swift_expr(h))
        } else {
            String::new()
        };
        let update_code = if let Some(h) = &node.events.on_update {
            let key = node.events.watch.as_ref()
                .map(|w| emit_swift_expr(w))
                .unwrap_or_else(|| "\"\"".to_string());
            format!("{pad}// on_update(watch: {key}) — wire to your observation pattern\n\
                     {pad}DispatchQueue.main.async {{ {}() }}\n", emit_swift_expr(h))
        } else {
            String::new()
        };
        let unmount_code = if let Some(h) = &node.events.on_unmount {
            format!("{pad}// on_unmount — call {}() in deinit or viewDidDisappear\n", emit_swift_expr(h))
        } else {
            String::new()
        };
        format!("{body}{mount_code}{update_code}{unmount_code}")
    }
}

// ─── Original 21 component emitters ──────────────────────────────────────────

fn emit_ios_text(node: &ComponentNode, parent: &str, pad: &str, _i1: &str) -> String {
    let text = node.props.get("content").or_else(|| node.props.get("text"))
        .map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let var = fresh_var("label");
    let color = node.styles.color.as_deref().map(|c| format!("\n{pad}{var}.textColor = UIColor(hex: \"{c}\")")).unwrap_or_default();
    let font_size = node.styles.font_size.as_deref().and_then(parse_sp)
        .map(|s| format!("\n{pad}{var}.font = UIFont.systemFont(ofSize: {s})")).unwrap_or_default();
    let max_lines = node.styles.line_clamp.or(node.styles.max_lines)
        .map(|n| format!("\n{pad}{var}.numberOfLines = {n}")).unwrap_or_default();
    let overflow = match node.styles.text_overflow {
        TextOverflowValue::Ellipsis => format!("\n{pad}{var}.lineBreakMode = .byTruncatingTail"),
        TextOverflowValue::Clip     => format!("\n{pad}{var}.lineBreakMode = .byClipping"),
        _                           => String::new(),
    };
    format!("{pad}let {var} = UILabel()\n\
             {pad}{var}.text = {text}{color}{font_size}{max_lines}{overflow}\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_button(node: &ComponentNode, parent: &str, pad: &str, _i1: &str) -> String {
    let title = node.props.get("content").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let on_click = node.events.on_click.as_ref().map(|e| emit_swift_expr(e)).unwrap_or_else(|| "{}".to_string());
    let var = fresh_var("button");
    format!("{pad}let {var} = UIButton(type: .system)\n\
             {pad}{var}.setTitle({title}, for: .normal)\n\
             {pad}{var}.addAction(UIAction {{ _ in {on_click} }}, for: .touchUpInside)\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_image(node: &ComponentNode, parent: &str, pad: &str, _i1: &str) -> String {
    let src = node.props.get("src").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let content_mode = match node.styles.image_fit {
        ImageFitValue::Cover     => "scaleAspectFill",
        ImageFitValue::Fill      => "scaleToFill",
        ImageFitValue::None_     => "center",
        ImageFitValue::ScaleDown => "topLeft",
        ImageFitValue::Contain   => "scaleAspectFit",
    };
    let var = fresh_var("imageView");
    format!("{pad}let {var} = UIImageView()\n\
             {pad}{var}.contentMode = .{content_mode}\n\
             {pad}{var}.clipsToBounds = true\n\
             {pad}// load image from: {src}\n\
             {pad}if let url = URL(string: {src}) {{\n\
             {pad}    URLSession.shared.dataTask(with: url) {{ data, _, _ in\n\
             {pad}        if let data = data {{ DispatchQueue.main.async {{ {var}.image = UIImage(data: data) }} }}\n\
             {pad}    }}.resume()\n\
             {pad}}}\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_icon(node: &ComponentNode, parent: &str, pad: &str, _i1: &str) -> String {
    let icon = node.props.get("icon").map(|e| emit_swift_expr(e))
        .unwrap_or_else(|| "\"star\"".to_string());
    let var = fresh_var("iconView");
    format!("{pad}let {var} = UIImageView(image: UIImage(systemName: {icon}))\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_stack(node: &ComponentNode, parent: &str, pad: &str, _i1: &str, indent: usize, axis: &str) -> String {
    let var = fresh_var("stack");
    let children: String = node.children.iter().map(|c| emit_uikit_view(c, var.as_str(), indent + 1)).collect();
    format!("{pad}let {var} = UIStackView()\n\
             {pad}{var}.axis = .{axis}\n\
             {pad}{var}.distribution = .fill\n\
             {pad}{parent}.addSubview({var})\n{children}")
}

fn emit_ios_container(node: &ComponentNode, parent: &str, pad: &str, _i1: &str, indent: usize) -> String {
    let var = fresh_var("container");
    let overflow_props = emit_ios_overflow_props(&node.styles, &var, pad);
    let radius = node.styles.border_radius.as_deref().and_then(parse_dp_str)
        .map(|r| format!("\n{pad}{var}.layer.cornerRadius = {r}")).unwrap_or_default();
    let children: String = node.children.iter().map(|c| emit_uikit_view(c, var.as_str(), indent + 1)).collect();
    format!("{pad}let {var} = UIView(){radius}\n\
             {pad}{parent}.addSubview({var})\n{overflow_props}{children}")
}

fn emit_ios_stack_absolute(node: &ComponentNode, parent: &str, pad: &str, _i1: &str, indent: usize) -> String {
    let var = fresh_var("stackView");
    let alignment = ios_stack_alignment(&node.alignment);
    let children: String = node.children.iter().map(|c| {
        if let Some(pos) = &c.positioned {
            let cv = fresh_var("positioned");
            let child_code = emit_uikit_view(c, cv.as_str(), indent + 1);
            let top    = pos.top.as_deref().and_then(parse_dp_str).map(|v| format!("{pad}    {cv}.topAnchor.constraint(equalTo: {var}.topAnchor, constant: {v}).isActive = true\n")).unwrap_or_default();
            let left   = pos.left.as_deref().and_then(parse_dp_str).map(|v| format!("{pad}    {cv}.leadingAnchor.constraint(equalTo: {var}.leadingAnchor, constant: {v}).isActive = true\n")).unwrap_or_default();
            let bottom = pos.bottom.as_deref().and_then(parse_dp_str).map(|v| format!("{pad}    {cv}.bottomAnchor.constraint(equalTo: {var}.bottomAnchor, constant: -{v}).isActive = true\n")).unwrap_or_default();
            let right  = pos.right.as_deref().and_then(parse_dp_str).map(|v| format!("{pad}    {cv}.trailingAnchor.constraint(equalTo: {var}.trailingAnchor, constant: -{v}).isActive = true\n")).unwrap_or_default();
            let w      = pos.width.as_deref().and_then(parse_dp_str).map(|v| format!("{pad}    {cv}.widthAnchor.constraint(equalToConstant: {v}).isActive = true\n")).unwrap_or_default();
            let h      = pos.height.as_deref().and_then(parse_dp_str).map(|v| format!("{pad}    {cv}.heightAnchor.constraint(equalToConstant: {v}).isActive = true\n")).unwrap_or_default();
            format!("{pad}let {cv} = UIView()\n{pad}{var}.addSubview({cv})\n{pad}{cv}.translatesAutoresizingMaskIntoConstraints = false\n{child_code}{top}{left}{bottom}{right}{w}{h}")
        } else {
            emit_uikit_view(c, var.as_str(), indent + 1)
        }
    }).collect();
    format!("{pad}let {var} = UIView() // stack — {alignment}\n\
             {pad}{parent}.addSubview({var})\n{children}")
}

fn emit_ios_table_view(node: &ComponentNode, parent: &str, pad: &str, _i1: &str) -> String {
    let var = fresh_var("tableView");
    let data = node.data.as_ref().map(|e| emit_swift_expr(e)).unwrap_or_else(|| "[]".to_string());
    format!("{pad}let {var} = UITableView()\n\
             {pad}// data: {data}\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_text_field(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let placeholder = node.props.get("placeholder").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let var = fresh_var("textField");
    format!("{pad}let {var} = UITextField()\n\
             {pad}{var}.placeholder = {placeholder}\n\
             {pad}{var}.borderStyle = .roundedRect\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_picker(node: &ComponentNode, parent: &str, pad: &str, _i1: &str, indent: usize) -> String {
    let var = fresh_var("picker");
    format!("{pad}let {var} = UIPickerView()\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_navigation_bar(node: &ComponentNode, parent: &str, pad: &str, indent: usize) -> String {
    let var = fresh_var("navBar");
    let title = node.props.get("title").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let leading = node.props.get("leading").map(|e| emit_swift_expr(e)).unwrap_or_default();
    let leading_block = if !leading.is_empty() && leading != "\"\"" {
        format!("\n{pad}let {var}Leading = UIBarButtonItem(image: UIImage(systemName: {leading}), style: .plain, target: self, action: nil)\n{pad}navigationItem.leftBarButtonItem = {var}Leading")
    } else {
        String::new()
    };
    let actions: String = node.children.iter()
        .map(|c| emit_uikit_action(c, var.as_str(), pad, indent + 1))
        .collect();
    let actions_block = if actions.is_empty() {
        String::new()
    } else {
        format!("\n{pad}navigationItem.rightBarButtonItems = [{actions}]")
    };
    format!(
        "{pad}navigationItem.title = {title}\
         {leading_block}\
         {actions_block}\n"
    )
}

fn emit_uikit_action(node: &ComponentNode, _parent: &str, pad: &str, indent: usize) -> String {
    // Emit an icon as a UIBarButtonItem for the navigation bar actions
    let icon = node.props.get("name")
        .or_else(|| node.props.get("icon"))
        .map(|e| emit_swift_expr(e))
        .unwrap_or_default();
    let on_click = node.events.on_click.as_ref()
        .map(|e| emit_swift_expr(e))
        .unwrap_or_default();
    let action_sel = if on_click.is_empty() {
        "nil".to_string()
    } else {
        format!("#selector({on_click})")
    };
    let i_pad = "    ".repeat(indent);
    if !icon.is_empty() && icon != "\"\"" {
        format!("\n{i_pad}UIBarButtonItem(image: UIImage(systemName: {icon}), style: .plain, target: self, action: {action_sel}),")
    } else {
        let text = node.props.get("content")
            .map(|e| emit_swift_expr(e))
            .unwrap_or_else(|| "\"\"".to_string());
        format!("\n{i_pad}UIBarButtonItem(title: {text}, style: .plain, target: self, action: {action_sel}),")
    }
}

fn emit_ios_tab_bar(node: &ComponentNode, parent: &str, pad: &str, _i1: &str, indent: usize) -> String {
    let var = fresh_var("tabBar");
    format!("{pad}let {var} = UITabBar()\n{pad}{parent}.addSubview({var})\n")
}

fn emit_ios_scaffold(node: &ComponentNode, parent: &str, pad: &str, _i1: &str, indent: usize) -> String {
    let var = fresh_var("scaffold");
    let use_safe = node.styles.safe_area.unwrap_or(true);
    let anchor = if use_safe {
        format!("{}.safeAreaLayoutGuide", parent)
    } else {
        parent.to_string()
    };
    let children: String = node.children.iter().map(|c| emit_uikit_view(c, var.as_str(), indent + 1)).collect();
    format!(
        "{pad}let {var} = UIView()\n\
         {pad}{var}.translatesAutoresizingMaskIntoConstraints = false\n\
         {pad}{parent}.addSubview({var})\n\
         {pad}NSLayoutConstraint.activate([\n\
         {pad}    {var}.topAnchor.constraint(equalTo: {anchor}.topAnchor),\n\
         {pad}    {var}.leadingAnchor.constraint(equalTo: {anchor}.leadingAnchor),\n\
         {pad}    {var}.trailingAnchor.constraint(equalTo: {anchor}.trailingAnchor),\n\
         {pad}    {var}.bottomAnchor.constraint(equalTo: {anchor}.bottomAnchor),\n\
         {pad}])\n{children}"
    )
}

fn emit_ios_sidebar(node: &ComponentNode, parent: &str, pad: &str, _i1: &str, indent: usize) -> String {
    let var = fresh_var("sidebar");
    let width = node.props.get("width")
        .and_then(|e| if let Expr::Literal(Value::Str(s)) = e { Some(s.clone()) } else { None })
        .unwrap_or_else(|| "260".to_string());
    let children: String = node.children.iter().map(|c| emit_uikit_view(c, var.as_str(), indent + 1)).collect();
    format!(
        "{pad}let {var} = UIStackView()\n\
         {pad}{var}.axis = .vertical\n\
         {pad}{var}.translatesAutoresizingMaskIntoConstraints = false\n\
         {pad}{parent}.addSubview({var})\n\
         {pad}NSLayoutConstraint.activate([\n\
         {pad}    {var}.topAnchor.constraint(equalTo: {parent}.topAnchor),\n\
         {pad}    {var}.leadingAnchor.constraint(equalTo: {parent}.leadingAnchor),\n\
         {pad}    {var}.widthAnchor.constraint(equalToConstant: {width}),\n\
         {pad}    {var}.bottomAnchor.constraint(equalTo: {parent}.bottomAnchor),\n\
         {pad}])\n\
         {children}"
    )
}

fn emit_ios_fab(node: &ComponentNode, parent: &str, pad: &str, indent: usize) -> String {
    let var = fresh_var("fab");
    let content = node.props.get("content")
        .map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let icon = node.props.get("icon")
        .map(|e| emit_swift_expr(e)).unwrap_or_default();
    let on_click = node.events.on_click.as_ref()
        .map(|e| emit_swift_expr(e)).unwrap_or_default();
    let tap_handler = if on_click.is_empty() {
        String::new()
    } else {
        format!("\n{pad}{var}.addTarget(self, action: #selector({on_click}), for: .touchUpInside)")
    };
    // If children exist, use them as the FAB content (e.g. icon component)
    if !node.children.is_empty() {
        let content_var = fresh_var("fabContent");
        let children: String = node.children.iter()
            .map(|c| emit_uikit_view(c, content_var.as_str(), indent + 2))
            .collect();
        format!(
            "{pad}let {var} = UIView()\n\
             {pad}{var}.backgroundColor = UIColor(hex: \"#007AFF\")\n\
             {pad}{var}.layer.cornerRadius = 28\n\
             {pad}{var}.translatesAutoresizingMaskIntoConstraints = false\n\
             {pad}{parent}.addSubview({var})\n\
             {pad}let {content_var} = UIStackView()\n\
             {pad}{content_var}.axis = .horizontal\n\
             {pad}{content_var}.alignment = .center\n\
             {pad}{content_var}.translatesAutoresizingMaskIntoConstraints = false\n\
             {pad}{var}.addSubview({content_var})\n\
             {pad}NSLayoutConstraint.activate([\n\
             {pad}    {content_var}.centerXAnchor.constraint(equalTo: {var}.centerXAnchor),\n\
             {pad}    {content_var}.centerYAnchor.constraint(equalTo: {var}.centerYAnchor),\n\
             {pad}])\
             {tap_handler}\n{children}\
             {pad}NSLayoutConstraint.activate([\n\
             {pad}    {var}.trailingAnchor.constraint(equalTo: {parent}.trailingAnchor, constant: -16),\n\
             {pad}    {var}.bottomAnchor.constraint(equalTo: {parent}.bottomAnchor, constant: -16),\n\
             {pad}    {var}.widthAnchor.constraint(equalToConstant: 56),\n\
             {pad}    {var}.heightAnchor.constraint(equalToConstant: 56),\n\
             {pad}])\n"
        )
    } else {
        let title = if !content.is_empty() && content != "\"\"" {
            format!("\n{pad}{var}.setTitle({content}, for: .normal)")
        } else {
            String::new()
        };
        let icon_setup = if !icon.is_empty() && icon != "\"\"" {
            format!("\n{pad}{var}.setImage(UIImage(systemName: {icon}), for: .normal)")
        } else if content.is_empty() || content == "\"\"" {
            "\n{pad}{var}.setImage(UIImage(systemName: \"plus\"), for: .normal)".to_string()
        } else {
            String::new()
        };
        format!(
            "{pad}let {var} = UIButton(type: .system)\n\
             {pad}{var}.backgroundColor = UIColor(hex: \"#007AFF\")\n\
             {pad}{var}.tintColor = .white\n\
             {pad}{var}.layer.cornerRadius = 28\n\
             {pad}{var}.translatesAutoresizingMaskIntoConstraints = false\
             {title}{icon_setup}{tap_handler}\n\
             {pad}{parent}.addSubview({var})\n\
             {pad}NSLayoutConstraint.activate([\n\
             {pad}    {var}.trailingAnchor.constraint(equalTo: {parent}.trailingAnchor, constant: -16),\n\
             {pad}    {var}.bottomAnchor.constraint(equalTo: {parent}.bottomAnchor, constant: -16),\n\
             {pad}    {var}.widthAnchor.constraint(equalToConstant: 56),\n\
             {pad}    {var}.heightAnchor.constraint(equalToConstant: 56),\n\
             {pad}])\n"
        )
    }
}

fn emit_ios_card(node: &ComponentNode, parent: &str, pad: &str, _i1: &str, indent: usize) -> String {
    let var = fresh_var("card");
    let children: String = node.children.iter().map(|c| emit_uikit_view(c, var.as_str(), indent + 1)).collect();
    format!("{pad}let {var} = UIView()\n\
             {pad}{var}.layer.cornerRadius = 8\n\
             {pad}{var}.layer.shadowColor = UIColor.black.cgColor\n\
             {pad}{var}.layer.shadowOpacity = 0.15\n\
             {pad}{var}.layer.shadowRadius = 4\n\
             {pad}{parent}.addSubview({var})\n{children}")
}

fn emit_ios_divider(parent: &str, pad: &str) -> String {
    let var = fresh_var("divider");
    format!("{pad}let {var} = UIView()\n\
             {pad}{var}.backgroundColor = .separator\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_spacer(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let var = fresh_var("spacer");
    let h = node.styles.height.as_deref().and_then(parse_dp_str).unwrap_or(8.0);
    format!("{pad}let {var} = UIView()\n\
             {pad}{var}.heightAnchor.constraint(equalToConstant: {h}).isActive = true\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_alert(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let title = node.props.get("title").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let message = node.props.get("message").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    format!("{pad}let alert = UIAlertController(title: {title}, message: {message}, preferredStyle: .alert)\n\
             {pad}alert.addAction(UIAlertAction(title: \"OK\", style: .default))\n\
             {pad}present(alert, animated: true)\n")
}

fn emit_ios_scroll_view(node: &ComponentNode, parent: &str, pad: &str, _i1: &str, indent: usize) -> String {
    let var = fresh_var("scrollView");
    let children: String = node.children.iter().map(|c| emit_uikit_view(c, var.as_str(), indent + 1)).collect();
    format!("{pad}let {var} = UIScrollView()\n\
             {pad}{parent}.addSubview({var})\n{children}")
}

fn emit_ios_collection_view(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let columns = node.props.get("columns").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "2".to_string());
    let var = fresh_var("collectionView");
    format!("{pad}let layout = UICollectionViewFlowLayout()\n\
             {pad}layout.minimumInteritemSpacing = 8\n\
             {pad}let {var} = UICollectionView(frame: .zero, collectionViewLayout: layout)\n\
             {pad}// columns: {columns}\n\
             {pad}{parent}.addSubview({var})\n")
}

// ─── Feedback components ──────────────────────────────────────────────────────

fn emit_ios_toast(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let msg = node.props.get("message").or_else(|| node.props.get("content"))
        .map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    format!("{pad}// toast: show auto-dismissing alert\n\
             {pad}let toastAlert = UIAlertController(title: nil, message: {msg}, preferredStyle: .alert)\n\
             {pad}present(toastAlert, animated: true)\n\
             {pad}DispatchQueue.main.asyncAfter(deadline: .now() + 2) {{ toastAlert.dismiss(animated: true) }}\n")
}

fn emit_ios_tooltip(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let text = node.props.get("text").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let var = fresh_var("tooltipLabel");
    format!("{pad}let {var} = UILabel()\n\
             {pad}{var}.text = {text}\n\
             {pad}{var}.font = UIFont.systemFont(ofSize: 12)\n\
             {pad}{var}.textColor = .white\n\
             {pad}{var}.backgroundColor = UIColor.darkGray.withAlphaComponent(0.8)\n\
             {pad}{var}.layer.cornerRadius = 4\n\
             {pad}{var}.clipsToBounds = true\n\
             {pad}{var}.sizeToFit()\n\
             {pad}{var}.isHidden = true\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_badge(node: &ComponentNode, parent: &str, pad: &str, _i1: &str, indent: usize) -> String {
    let count = node.props.get("count").or_else(|| node.props.get("value"))
        .map(|e| emit_swift_expr(e)).unwrap_or_else(|| "0".to_string());
    let var = fresh_var("badgeLabel");
    format!("{pad}let {var} = UILabel()\n\
             {pad}{var}.text = String({count})\n\
             {pad}{var}.backgroundColor = .systemRed\n\
             {pad}{var}.textColor = .white\n\
             {pad}{var}.font = UIFont.boldSystemFont(ofSize: 12)\n\
             {pad}{var}.layer.cornerRadius = 10\n\
             {pad}{var}.clipsToBounds = true\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_progress_bar(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let value = node.props.get("value").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "0.0".to_string());
    let var = fresh_var("progressView");
    format!("{pad}let {var} = UIProgressView(progressViewStyle: .default)\n\
             {pad}{var}.progress = Float({value})\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_progress_circle(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let value = node.props.get("value").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "0.0".to_string());
    let var = fresh_var("activityIndicator");
    format!("{pad}let {var} = UIActivityIndicatorView(style: .large)\n\
             {pad}// for determinate circle use CAShapeLayer with strokeEnd = {value}\n\
             {pad}{var}.startAnimating()\n\
             {pad}{parent}.addSubview({var})\n")
}

// ─── Navigation components ────────────────────────────────────────────────────

fn emit_ios_tab_bar_controller(node: &ComponentNode, parent: &str, pad: &str, _i1: &str, indent: usize) -> String {
    let var = fresh_var("tabBarController");
    let tabs: String = node.children.iter().enumerate().map(|(i, c)| {
        let title = c.props.get("content").or_else(|| c.props.get("title"))
            .map(|e| emit_swift_expr(e)).unwrap_or_else(|| format!("\"Tab {}\"", i+1));
        format!("{pad}    let vc{i} = UIViewController()\n\
                 {pad}    vc{i}.tabBarItem = UITabBarItem(title: {title}, image: nil, tag: {i})\n")
    }).collect();
    format!("{pad}let {var} = UITabBarController()\n{tabs}\
             {pad}{parent}.addSubview({var}.view)\n")
}

fn emit_ios_tab_item(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let title = node.props.get("content").or_else(|| node.props.get("title"))
        .map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"Tab\"".to_string());
    format!("{pad}tabBarItem.title = {title}\n")
}

fn emit_ios_bottom_sheet(node: &ComponentNode, parent: &str, pad: &str, _i1: &str, indent: usize) -> String {
    let var = fresh_var("sheetVC");
    let children: String = node.children.iter().map(|c| emit_uikit_view(c, var.as_str(), indent + 1)).collect();
    format!("{pad}let {var} = UIViewController()\n\
             {pad}if let sheet = {var}.sheetPresentationController {{\n\
             {pad}    sheet.detents = [.medium(), .large()]\n\
             {pad}}}\n\
             {children}\
             {pad}present({var}, animated: true)\n")
}

// ─── Input components ─────────────────────────────────────────────────────────

fn emit_ios_switch(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let is_on = node.props.get("value").or_else(|| node.props.get("checked"))
        .map(|e| emit_swift_expr(e)).unwrap_or_else(|| "false".to_string());
    let var = fresh_var("toggle");
    format!("{pad}let {var} = UISwitch()\n\
             {pad}{var}.isOn = {is_on}\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_checkbox(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let checked = node.props.get("value").or_else(|| node.props.get("checked"))
        .map(|e| emit_swift_expr(e)).unwrap_or_else(|| "false".to_string());
    let var = fresh_var("checkbox");
    format!("{pad}let {var} = UIButton(type: .custom)\n\
             {pad}{var}.setImage(UIImage(systemName: {checked} ? \"checkmark.square.fill\" : \"square\"), for: .normal)\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_radio(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let selected = node.props.get("selected").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "false".to_string());
    let var = fresh_var("radio");
    format!("{pad}let {var} = UIButton(type: .custom)\n\
             {pad}{var}.setImage(UIImage(systemName: {selected} ? \"largecircle.fill.circle\" : \"circle\"), for: .normal)\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_slider(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let value = node.props.get("value").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "0".to_string());
    let min = node.props.get("min").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "0".to_string());
    let max = node.props.get("max").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "100".to_string());
    let var = fresh_var("slider");
    format!("{pad}let {var} = UISlider()\n\
             {pad}{var}.minimumValue = Float({min})\n\
             {pad}{var}.maximumValue = Float({max})\n\
             {pad}{var}.value = Float({value})\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_stepper(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let value = node.props.get("value").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "0".to_string());
    let var = fresh_var("stepper");
    format!("{pad}let {var} = UIStepper()\n\
             {pad}{var}.value = Double({value})\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_text_view(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let var = fresh_var("textView");
    let placeholder = node.props.get("placeholder").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    format!("{pad}let {var} = UITextView()\n\
             {pad}{var}.layer.borderWidth = 1\n\
             {pad}{var}.layer.cornerRadius = 4\n\
             {pad}// placeholder: {placeholder}\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_search_bar(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let placeholder = node.props.get("placeholder").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"Search\"".to_string());
    let var = fresh_var("searchBar");
    format!("{pad}let {var} = UISearchBar()\n\
             {pad}{var}.placeholder = {placeholder}\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_date_picker(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let var = fresh_var("datePicker");
    format!("{pad}let {var} = UIDatePicker()\n\
             {pad}{var}.datePickerMode = .date\n\
             {pad}{var}.preferredDatePickerStyle = .inline\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_time_picker(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let var = fresh_var("timePicker");
    format!("{pad}let {var} = UIDatePicker()\n\
             {pad}{var}.datePickerMode = .time\n\
             {pad}{var}.preferredDatePickerStyle = .wheels\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_color_picker(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let value = node.props.get("value").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "UIColor.blue".to_string());
    let on_change = node.events.on_change.as_ref().map(|e| emit_swift_expr(e)).unwrap_or_else(|| "{}".to_string());
    let var = fresh_var("colorWell");
    format!("{pad}let {var} = UIColorWell(frame: .zero)\n\
             {pad}{var}.selectedColor = {value}\n\
             {pad}{var}.addTarget(self, action: #selector(colorChanged(_:)), for: .valueChanged)\n\
             {pad}{parent}.addSubview({var})\n\
             {pad}// on_change: {on_change}\n")
}

fn emit_ios_rating(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let value = node.props.get("value").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "0".to_string());
    let max = node.props.get("max").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "5".to_string());
    let var = fresh_var("ratingStack");
    format!("{pad}let {var} = UIStackView()\n\
             {pad}{var}.axis = .horizontal\n\
             {pad}for i in 0..<{max} {{\n\
             {pad}    let star = UIButton(type: .custom)\n\
             {pad}    star.setImage(UIImage(systemName: i < {value} ? \"star.fill\" : \"star\"), for: .normal)\n\
             {pad}    {var}.addArrangedSubview(star)\n\
             {pad}}}\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_otp_input(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let length = node.props.get("length").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "6".to_string());
    let var = fresh_var("otpStack");
    format!("{pad}let {var} = UIStackView()\n\
             {pad}{var}.axis = .horizontal\n\
             {pad}{var}.spacing = 8\n\
             {pad}for _ in 0..<{length} {{\n\
             {pad}    let tf = UITextField()\n\
             {pad}    tf.borderStyle = .roundedRect\n\
             {pad}    tf.textAlignment = .center\n\
             {pad}    tf.keyboardType = .numberPad\n\
             {pad}    {var}.addArrangedSubview(tf)\n\
             {pad}}}\n\
             {pad}{parent}.addSubview({var})\n")
}

// ─── Display components ───────────────────────────────────────────────────────

fn emit_ios_avatar(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let src = node.props.get("src").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let size = node.styles.width.as_deref().and_then(parse_dp_str).unwrap_or(48.0);
    let var = fresh_var("avatar");
    format!("{pad}let {var} = UIImageView()\n\
             {pad}{var}.layer.cornerRadius = {half}\n\
             {pad}{var}.clipsToBounds = true\n\
             {pad}{var}.contentMode = .scaleAspectFill\n\
             {pad}// load from: {src}\n\
             {pad}{parent}.addSubview({var})\n", half = size / 2.0)
}

fn emit_ios_chip(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let label = node.props.get("content").or_else(|| node.props.get("label"))
        .map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let on_click = node.events.on_click.as_ref().map(|e| emit_swift_expr(e)).unwrap_or_else(|| "{}".to_string());
    let var = fresh_var("chip");
    format!("{pad}let {var} = UIButton(type: .system)\n\
             {pad}{var}.setTitle({label}, for: .normal)\n\
             {pad}{var}.layer.cornerRadius = 16\n\
             {pad}{var}.layer.borderWidth = 1\n\
             {pad}{var}.addAction(UIAction {{ _ in {on_click} }}, for: .touchUpInside)\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_tag(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let label = node.props.get("content").or_else(|| node.props.get("label"))
        .map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let var = fresh_var("tag");
    format!("{pad}let {var} = UILabel()\n\
             {pad}{var}.text = {label}\n\
             {pad}{var}.layer.cornerRadius = 8\n\
             {pad}{var}.clipsToBounds = true\n\
             {pad}{var}.backgroundColor = .systemGray6\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_banner(node: &ComponentNode, parent: &str, pad: &str, _i1: &str, indent: usize) -> String {
    let var = fresh_var("banner");
    let children: String = node.children.iter().map(|c| emit_uikit_view(c, var.as_str(), indent + 1)).collect();
    format!("{pad}let {var} = UIView()\n\
             {pad}{var}.backgroundColor = .systemBlue.withAlphaComponent(0.15)\n\
             {pad}{parent}.addSubview({var})\n{children}")
}

fn emit_ios_accordion(node: &ComponentNode, parent: &str, pad: &str, _i1: &str, indent: usize) -> String {
    let title = node.props.get("title").or_else(|| node.props.get("content"))
        .map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let var = fresh_var("accordion");
    let children: String = node.children.iter().map(|c| emit_uikit_view(c, var.as_str(), indent + 1)).collect();
    format!("{pad}let {var} = UIView()\n\
             {pad}let {var}Header = UIButton(type: .system)\n\
             {pad}{var}Header.setTitle({title}, for: .normal)\n\
             {pad}{var}.addSubview({var}Header)\n\
             {pad}{parent}.addSubview({var})\n{children}")
}

fn emit_ios_timeline(node: &ComponentNode, parent: &str, pad: &str, _i1: &str, indent: usize) -> String {
    let var = fresh_var("timeline");
    let items: String = node.children.iter().map(|c| emit_uikit_view(c, var.as_str(), indent + 1)).collect();
    format!("{pad}let {var} = UITableView(style: .plain)\n\
             {pad}{parent}.addSubview({var})\n{items}")
}

fn emit_ios_skeleton(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let var = fresh_var("skeleton");
    format!("{pad}let {var} = UIView()\n\
             {pad}{var}.backgroundColor = .systemGray5\n\
             {pad}{var}.layer.cornerRadius = 4\n\
             {pad}{parent}.addSubview({var})\n")
}

// ─── Media components ─────────────────────────────────────────────────────────

fn emit_ios_video_player(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let src = node.props.get("src").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let var = fresh_var("player");
    format!("{pad}// video_player: requires AVFoundation\n\
             {pad}import AVKit\n\
             {pad}if let url = URL(string: {src}) {{\n\
             {pad}    let {var} = AVPlayer(url: url)\n\
             {pad}    let {var}Layer = AVPlayerLayer(player: {var})\n\
             {pad}    {parent}.layer.addSublayer({var}Layer)\n\
             {pad}    {var}.play()\n\
             {pad}}}\n")
}

fn emit_ios_audio_player(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let src = node.props.get("src").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let var = fresh_var("audioPlayer");
    format!("{pad}if let url = URL(string: {src}) {{\n\
             {pad}    let {var} = try? AVAudioPlayer(contentsOf: url)\n\
             {pad}    {var}?.prepareToPlay()\n\
             {pad}    {var}?.play()\n\
             {pad}}}\n")
}

fn emit_ios_lottie(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let src = node.props.get("src").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let var = fresh_var("lottieView");
    format!("{pad}// lottie: requires pod 'lottie-ios'\n\
             {pad}let {var} = LottieAnimationView()\n\
             {pad}{var}.load(filePath: {src})\n\
             {pad}{var}.play()\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_web_view(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let url = node.props.get("url").or_else(|| node.props.get("src"))
        .map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let var = fresh_var("webView");
    format!("{pad}import WebKit\n\
             {pad}let {var} = WKWebView()\n\
             {pad}if let url = URL(string: {url}) {{ {var}.load(URLRequest(url: url)) }}\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_map_view(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let lat = node.props.get("lat").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "0.0".to_string());
    let lng = node.props.get("lng").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "0.0".to_string());
    let var = fresh_var("mapView");
    format!("{pad}import MapKit\n\
             {pad}let {var} = MKMapView()\n\
             {pad}let coord = CLLocationCoordinate2D(latitude: {lat}, longitude: {lng})\n\
             {pad}{var}.setCenter(coord, animated: false)\n\
             {pad}{parent}.addSubview({var})\n")
}

fn emit_ios_camera_view(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let var = fresh_var("previewLayer");
    format!("{pad}// camera_view: AVCaptureVideoPreviewLayer\n\
             {pad}import AVFoundation\n\
             {pad}let session = AVCaptureSession()\n\
             {pad}let {var} = AVCaptureVideoPreviewLayer(session: session)\n\
             {pad}{parent}.layer.addSublayer({var})\n")
}

fn emit_ios_qr_scanner(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let on_scan = node.props.get("on_scan").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "{}".to_string());
    let var = fresh_var("captureSession");
    format!("{pad}let {var} = AVCaptureSession()\n\
             {pad}guard let device = AVCaptureDevice.default(for: .video) else {{ return }}\n\
             {pad}guard let input = try? AVCaptureDeviceInput(device: device) else {{ return }}\n\
             {pad}{var}.addInput(input)\n\
             {pad}let previewLayer = AVCaptureVideoPreviewLayer(session: {var})\n\
             {pad}{parent}.layer.addSublayer(previewLayer)\n\
             {pad}// on_scan: {on_scan}\n")
}

// ─── Gesture components ───────────────────────────────────────────────────────

fn emit_ios_swipeable(node: &ComponentNode, parent: &str, pad: &str, _i1: &str, indent: usize) -> String {
    let on_swipe = node.props.get("on_swipe").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "{}".to_string());
    let var = fresh_var("swipeGesture");
    let children: String = node.children.iter().map(|c| emit_uikit_view(c, parent, indent + 1)).collect();
    format!("{pad}let {var} = UISwipeGestureRecognizer(target: self, action: #selector(onSwipeAction))\n\
             {pad}{parent}.addGestureRecognizer({var})\n\
             {pad}// on_swipe: {on_swipe}\n{children}")
}

fn emit_ios_draggable(node: &ComponentNode, parent: &str, pad: &str, _i1: &str, indent: usize) -> String {
    let on_drag = node.props.get("on_drag").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "{}".to_string());
    let var = fresh_var("panGesture");
    let children: String = node.children.iter().map(|c| emit_uikit_view(c, parent, indent + 1)).collect();
    format!("{pad}let {var} = UIPanGestureRecognizer(target: self, action: #selector(handleDrag))\n\
             {pad}{parent}.addGestureRecognizer({var})\n\
             {pad}// on_drag: {on_drag}\n{children}")
}

fn emit_ios_refresh(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let on_refresh = node.props.get("on_refresh").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "{}".to_string());
    format!("{pad}let refreshControl = UIRefreshControl()\n\
             {pad}refreshControl.addAction(UIAction {{ _ in {on_refresh} }}, for: .valueChanged)\n\
             {pad}// attach to: scrollView.refreshControl = refreshControl\n")
}

fn emit_ios_long_press(node: &ComponentNode, parent: &str, pad: &str, _i1: &str, indent: usize) -> String {
    let on_long_press = node.props.get("on_long_press").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "{}".to_string());
    let var = fresh_var("longPressGesture");
    let children: String = node.children.iter().map(|c| emit_uikit_view(c, parent, indent + 1)).collect();
    format!("{pad}let {var} = UILongPressGestureRecognizer(target: self, action: #selector(handleLongPress))\n\
             {pad}{parent}.addGestureRecognizer({var})\n\
             {pad}// on_long_press: {on_long_press}\n{children}")
}

fn emit_ios_plugin(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let name = node.props.get("name").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let method = node.props.get("method").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let args: Vec<String> = node.props.iter()
        .filter(|(k, _)| *k != "name" && *k != "method")
        .map(|(k, v)| format!("{}: {}", k, emit_swift_expr(v)))
        .collect();
    let args_str = if args.is_empty() { String::new() } else { format!(", {}", args.join(", ")) };
    format!("{pad}let pluginView = PluginBridge.call(name: {name}, method: {method}{args_str})\n\
             {pad}{parent}.addSubview(pluginView)\n")
}

// ─── Swift expression / statement emitters ───────────────────────────────────

pub fn emit_swift_expr(expr: &Expr) -> String {
    match expr {
        Expr::Literal(v) => emit_swift_value(v),
        Expr::Var(name)  => name.clone(),
        Expr::StateField(f) => f.clone(),
        Expr::StoreField(store, field) => format!("{store}.{field}"),
        Expr::BinOp(l, op, r) => {
            let op_str = match op {
                Op::Add => "+", Op::Sub => "-", Op::Mul => "*", Op::Div => "/",
                Op::Mod => "%", Op::Eq => "==", Op::Ne => "!=",
                Op::Lt => "<", Op::Le => "<=", Op::Gt => ">", Op::Ge => ">=",
                Op::And => "&&", Op::Or => "||", Op::Not => "!",
            };
            format!("{} {op_str} {}", emit_swift_expr(l), emit_swift_expr(r))
        }
        Expr::Call(c) if c.func == "navigate" => {
            let route = c.args.first().map(|a| emit_swift_expr(a)).unwrap_or_default();
            format!("navigationController?.pushViewController(routeVC(for: {route}), animated: true)")
        }
        Expr::Call(c) if c.func == "navigate_back" => {
            "navigationController?.popViewController(animated: true)".to_string()
        }
        // ── New typed navigate variants ────────────────────────────────────
        Expr::Navigate(route, opts) => {
            let r = emit_swift_expr(route);
            if opts.clear_stack {
                format!("navigationController?.setViewControllers([routeVC(for: {r})], animated: true)")
            } else if opts.replace {
                format!(
                    "navigationController?.popViewController(animated: false)\nnavigationController?.pushViewController(routeVC(for: {r}), animated: {})",
                    if opts.transition.is_some() { "true" } else { "false" }
                )
            } else {
                let animated = opts.transition.as_deref() != Some("none");
                format!("navigationController?.pushViewController(routeVC(for: {r}), animated: {animated})")
            }
        }
        Expr::NavigateReplace(route) => {
            let r = emit_swift_expr(route);
            format!("navigationController?.popViewController(animated: false)\nnavigationController?.pushViewController(routeVC(for: {r}), animated: false)")
        }
        Expr::NavigateBack => "navigationController?.popViewController(animated: true)".to_string(),
        Expr::NavigateBackTo(route) => {
            let r = emit_swift_expr(route);
            // Pop to the matching VC by route tag
            format!("if let target = navigationController?.viewControllers.first(where: {{ ($0 as? FrameRoutable)?.frameRoute == {r} }}) {{ navigationController?.popToViewController(target, animated: true) }}")
        }
        Expr::NavigateModal(route) => {
            let r = emit_swift_expr(route);
            format!("present(routeVC(for: {r}), animated: true)")
        }
        Expr::NavigateDismiss => "dismiss(animated: true)".to_string(),
        Expr::Call(c) => {
            let mut parts: Vec<String> = c.args.iter().map(|a| emit_swift_expr(a)).collect();
            for (k, v) in &c.named_args {
                parts.push(format!("{}: {}", k, emit_swift_expr(v)));
            }
            if c.func.starts_with("wait:") {
                let inner_func = c.func.strip_prefix("wait:").unwrap_or(&c.func);
                format!("Task {{ await {}({}) }}", inner_func, parts.join(", "))
            } else if c.func.contains('.') {
                if let Some(translated) = crate::stdlib::translate_stdlib_call(&c.func, &parts, "ios") {
                    translated
                } else {
                    format!("{}({})", c.func, parts.join(", "))
                }
            } else {
                format!("{}({})", c.func, parts.join(", "))
            }
        }
        Expr::NullCoalesce(a, b) => format!("{} ?? {}", emit_swift_expr(a), emit_swift_expr(b)),
        Expr::SafeNav(parts) => parts.join("?."),
        Expr::MethodCall(recv, method, args) => {
            let a: String = args.iter().map(|a| emit_swift_expr(a)).collect::<Vec<_>>().join(", ");
            format!("{}.{}({})", emit_swift_expr(recv), method, a)
        }
        Expr::Lambda(params, body) => {
            let p = params.join(", ");
            let stmts: String = body.iter().map(|s| emit_swift_stmt(s, 1)).collect();
            format!("{{ {p} in {stmts} }}")
        }
        Expr::Interpolated(segments) => {
            // Swift string interpolation: "Hello \(name)!"
            use crate::parser::ast::InterpolatedSegment;
            let inner: String = segments.iter().map(|seg| match seg {
                InterpolatedSegment::Literal(s) => s.clone(),
                InterpolatedSegment::Expr(e)    => format!("\\({})", emit_swift_expr(e)),
            }).collect();
            format!("\"{}\"", inner)
        }
    }
}

fn emit_swift_value(v: &Value) -> String {
    match v {
        Value::Str(s)    => format!("\"{}\"", s),
        Value::Int(n)    => n.to_string(),
        Value::Float(f)  => format!("{}", f),
        Value::Bool(b)   => b.to_string(),
        Value::Null      => "nil".to_string(),
        Value::List(items) => {
            let inner = items.iter().map(emit_swift_value).collect::<Vec<_>>().join(", ");
            format!("[{}]", inner)
        }
        Value::Object(map) => {
            let inner = map.iter()
                .map(|(k, v)| format!("\"{}\": {}", k, emit_swift_value(v)))
                .collect::<Vec<_>>().join(", ");
            format!("[{}]", inner)
        }
    }
}

pub fn emit_swift_stmt(stmt: &Stmt, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    match stmt {
        Stmt::VarDecl(vd) => {
            let (sw_type, sw_default) = vd.type_.as_ref().map_or(("Any".to_string(), "nil".to_string()), |t| swift_type_default(t));
            let keyword = if vd.mutable { "var" } else { "let" };
            match &vd.initializer {
                Some(init) => format!("{pad}{keyword} {}: {sw_type} = {}\n", vd.name, emit_swift_expr(init)),
                None => format!("{pad}{keyword} {}: {sw_type} = {sw_default}\n", vd.name),
            }
        }
        Stmt::Assign(name, expr)  => format!("{pad}{name} = {}\n", emit_swift_expr(expr)),
        Stmt::Return(expr)        => format!("{pad}return {}\n", emit_swift_expr(expr)),
        Stmt::Call(c)             => {
            let mut parts: Vec<String> = c.args.iter().map(|a| emit_swift_expr(a)).collect();
            for (k, v) in &c.named_args { parts.push(format!("{}: {}", k, emit_swift_expr(v))); }
            format!("{pad}{}({})\n", c.func, parts.join(", "))
        }
        Stmt::Wait(c)             => {
            let mut parts: Vec<String> = c.args.iter().map(|a| emit_swift_expr(a)).collect();
            for (k, v) in &c.named_args { parts.push(format!("{}: {}", k, emit_swift_expr(v))); }
            format!("{pad}await {}({})\n", c.func, parts.join(", "))
        },
        Stmt::WaitFetch(fe)       => emit_swift_fetch(fe, indent),
        Stmt::If(cond, then, else_) => {
            let then_s: String = then.iter().map(|s| emit_swift_stmt(s, indent+1)).collect();
            let else_s = else_.as_ref().map(|e| {
                let s: String = e.iter().map(|s| emit_swift_stmt(s, indent+1)).collect();
                format!(" else {{\n{s}{pad}}}")
            }).unwrap_or_default();
            format!("{pad}if {} {{\n{then_s}{pad}}}{else_s}\n", emit_swift_expr(cond))
        }
        Stmt::For(var, iter, body) => {
            let b: String = body.iter().map(|s| emit_swift_stmt(s, indent+1)).collect();
            format!("{pad}for {var} in {} {{\n{b}{pad}}}\n", emit_swift_expr(iter))
        }
        Stmt::Switch(expr, cases) => {
            let c: String = cases.iter().map(|(v, b)| {
                let bs: String = b.iter().map(|s| emit_swift_stmt(s, indent+2)).collect();
                format!("{pad}    case {}:\n{bs}", emit_swift_expr(v))
            }).collect();
            format!("{pad}switch {} {{\n{c}{pad}}}\n", emit_swift_expr(expr))
        }
        Stmt::TryCatch { body, catch_param, catch_body, finally_body } => {
            let b: String = body.iter().map(|s| emit_swift_stmt(s, indent+1)).collect();
            let c: String = catch_body.iter().map(|s| emit_swift_stmt(s, indent+1)).collect();
            let f = finally_body.as_ref().map(|f| {
                let fs: String = f.iter().map(|s| emit_swift_stmt(s, indent+1)).collect();
                format!(" defer {{\n{fs}{pad}}}")
            }).unwrap_or_default();
            format!("{pad}do {{\n{b}{pad}}} catch let {catch_param} {{\n{c}{pad}}}{f}\n")
        }
        Stmt::PluginCall(pc) => {
            // Emit plugin bridge call: resolve to the Swift method from the plugin's ios/ bridge
            let params_str: String = pc.params.iter()
                .map(|(k, v)| format!("{}: {}", k, emit_swift_expr(v)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{pad}{}Plugin.{}({})\n", snake_to_pascal(&pc.plugin_name), pc.method, params_str)
        }
    }
}

fn emit_swift_fetch(fe: &FetchExpr, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    let url = emit_swift_expr(&fe.url);
    let method = fe.method.to_uppercase();
    let then_code: String = fe.then_branch.iter().map(|s| emit_swift_stmt(s, indent + 2)).collect();
    let catch_code: String = fe.catch_branch.iter().map(|s| emit_swift_stmt(s, indent + 2)).collect();
    // plan §1h — inject headers
    let headers_code: String = fe.headers.iter()
        .map(|(k, v)| format!("{pad}        request.setValue({}, forHTTPHeaderField: \"{}\")\n",
            emit_swift_expr(v), k))
        .collect();
    format!("{pad}Task {{\n\
             {pad}    do {{\n\
             {pad}        var request = URLRequest(url: URL(string: {url})!)\n\
             {pad}        request.httpMethod = \"{method}\"\n\
             {headers_code}\
             {pad}        let (data, _) = try await URLSession.shared.data(for: request)\n\
             {then_code}\
             {pad}        await MainActor.run {{ /* update state */ }}\n\
             {pad}    }} catch {{\n\
             {catch_code}\
             {pad}    }}\n\
             {pad}}}\n")
}

// ─── Utility helpers ──────────────────────────────────────────────────────────

/// Simple thread-unsafe counter for unique variable names in generated code.
/// In tests this produces "label0", "button1", etc.
fn fresh_var(prefix: &str) -> String {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static CTR: AtomicUsize = AtomicUsize::new(0);
    let n = CTR.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}{n}")
}

/// Parse a dp/sp string to an f64 (e.g. "16dp" → 16.0, "24sp" → 24.0).
fn parse_dp_str(s: &str) -> Option<f64> {
    let s = s.trim();
    for unit in &["dp", "sp", "px", "%"] {
        if let Some(n) = s.strip_suffix(unit) {
            return n.trim().parse::<f64>().ok();
        }
    }
    s.parse::<f64>().ok()
}

/// Parse an sp/pt font size string to f64.
fn parse_sp(s: &str) -> Option<f64> {
    parse_dp_str(s)
}

/// Convert a StackAlignment to an iOS auto-layout description string.
fn ios_stack_alignment(a: &StackAlignment) -> &'static str {
    match a {
        StackAlignment::TopLeft      => "top-left",
        StackAlignment::TopCenter    => "top-center",
        StackAlignment::TopRight     => "top-right",
        StackAlignment::CenterLeft   => "center-left",
        StackAlignment::Center       => "center",
        StackAlignment::CenterRight  => "center-right",
        StackAlignment::BottomLeft   => "bottom-left",
        StackAlignment::BottomCenter => "bottom-center",
        StackAlignment::BottomRight  => "bottom-right",
    }
}

/// Convert Frame FRType to a Swift type + default value.
fn swift_type_default(t: &FRType) -> (String, String) {
    match t {
        FRType::String_      => ("String".into(), "\"\"".into()),
        FRType::Int          => ("Int".into(), "0".into()),
        FRType::Float        => ("Double".into(), "0.0".into()),
        FRType::Bool         => ("Bool".into(), "false".into()),
        // object and list default to nil-able optionals so `= null` in .fr works
        FRType::Object       => ("[String: Any]?".into(), "nil".into()),
        FRType::List         => ("[Any]?".into(), "nil".into()),
        FRType::Nullable(_)  => ("Any?".into(), "nil".into()),
        FRType::Custom(name) => (name.clone(), "nil".into()),
    }
}

/// snake_case / kebab-case → camelCase for Swift variable names.
fn to_var_name(s: &str) -> String {
    let mut result = String::new();
    let mut cap_next = false;
    for ch in s.chars() {
        if ch == '_' || ch == '-' {
            cap_next = true;
        } else if cap_next {
            result.push(ch.to_ascii_uppercase());
            cap_next = false;
        } else {
            result.push(ch.to_ascii_lowercase());
        }
    }
    result
}

/// snake_case / kebab-case → PascalCase (for plugin class names like FrameMapsPlugin).
fn snake_to_pascal(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-' || c == ':')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect()
}

// ─── :obj code generator (iOS) ────────────────────────────────────────────────

/// Generate a Swift struct for a `:obj` declaration.
/// e.g. `:obj User { id: string  name: string  email: string? }`
/// → a Codable Swift struct with CodingKeys
pub fn gen_obj_swift(obj: &ObjDef) -> OutputFile {
    let fields: Vec<String> = obj.fields.iter().map(|f| {
        let (swift_t, _) = swift_type_default(&f.type_);
        if f.optional {
            format!("    var {}: {}?", f.name, swift_t.trim_end_matches('?'))
        } else {
            format!("    var {}: {}", f.name, swift_t.trim_end_matches('?'))
        }
    }).collect();

    let coding_keys: Vec<String> = obj.fields.iter().map(|f| {
        format!("        case {} = \"{}\"", f.name, f.name)
    }).collect();

    OutputFile {
        path: format!("{}.swift", obj.name),
        content: format!(
r#"import Foundation

/// {name} — generated from :obj declaration in Frame source.
/// Do not edit manually; re-run `frame build` to regenerate.
struct {name}: Codable {{
{fields}

    enum CodingKeys: String, CodingKey {{
{coding_keys}
    }}
}}
"#,
            name        = obj.name,
            fields      = fields.join("\n"),
            coding_keys = coding_keys.join("\n"),
        ),
    }
}

// ─── :enum → Swift enum ───────────────────────────────────────────────────────

/// Generate a Swift `enum` for a `:enum` declaration (plan §1b).
///
/// Variants with values → `case name = "value"` (String raw value).
/// Variants without values → `case name` (no raw value).
pub fn gen_enum_swift(enum_def: &EnumDef) -> OutputFile {
    let has_values = enum_def.variants.iter().any(|v| v.value.is_some());
    let raw_value  = if has_values { ": String" } else { "" };

    let cases: Vec<String> = enum_def.variants.iter().map(|v| {
        let swift_name = to_lower_camel_case(&v.name);
        match &v.value {
            Some(val) => format!("    case {} = \"{}\"", swift_name, val),
            None      => format!("    case {}", swift_name),
        }
    }).collect();

    OutputFile {
        path: format!("{}.swift", enum_def.name),
        content: format!(
"import Foundation\n\
\n\
/// {} — generated from :enum declaration in Frame source.\n\
/// Do not edit manually; re-run `frame build` to regenerate.\n\
enum {}{} {{\n\
{}\n\
}}\n",
            enum_def.name,
            enum_def.name,
            raw_value,
            cases.join("\n"),
        ),
    }
}

// ─── :type → Swift typealias ─────────────────────────────────────────────────

/// Generate a Swift `typealias` for a `:type` alias declaration (plan §1c).
pub fn gen_type_alias_swift(alias: &TypeAlias) -> OutputFile {
    let (swift_t, _) = swift_type_default(&alias.target);
    // strip trailing ? from nullable — typealias keeps base type readable
    let base = swift_t.trim_end_matches('?');
    OutputFile {
        path: format!("{}.swift", alias.alias),
        content: format!(
"import Foundation\n\n\
/// {} — generated from :type declaration. Do not edit manually.\n\
typealias {} = {}\n",
            alias.alias, alias.alias, base,
        ),
    }
}

/// Convert PascalCase variant name to lowerCamelCase for Swift convention.
fn to_lower_camel_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None    => String::new(),
        Some(f) => f.to_lowercase().collect::<String>() + chars.as_str(),
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_config() -> IosConfig { IosConfig::default() }

    #[test]
    fn test_gen_ios_empty_ast_produces_core_files() {
        let ast = AST::default();
        let files = gen_ios(&ast, &minimal_config());
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.iter().any(|p| p.contains("Info.plist")));
        assert!(paths.iter().any(|p| p.contains("AppDelegate.swift")));
        assert!(paths.iter().any(|p| p.contains("SceneDelegate.swift")));
        assert!(paths.iter().any(|p| p.contains("MainViewController.swift")));
    }

    #[test]
    fn test_gen_ios_page_produces_view_controller() {
        let mut ast = AST::default();
        ast.pages.push(Page {
            name: "Home".to_string(),
            route: "/".to_string(),
            ..Default::default()
        });
        let files = gen_ios(&ast, &minimal_config());
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.iter().any(|p| p.contains("HomeViewController.swift")));
    }

    #[test]
    fn test_gen_ios_component_produces_view_class() {
        let mut ast = AST::default();
        ast.components.insert("Card".to_string(), ComponentDef { name: "Card".to_string(), ..Default::default() });
        let files = gen_ios(&ast, &minimal_config());
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.iter().any(|p| p.contains("CardView.swift")));
    }

    #[test]
    fn test_gen_ios_store_produces_observable_object() {
        let mut ast = AST::default();
        let store = StoreSlice { name: "Auth".to_string(), ..Default::default() };
        ast.stores.insert("Auth".to_string(), store);
        let files = gen_ios(&ast, &minimal_config());
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.iter().any(|p| p.contains("AuthStore.swift")));
        let store_file = files.iter().find(|f| f.path.contains("AuthStore.swift")).unwrap();
        assert!(store_file.content.contains("ObservableObject"));
    }

    #[test]
    fn test_gen_ios_store_secure_field_uses_keychain() {
        let mut ast = AST::default();
        let mut store = StoreSlice { name: "Auth".to_string(), ..Default::default() };
        store.fields.insert("token".to_string(), StoreField {
            name: "token".to_string(), type_: FRType::String_,
            default: Some(Expr::Literal(Value::Str("".to_string()))),
        });
        store.persist.insert("token".to_string(), PersistStrategy::Secure);
        ast.stores.insert("Auth".to_string(), store);
        let files = gen_ios(&ast, &minimal_config());
        let sf = files.iter().find(|f| f.path.contains("AuthStore.swift")).unwrap();
        assert!(sf.content.contains("KeychainHelper"), "expected Keychain: {}", sf.content);
    }

    #[test]
    fn test_gen_ios_store_local_field_uses_userdefaults() {
        let mut ast = AST::default();
        let mut store = StoreSlice { name: "Settings".to_string(), ..Default::default() };
        store.fields.insert("theme".to_string(), StoreField {
            name: "theme".to_string(), type_: FRType::String_,
            default: Some(Expr::Literal(Value::Str("dark".to_string()))),
        });
        store.persist.insert("theme".to_string(), PersistStrategy::Local);
        ast.stores.insert("Settings".to_string(), store);
        let files = gen_ios(&ast, &minimal_config());
        let sf = files.iter().find(|f| f.path.contains("SettingsStore.swift")).unwrap();
        assert!(sf.content.contains("UserDefaults"), "expected UserDefaults: {}", sf.content);
    }

    #[test]
    fn test_info_plist_contains_camera_key_when_used() {
        let config = minimal_config();
        let file = gen_info_plist_full(&config, true, false, false, false, false, false, false, false, false, false, false, false, false);
        assert!(file.content.contains("NSCameraUsageDescription"));
    }

    #[test]
    fn test_info_plist_contains_location_key_when_used() {
        let config = minimal_config();
        let file = gen_info_plist_full(&config, false, false, true, false, false, false, false, false, false, false, false, false, false);
        assert!(file.content.contains("NSLocationWhenInUseUsageDescription"));
    }

    #[test]
    fn test_info_plist_contains_ats_when_http_used() {
        let config = minimal_config();
        let file = gen_info_plist_full(&config, false, false, false, false, false, false, false, false, false, false, false, false, true);
        assert!(file.content.contains("NSAppTransportSecurity"));
    }

    #[test]
    fn test_camera_helper_generated_when_camera_used() {
        let mut ast = AST::default();
        ast.functions.insert("capture".to_string(), Function {
            name: "capture".to_string(), is_async: false, params: vec![],
            return_type: None,
            body: vec![Stmt::Call(CallExpr { func: "camera:capture".to_string(), args: vec![], named_args: vec![] })],
        });
        let files = gen_ios(&ast, &minimal_config());
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.iter().any(|p| p.contains("CameraHelper.swift")));
    }

    #[test]
    fn test_location_helper_generated_when_location_used() {
        let mut ast = AST::default();
        ast.functions.insert("getPos".to_string(), Function {
            name: "getPos".to_string(), is_async: false, params: vec![],
            return_type: None,
            body: vec![Stmt::Call(CallExpr { func: "location:get".to_string(), args: vec![], named_args: vec![] })],
        });
        let files = gen_ios(&ast, &minimal_config());
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.iter().any(|p| p.contains("LocationHelper.swift")));
    }

    #[test]
    fn test_notification_helper_generated_when_notification_used() {
        let mut ast = AST::default();
        ast.functions.insert("notify".to_string(), Function {
            name: "notify".to_string(), is_async: false, params: vec![],
            return_type: None,
            body: vec![Stmt::Call(CallExpr { func: "notification:send".to_string(), args: vec![], named_args: vec![] })],
        });
        let files = gen_ios(&ast, &minimal_config());
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.iter().any(|p| p.contains("NotificationHelper.swift")));
    }

    #[test]
    fn test_swift_expr_navigate_emits_push_view_controller() {
        let expr = Expr::Call(CallExpr {
            func: "navigate".to_string(),
            args: vec![Expr::Literal(Value::Str("/home".to_string()))],
            named_args: vec![],
        });
        let result = emit_swift_expr(&expr);
        assert!(result.contains("navigationController?.pushViewController"), "got: {result}");
    }

    #[test]
    fn test_swift_expr_navigate_back_emits_pop() {
        let expr = Expr::Call(CallExpr { func: "navigate_back".to_string(), args: vec![], named_args: vec![] });
        let result = emit_swift_expr(&expr);
        assert_eq!(result, "navigationController?.popViewController(animated: true)");
    }

    #[test]
    fn test_emit_ios_text_node() {
        let mut node = ComponentNode::default();
        node.kind = "text".to_string();
        node.props.insert("content".to_string(), Expr::Literal(Value::Str("Hello".to_string())));
        let result = emit_uikit_view(&node, "view", 1);
        assert!(result.contains("UILabel"), "got: {result}");
        assert!(result.contains("\"Hello\""), "got: {result}");
    }

    #[test]
    fn test_emit_ios_button_node() {
        let mut node = ComponentNode::default();
        node.kind = "button".to_string();
        node.props.insert("content".to_string(), Expr::Literal(Value::Str("Tap".to_string())));
        let result = emit_uikit_view(&node, "view", 1);
        assert!(result.contains("UIButton"), "got: {result}");
    }

    #[test]
    fn test_emit_ios_switch_node() {
        let mut node = ComponentNode::default();
        node.kind = "switch".to_string();
        node.props.insert("value".to_string(), Expr::Literal(Value::Bool(true)));
        let result = emit_uikit_view(&node, "view", 1);
        assert!(result.contains("UISwitch"), "got: {result}");
    }

    #[test]
    fn test_emit_ios_progress_bar_node() {
        let mut node = ComponentNode::default();
        node.kind = "progress_bar".to_string();
        node.props.insert("value".to_string(), Expr::Literal(Value::Float(0.5)));
        let result = emit_uikit_view(&node, "view", 1);
        assert!(result.contains("UIProgressView"), "got: {result}");
    }

    #[test]
    fn test_emit_ios_image_node_cover() {
        let mut node = ComponentNode::default();
        node.kind = "image".to_string();
        node.styles.image_fit = ImageFitValue::Cover;
        node.props.insert("src".to_string(), Expr::Literal(Value::Str("photo.png".to_string())));
        let result = emit_uikit_view(&node, "view", 1);
        assert!(result.contains("scaleAspectFill"), "got: {result}");
    }

    #[test]
    fn test_podfile_includes_maps_when_map_view_used() {
        let mut ast = AST::default();
        let mut page = Page { name: "Map".to_string(), route: "/map".to_string(), ..Default::default() };
        page.children.push(ComponentNode { kind: "map_view".to_string(), ..Default::default() });
        ast.pages.push(page);
        let file = gen_podfile(&minimal_config(), &ast);
        assert!(file.content.contains("GoogleMaps"), "got: {}", file.content);
    }

    #[test]
    fn test_podfile_includes_lottie_when_lottie_used() {
        let mut ast = AST::default();
        let mut page = Page { name: "P".to_string(), route: "/p".to_string(), ..Default::default() };
        page.children.push(ComponentNode { kind: "lottie".to_string(), ..Default::default() });
        ast.pages.push(page);
        let file = gen_podfile(&minimal_config(), &ast);
        assert!(file.content.contains("lottie-ios"), "got: {}", file.content);
    }
}

// ─── Task 13: KeychainHelper + route param passing ────────────────────────────

/// UIColor hex extension — lets generated code call UIColor(hex: "#RRGGBB").
pub fn gen_uicolor_extension() -> OutputFile {
    OutputFile {
        path: "UIColorExtension.swift".to_string(),
        content: concat!(
            "import UIKit\n\n",
            "extension UIColor {\n",
            "    // Init from a CSS hex string like #RRGGBB or #RRGGBBAA\n",
            "    convenience init(hex: String) {\n",
            "        var h = hex.trimmingCharacters(in: .whitespacesAndNewlines)\n",
            "        if h.hasPrefix(\"#\") { h = String(h.dropFirst()) }\n",
            "        let scanner = Scanner(string: h)\n",
            "        var rgba: UInt64 = 0\n",
            "        scanner.scanHexInt64(&rgba)\n",
            "        let r, g, b, a: CGFloat\n",
            "        switch h.count {\n",
            "        case 6:\n",
            "            r = CGFloat((rgba >> 16) & 0xFF) / 255\n",
            "            g = CGFloat((rgba >>  8) & 0xFF) / 255\n",
            "            b = CGFloat( rgba        & 0xFF) / 255\n",
            "            a = 1.0\n",
            "        case 8:\n",
            "            r = CGFloat((rgba >> 24) & 0xFF) / 255\n",
            "            g = CGFloat((rgba >> 16) & 0xFF) / 255\n",
            "            b = CGFloat((rgba >>  8) & 0xFF) / 255\n",
            "            a = CGFloat( rgba        & 0xFF) / 255\n",
            "        default:\n",
            "            r = 1; g = 1; b = 1; a = 1\n",
            "        }\n",
            "        self.init(red: r, green: g, blue: b, alpha: a)\n",
            "    }\n",
            "}\n"
        ).to_string(),
    }
}

/// Generate a KeychainHelper.swift utility file.
pub fn gen_keychain_helper() -> OutputFile {
    OutputFile {
        path: "KeychainHelper.swift".to_string(),
        content: r#"import Foundation
import Security

struct KeychainHelper {
    static func save(key: String, value: String) {
        let data = Data(value.utf8)
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: key,
            kSecValueData as String: data
        ]
        SecItemDelete(query as CFDictionary)
        SecItemAdd(query as CFDictionary, nil)
    }

    static func read(key: String) -> String? {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: key,
            kSecReturnData as String: true,
            kSecMatchLimit as String: kSecMatchLimitOne
        ]
        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        guard status == errSecSuccess, let data = result as? Data else { return nil }
        return String(data: data, encoding: .utf8)
    }

    static func delete(key: String) {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: key
        ]
        SecItemDelete(query as CFDictionary)
    }
}
"#.to_string(),
    }
}

// ─── FrameAppLifecycle.swift ──────────────────────────────────────────────────

/// Generate the FrameAppLifecycle.swift file that wires :app {} hooks into
/// the FrameAppLifecycle singleton declared in AppDelegate.
fn gen_frame_app_lifecycle(ast: &AST) -> OutputFile {
    let on_launch = ast.on_launch.as_ref()
        .map(|f| format!("        FrameAppLifecycle.shared.onLaunchHandler     = {{ {f}() }}\n"))
        .unwrap_or_default();
    let on_fg = ast.on_foreground.as_ref()
        .map(|f| format!("        FrameAppLifecycle.shared.onForegroundHandler = {{ {f}() }}\n"))
        .unwrap_or_default();
    let on_bg = ast.on_background.as_ref()
        .map(|f| format!("        FrameAppLifecycle.shared.onBackgroundHandler = {{ {f}() }}\n"))
        .unwrap_or_default();

    OutputFile {
        path: "FrameAppLifecycle.swift".to_string(),
        content: format!(r#"// FrameAppLifecycle.swift — generated by Frame
// Wires :app {{ }} lifecycle hooks into FrameAppLifecycle singleton.
import UIKit

/// Called once at startup from AppDelegate.application(_:didFinishLaunchingWithOptions:).
func frameRegisterAppLifecycle() {{
{on_launch}{on_fg}{on_bg}}}
"#),
    }
}

/// Generate RouteHelper.swift — maps route strings to typed ViewControllers.
/// Also defines the FrameRoutable protocol used by NavigateBackTo.
pub fn gen_route_helper(ast: &AST) -> OutputFile {
    let mut cases = String::new();
    for page in &ast.pages {
        let route = &page.route;
        let name  = &page.name;

        // Use explicit page.params if declared, otherwise extract from route segments
        let param_names: Vec<(String, String)> = if !page.params.is_empty() {
            page.params.iter().map(|(n, t)| {
                let (st, _) = swift_type_default(t);
                (n.clone(), st)
            }).collect()
        } else {
            route.split('/').filter(|s| s.starts_with(':'))
                .map(|s| (s.trim_start_matches(':').to_string(), "String".to_string()))
                .collect()
        };

        if param_names.is_empty() {
            cases.push_str(&format!(
                "        case \"{route}\":\n            return {name}ViewController()\n"
            ));
        } else {
            // Build a regex pattern for matching /profile/42 against /profile/:userId
            let pattern = route.split('/').map(|seg| {
                if seg.starts_with(':') { "([^/]+)".to_string() } else { regex_escape(seg) }
            }).collect::<Vec<_>>().join("/");
            let names_arr = param_names.iter()
                .map(|(p, _)| format!("\"{}\"", p)).collect::<Vec<_>>().join(", ");
            let vc_args = param_names.iter()
                .map(|(p, _)| format!("{p}: params[\"{p}\"]")).collect::<Vec<_>>().join(", ");
            cases.push_str(&format!(
                "        case let r where r.matches(regex: \"{pattern}\"):\n            let params = r.extractParams(pattern: \"{pattern}\", names: [{names_arr}])\n            return {name}ViewController({vc_args})\n"
            ));
        }
    }

    let mut content = String::new();
    content.push_str("import UIKit\n\n");
    // Protocol that lets navigateBackTo find a VC by route
    content.push_str("/// Protocol adopted by all generated ViewControllers so navigate_back_to() can find them.\n");
    content.push_str("protocol FrameRoutable {\n    var frameRoute: String { get }\n}\n\n");
    content.push_str("extension String {\n");
    content.push_str("    func matches(regex pattern: String) -> Bool {\n");
    content.push_str("        return range(of: \"^\\(pattern)$\", options: .regularExpression) != nil\n");
    content.push_str("    }\n");
    content.push_str("    func extractParams(pattern: String, names: [String]) -> [String: String] {\n");
    content.push_str("        var result = [String: String]()\n");
    content.push_str("        guard let regex = try? NSRegularExpression(pattern: \"^\\(pattern)$\"),\n");
    content.push_str("              let match = regex.firstMatch(in: self, range: NSRange(self.startIndex..., in: self)) else { return result }\n");
    content.push_str("        for (i, name) in names.enumerated() {\n");
    content.push_str("            let range = match.range(at: i + 1)\n");
    content.push_str("            if let r = Range(range, in: self) { result[name] = String(self[r]) }\n");
    content.push_str("        }\n");
    content.push_str("        return result\n");
    content.push_str("    }\n");
    content.push_str("}\n\n");
    content.push_str("func routeVC(for route: String) -> UIViewController {\n");
    content.push_str("    switch route {\n");
    content.push_str(&cases);
    content.push_str("    default:\n        return UIViewController()\n");
    content.push_str("    }\n}\n");

    OutputFile { path: "RouteHelper.swift".to_string(), content }
}

/// Escape a literal route segment so it's safe inside a regex pattern.
fn regex_escape(s: &str) -> String {
    s.chars().map(|c| match c {
        '.' | '+' | '*' | '?' | '^' | '$' | '{' | '}' | '[' | ']' | '(' | ')' | '|' | '\\' => {
            format!("\\{c}")
        }
        _ => c.to_string(),
    }).collect()
}

// ─── Task 14: Overflow & Responsive helpers ───────────────────────────────────

/// Apply overflow-related UIKit properties to a UIView variable.
/// Returns the Swift lines to add after the view is created.
pub fn emit_ios_overflow_props(styles: &Styles, var: &str, pad: &str) -> String {
    let mut lines = String::new();

    // overflow: hidden + border_radius → masksToBounds + cornerRadius
    match styles.overflow {
        OverflowValue::Hidden => {
            lines.push_str(&format!("{pad}{var}.clipsToBounds = true\n"));
            if let Some(br) = &styles.border_radius {
                if let Some(r) = parse_dp_str(br) {
                    lines.push_str(&format!("{pad}{var}.layer.cornerRadius = {r}\n"));
                }
            }
        }
        OverflowValue::ScrollY => {
            lines.push_str(&format!("{pad}let {var}Scroll = UIScrollView()\n"));
            lines.push_str(&format!("{pad}{var}Scroll.showsVerticalScrollIndicator = {}\n",
                styles.scroll_indicator.unwrap_or(true)));
            lines.push_str(&format!("{pad}{var}Scroll.addSubview({var})\n"));
        }
        OverflowValue::ScrollX => {
            lines.push_str(&format!("{pad}let {var}Scroll = UIScrollView()\n"));
            lines.push_str(&format!("{pad}{var}Scroll.showsHorizontalScrollIndicator = {}\n",
                styles.scroll_indicator.unwrap_or(true)));
            lines.push_str(&format!("{pad}{var}Scroll.addSubview({var})\n"));
        }
        OverflowValue::Scroll => {
            lines.push_str(&format!("{pad}let {var}Scroll = UIScrollView()\n"));
            lines.push_str(&format!("{pad}{var}Scroll.addSubview({var})\n"));
        }
        OverflowValue::Auto => {
            lines.push_str(&format!("{pad}// overflow: auto — add UIScrollView if content exceeds bounds\n"));
            lines.push_str(&format!("{pad}let {var}Scroll = UIScrollView()\n"));
            lines.push_str(&format!("{pad}{var}Scroll.addSubview({var})\n"));
        }
        OverflowValue::Visible => {} // default — no clipping
    }

    // scroll_snap
    match styles.scroll_snap {
        ScrollSnap::Start | ScrollSnap::Center | ScrollSnap::End => {
            lines.push_str(&format!("{pad}{var}Scroll.isPagingEnabled = true\n"));
        }
        ScrollSnap::None_ => {}
    }

    // on_scroll / on_scroll_end
    if let Some(handler) = &styles.on_scroll {
        lines.push_str(&format!("{pad}// on_scroll: {handler} — implement UIScrollViewDelegate.scrollViewDidScroll\n"));
    }
    if let Some(handler) = &styles.on_scroll_end {
        lines.push_str(&format!("{pad}// on_scroll_end: {handler} — implement UIScrollViewDelegate.scrollViewDidEndDecelerating\n"));
    }

    lines
}

/// Emit Swift screen-size utility variables (mirrors Android's LocalConfiguration bindings).
pub fn emit_ios_screen_utilities(indent: usize) -> String {
    let pad = "    ".repeat(indent);
    format!(
        "{pad}let screenWidth  = UIScreen.main.bounds.width\n\
         {pad}let screenHeight = UIScreen.main.bounds.height\n\
         {pad}let isPhone      = UIDevice.current.userInterfaceIdiom == .phone\n\
         {pad}let isTablet     = UIDevice.current.userInterfaceIdiom == .pad\n\
         {pad}let isLandscape  = UIDevice.current.orientation.isLandscape\n\
         {pad}let orientation  = isLandscape ? \"landscape\" : \"portrait\"\n"
    )
}

/// Emit NSLayoutConstraint-based breakpoint overrides for a UIView.
/// Works with traitCollectionDidChange — documents what constraints to toggle.
pub fn emit_ios_breakpoint_overrides(styles: &Styles, var: &str, pad: &str) -> String {
    if styles.breakpoint_overrides.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    let mut bps: Vec<(&String, &Box<Styles>)> = styles.breakpoint_overrides.iter().collect();
    bps.sort_by_key(|(k, _)| k.as_str());
    out.push_str(&format!("{pad}// Responsive breakpoint overrides for {var}\n"));
    out.push_str(&format!("{pad}// Implement traitCollectionDidChange to apply:\n"));
    for (bp_name, bp_styles) in &bps {
        let threshold: u32 = match bp_name.as_str() { "sm" => 360, "md" => 600, "lg" => 900, "xl" => 1200, _ => 600 };
        if let Some(w) = &bp_styles.width {
            out.push_str(&format!("{pad}// @{bp_name} (>={threshold}pt): width = {w}\n"));
        }
        if let Some(fs) = &bp_styles.font_size {
            out.push_str(&format!("{pad}// @{bp_name} (>={threshold}pt): font_size = {fs}\n"));
        }
    }
    out
}

/// Generate a UIFont from a Frame font_size style value.
pub fn swift_font_from_styles(styles: &Styles) -> Option<String> {
    styles.font_size.as_deref().and_then(parse_dp_str).map(|size| {
        let weight = match styles.font_weight.as_deref().unwrap_or("regular") {
            "bold" | "700" => ".bold",
            "semibold" | "600" => ".semibold",
            "medium" | "500" => ".medium",
            "light" | "300" => ".light",
            _ => ".regular",
        };
        format!("UIFont.systemFont(ofSize: {size}, weight: {weight})")
    })
}
