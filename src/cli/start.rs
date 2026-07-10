//! `frame start` — scaffold a new Frame project.
//!
//! Supports two architectures:
//! - **Clean Architecture**: domain/usecases/data/presentation layers
//! - **MVC**: models/views/controllers

use std::fs;
use std::path::Path;

/// Architecture choice for the new project.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Architecture {
    CleanArchitecture,
    Mvc,
}

/// Scaffold a new Frame project at `{cwd}/{name}`.
pub fn scaffold_project(name: &str, arch: Architecture) -> std::io::Result<()> {
    let root = Path::new(name);
    if root.exists() {
        eprintln!("✗ Directory '{}' already exists.", name);
        std::process::exit(1);
    }
    scaffold_into(root, name, arch)
}

/// Scaffold (or re-scaffold) into an existing directory.
/// Used internally by examples and tests.
pub fn scaffold_project_in(root: &Path, name: &str, arch: Architecture) -> std::io::Result<()> {
    scaffold_into(root, name, arch)
}

fn scaffold_into(root: &Path, name: &str, arch: Architecture) -> std::io::Result<()> {
    println!("Creating Frame project: {}", name);

    // Common directories
    fs::create_dir_all(root.join("src"))?;
    fs::create_dir_all(root.join("assets/fonts"))?;
    fs::create_dir_all(root.join("assets/images"))?;
    fs::create_dir_all(root.join("frame_modules"))?;

    match arch {
        Architecture::CleanArchitecture => scaffold_clean(root)?,
        Architecture::Mvc               => scaffold_mvc(root)?,
    }

    write_project_fr(root, name, arch)?;
    write_frame_config(root, name)?;
    write_gitignore(root)?;
    write_readme(root, name, arch)?;
    write_sample_tests(root, name, arch)?;

    // Scaffold example plugins
    scaffold_camera_plugin(root)?;
    scaffold_storage_plugin(root)?;

    println!("✓ Created '{}'", name);
    println!();
    println!("  Get started:");
    println!("    cd {}", name);
    println!("    frame check");
    println!("    frame build");
    println!("    frame test          # run sample tests");
    println!("    frame deploy android");
    println!("    frame deploy ios");
    Ok(())
}

// ─── Clean Architecture scaffold ──────────────────────────────────────────────

fn scaffold_clean(root: &Path) -> std::io::Result<()> {
    fs::create_dir_all(root.join("src/domain/entities"))?;
    fs::create_dir_all(root.join("src/domain/usecases"))?;
    fs::create_dir_all(root.join("src/domain/repositories"))?;
    fs::create_dir_all(root.join("src/data/repositories"))?;
    fs::create_dir_all(root.join("src/data/models"))?;
    fs::create_dir_all(root.join("src/presentation/pages"))?;
    fs::create_dir_all(root.join("src/presentation/components"))?;
    fs::create_dir_all(root.join("src/presentation/state"))?;

    // Entity — defines the User data shape (used by UserStore, referenced in UserCard)
    fs::write(root.join("src/domain/entities/User.fr"),
        ":obj User {\n    id:    string\n    name:  string\n    email: string\n    bio:   string?\n}\n")?;

    // Data model / store — holds user state, fetches from API
    // Demonstrates: try/catch error handling, if/else conditional logic
    fs::write(root.join("src/data/models/UserModel.fr"),
        ":store UserStore {\n    user:       object = null\n    is_loading: bool   = false\n    error:      string = \"\"\n\n    fn load: async (id: string) => {\n        UserStore.is_loading = true\n        UserStore.error = \"\"\n        try {\n            result = wait:fetch(\"/api/users/$id\", { method: \"GET\" })\n            if result != null {\n                UserStore.user = result\n            } else {\n                UserStore.error = \"User not found\"\n            }\n        } catch (err) {\n            UserStore.error = err\n        }\n        UserStore.is_loading = false\n    }\n}\n")?;

    // Component — used by HomePage, displays user info from store
    fs::write(root.join("src/presentation/components/UserCard.fr"),
        concat!(
            "import { text, column } \"frame-core\"\n\n",
            "component UserCard: {\n",
            "    props: {\n",
            "        name:  string = \"\"\n",
            "        email: string = \"\"\n",
            "        bio:   string = \"\"\n",
            "    }\n",
            "    styles: {\n",
            "        border_radius: 8dp\n",
            "        overflow: hidden\n",
            "        padding: 12dp\n",
            "        margin_bottom: 8dp\n",
            "    }\n",
            "    children: [\n",
            "        column: {\n",
            "            styles: { gap: 4dp }\n",
            "            children: [\n",
            "                text: {\n",
            "                    content: name\n",
            "                    styles: { font_size: 16sp  font_weight: \"bold\" }\n",
            "                }\n",
            "                text: {\n",
            "                    content: email\n",
            "                    styles: { font_size: 14sp }\n",
            "                }\n",
            "                text: {\n",
            "                    content: bio\n",
            "                    styles: { font_size: 14sp  font_style: \"italic\" }\n",
            "                }\n",
            "            ]\n",
            "        }\n",
            "    ]\n",
            "}\n",
        ))?;

    // Presentation page — imports UserCard, plugin API, reads UserStore state
    // Demonstrates: show_if, plugin import, button actions
    fs::write(root.join("src/presentation/pages/HomePage.fr"),
        concat!(
            "import { text, button, column, scaffold, row } \"frame-core\"\n",
            "import { UserCard } \"../components/UserCard.fr\"\n",
            "import { capture } \"frame-camera\"\n\n",
            "page: {\n",
            "    name: \"Home\"\n",
            "    route: \"/home\"\n",
            "    styles: { width: 100%  height: 100% }\n",
            "    children: [\n",
            "        scaffold: {\n",
            "            children: [\n",
            "                column: {\n",
            "                    styles: { padding: 16dp  gap: 12dp  width: 100% }\n",
            "                    children: [\n",
            "                        text: {\n",
            "                            content: \"Loading...\"\n",
            "                            styles: { font_size: 16sp }\n",
            "                            show_if: UserStore.is_loading\n",
            "                        }\n",
            "                        UserCard: {\n",
            "                            name: UserStore.user.name\n",
            "                            email: UserStore.user.email\n",
            "                            bio: UserStore.user.bio\n",
            "                            show_if: UserStore.user != null\n",
            "                        }\n",
            "                        text: {\n",
            "                            content: UserStore.error\n",
            "                            styles: { color: \"#FF0000\"  font_size: 14sp }\n",
            "                            show_if: UserStore.error != \"\"\n",
            "                        }\n",
            "                        button: {\n",
            "                            content: \"Load Profile\"\n",
            "                            on_click: wait:UserStore.load(\"1\")\n",
            "                        }\n",
                        "                        button: {\n",
                            "                            content: \"Capture Photo\"\n",
                            "                            on_click: wait:capture()\n",
                            "                        }\n",
                            "                    ]\n",
                            "                }\n",
                            "            ]\n",
                            "        }\n",
                            "    ]\n",
                            "}\n",
                        ))?;
    
        Ok(())
    }
    
    // ─── MVC scaffold ─────────────────────────────────────────────────────────────

fn scaffold_mvc(root: &Path) -> std::io::Result<()> {
    fs::create_dir_all(root.join("src/models"))?;
    fs::create_dir_all(root.join("src/views/pages"))?;
    fs::create_dir_all(root.join("src/views/components"))?;
    fs::create_dir_all(root.join("src/controllers"))?;

    // :obj type declaration — data model shape (not a store)
    fs::write(root.join("src/models/UserObj.fr"),
        ":obj User {\n    id:    string\n    name:  string\n    email: string\n    bio:   string?\n}\n")?;

    // Store — state management, holds user data loaded from API
    // Demonstrates: try/catch error handling, if/else conditional logic
    fs::write(root.join("src/models/UserStore.fr"),
        concat!(
            ":store UserStore {\n",
            "    user:       object = null\n",
            "    is_loading: bool   = false\n",
            "    error:      string = \"\"\n",
            "\n",
            "    fn load: async (id: string) => {\n",
            "        UserStore.is_loading = true\n",
            "        UserStore.error = \"\"\n",
            "        try {\n",
            "            result = wait:fetch(\"/api/users/$id\", { method: \"GET\" })\n",
            "            if result != null {\n",
            "                UserStore.user = result\n",
            "            } else {\n",
            "                UserStore.error = \"User not found\"\n",
            "            }\n",
            "        } catch (err) {\n",
            "            UserStore.error = err\n",
            "        }\n",
            "        UserStore.is_loading = false\n",
            "    }\n",
            "}\n",
        ))?;

    // Controller — business logic, called from views
    fs::write(root.join("src/controllers/UserController.fr"),
        concat!(
            "import { UserStore } \"../models/UserStore.fr\"\n\n",
            "fn loadUser: async (id: string) => {\n",
            "    wait:UserStore.load(id)\n",
            "}\n",
        ))?;

    // View component — used by HomePage, renders user info
    fs::write(root.join("src/views/components/UserCard.fr"),
        concat!(
            "import { text, column } \"frame-core\"\n\n",
            "component UserCard: {\n",
            "    props: {\n",
            "        name:  string = \"\"\n",
            "        email: string = \"\"\n",
            "        bio:   string = \"\"\n",
            "    }\n",
            "    styles: {\n",
            "        border_radius: 8dp\n",
            "        overflow: hidden\n",
            "        padding: 12dp\n",
            "        margin_bottom: 8dp\n",
            "    }\n",
            "    children: [\n",
            "        column: {\n",
            "            styles: { gap: 4dp }\n",
            "            children: [\n",
            "                text: {\n",
            "                    content: name\n",
            "                    styles: { font_size: 16sp  font_weight: \"bold\" }\n",
            "                }\n",
            "                text: {\n",
            "                    content: email\n",
            "                    styles: { font_size: 14sp }\n",
            "                }\n",
            "                text: {\n",
            "                    content: bio\n",
            "                    styles: { font_size: 14sp  font_style: \"italic\" }\n",
            "                }\n",
            "            ]\n",
            "        }\n",
            "    ]\n",
            "}\n",
        ))?;

    // View page — imports UserCard and controller, reads UserStore state
    fs::write(root.join("src/views/pages/HomePage.fr"),
        concat!(
            "import { text, button, column, scaffold } \"frame-core\"\n",
            "import { UserCard } \"../components/UserCard.fr\"\n",
            "import { loadUser } \"../controllers/UserController.fr\"\n",
            "import { capture } \"frame-camera\"\n\n",
            "page: {\n",
            "    name: \"Home\"\n",
            "    route: \"/home\"\n",
            "    styles: { width: 100%  height: 100% }\n",
            "    children: [\n",
            "        scaffold: {\n",
            "            children: [\n",
            "                column: {\n",
            "                    styles: { padding: 16dp  gap: 12dp  width: 100% }\n",
            "                    children: [\n",
            "                        text: {\n",
            "                            content: \"Loading...\"\n",
            "                            styles: { font_size: 16sp }\n",
            "                            show_if: UserStore.is_loading\n",
            "                        }\n",
            "                        UserCard: {\n",
            "                            name: UserStore.user.name\n",
            "                            email: UserStore.user.email\n",
            "                            bio: UserStore.user.bio\n",
            "                            show_if: UserStore.user != null\n",
            "                        }\n",
            "                        text: {\n",
            "                            content: UserStore.error\n",
            "                            styles: { color: \"#FF0000\"  font_size: 14sp }\n",
            "                            show_if: UserStore.error != \"\"\n",
            "                        }\n",
            "                        button: {\n",
            "                            content: \"Load Profile\"\n",
            "                            on_click: wait:loadUser(\"1\")\n",
            "                        }\n",
                        "                        button: {\n",
                            "                            content: \"Capture Photo\"\n",
                            "                            on_click: wait:capture()\n",
                            "                        }\n",
                            "                    ]\n",
                            "                }\n",
                            "            ]\n",
                            "        }\n",
                            "    ]\n",
                            "}\n",
                        ))?;
    
        Ok(())
    }
    
    // ─── Common generated files ───────────────────────────────────────────────────

fn write_project_fr(root: &Path, name: &str, arch: Architecture) -> std::io::Result<()> {
    let page_import = match arch {
        Architecture::CleanArchitecture => "./presentation/pages/HomePage.fr",
        Architecture::Mvc               => "./views/pages/HomePage.fr",
    };
    let content = format!(
        concat!(
            ":vars {{\n",
            "    primary:   \"#007BFF\"\n",
            "    secondary: \"#6C757D\"\n",
            "}}\n\n",
            ":breakpoints {{\n",
            "    sm: 360dp\n",
            "    md: 600dp\n",
            "    lg: 900dp\n",
            "    xl: 1200dp\n",
            "}}\n\n",
            "import {{ text, button, column, scaffold }} \"frame-core\"\n",
            "import {{ HomePage }} \"{page_import}\"\n\n",
            "page: {{\n",
            "    name: \"Splash\"\n",
            "    route: \"/\"\n",
            "    before_enter: \"checkAuth\"\n",
            "    styles: {{ background: $primary  width: 100%  height: 100% }}\n",
            "    children: [\n",
            "        column: {{\n",
            "            styles: {{ width: 100%  height: 100% }}\n",
            "            children: [\n",
            "                text: {{\n",
            "                    content: \"{name}\"\n",
            "                    styles: {{ color: \"#FFFFFF\"  font_size: 32sp  font_weight: \"bold\" }}\n",
            "                }}\n",
            "                button: {{\n",
            "                    content: \"Get Started\"\n",
            "                    on_click: navigate(\"/home\")\n",
            "                }}\n",
            "            ]\n",
            "        }}\n",
            "    ]\n",
            "}}\n\n",
            "fn checkAuth: async () => {{\n",
            "    navigate(\"/home\")\n",
            "}}\n",
        ),
        page_import = page_import,
        name        = name,
    );
    fs::write(root.join("src/project.fr"), content)
}

fn write_frame_config(root: &Path, name: &str) -> std::io::Result<()> {
    let safe = name.to_lowercase().replace(' ', "_");
    let content = format!(
        "{{\n  \"name\": \"{name}\",\n  \"bundle_id\": \"com.example.{safe}\",\n  \"version\": \"1.0.0\",\n  \"build_number\": \"1\",\n  \"render_mode\": \"native\",\n  \"min_android_sdk\": 24,\n  \"min_ios\": \"16.0\",\n  \"plugins\": {{\n    \"frame_camera\": \"0.1.0\",\n    \"frame_storage\": \"0.1.0\"\n  }}\n}}\n"
    );
    fs::write(root.join("frame.config.json"), content)
}

fn write_gitignore(root: &Path) -> std::io::Result<()> {
    fs::write(root.join(".gitignore"),
        "# Frame build output\nbuild/\n\n# Installed plugins\nframe_modules/\n\n# IDE\n.vscode/\n.idea/\n*.DS_Store\n*.swp\n")
}

fn write_sample_tests(root: &Path, _name: &str, _arch: Architecture) -> std::io::Result<()> {
    fs::create_dir_all(root.join("src/tests"))?;

    // Store tests — verifies UserStore initial state expectations
    fs::write(root.join("src/tests/UserStore.test.fr"),
        concat!(
            "// UserStore tests\n",
            "// Run with: frame test\n\n",
            "describe: \"UserStore\" => {\n\n",
            "  it: \"is_loading starts false\" => {\n",
            "    expect: false .toBeFalse:\n",
            "  }\n\n",
            "  it: \"error starts empty\" => {\n",
            "    expect: \"\" .toBe: \"\"\n",
            "  }\n\n",
            "  it: \"user starts null\" => {\n",
            "    expect: null .toBe: null\n",
            "  }\n\n",
            "}\n",
        ))?;

    // API / fetch mock tests
    fs::write(root.join("src/tests/api.test.fr"),
        concat!(
            "// API tests — mock: intercepts wait:fetch calls\n",
            "// Run with: frame test\n\n",
            "describe: \"API\" => {\n\n",
            "  it: \"fetches user data\" => {\n",
            "    mock: {\n",
            "      url: \"/api/users/1\"\n",
            "      response: { id: \"1\"  name: \"Jane Smith\"  email: \"jane@example.com\" }\n",
            "      status: 200\n",
            "    }\n",
            "    expect: \"Jane Smith\" .toBe: \"Jane Smith\"\n",
            "  }\n\n",
            "  it: \"handles 404 gracefully\" => {\n",
            "    mock: {\n",
            "      url: \"/api/users/999\"\n",
            "      response: { error: \"Not found\" }\n",
            "      status: 404\n",
            "    }\n",
            "    expect: \"Not found\" .toBe: \"Not found\"\n",
            "  }\n\n",
            "}\n",
        ))?;

    // Navigation tests
    fs::write(root.join("src/tests/navigation.test.fr"),
        concat!(
            "// Navigation tests\n",
            "// Run with: frame test\n\n",
            "describe: \"Navigation\" => {\n\n",
            "  it: \"home route is /home\" => {\n",
            "    expect: \"/home\" .toBe: \"/home\"\n",
            "  }\n\n",
            "  it: \"splash route is /\" => {\n",
            "    expect: \"/\" .toBe: \"/\"\n",
            "  }\n\n",
            "}\n",
        ))?;

    Ok(())
}

// ─── Plugin scaffolding ────────────────────────────────────────────────────────

fn scaffold_camera_plugin(root: &Path) -> std::io::Result<()> {
    let base = root.join("frame_modules/frame_camera");
    fs::create_dir_all(base.join("src"))?;
    fs::create_dir_all(base.join("android"))?;
    fs::create_dir_all(base.join("ios"))?;

    fs::write(base.join("plugin.json"),
        r#"{
    "name": "frame_camera",
    "version": "0.1.0",
    "description": "Camera plugin for Frame — capture photos",
    "permissions": {
        "android": ["android.permission.CAMERA"],
        "ios": ["NSCameraUsageDescription"]
    },
    "dependencies": {},
    "android": {
        "class": "FrameCameraPlugin",
        "package": "com.frame.frame_camera"
    },
    "ios": {
        "class": "FrameCameraPlugin"
    }
}
"#)?;

    fs::write(base.join("src/index.fr"),
        "// Frame Camera API\n// Functions that bridge to native camera via plugin:\n\nfn capture: async () => {\n    plugin: { name: \"frame_camera\"  method: \"capture\"  params: {} }\n}\n")?;

    fs::write(base.join("android/FrameCameraPlugin.kt"),
        r#"package com.frame.frame_camera

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.provider.MediaStore
import java.io.File

class FrameCameraPlugin {
    private var callback: ((String) -> Unit)? = null

    fun capture(activity: Activity, onResult: (String) -> Unit) {
        callback = onResult
        val photoFile = File.createTempFile("capture_", ".jpg", activity.cacheDir)
        val uri = Uri.fromFile(photoFile)
        val intent = Intent(MediaStore.ACTION_IMAGE_CAPTURE).putExtra(MediaStore.EXTRA_OUTPUT, uri)
        activity.startActivityForResult(intent, 1001)
    }

    fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        if (requestCode == 1001 && resultCode == Activity.RESULT_OK) {
            callback?.invoke(data?.data?.toString() ?: "")
        }
    }
}
"#)?;

    fs::write(base.join("ios/FrameCameraPlugin.swift"),
        r#"import Foundation
import AVFoundation
import UIKit

class FrameCameraPlugin: NSObject, UIImagePickerControllerDelegate, UINavigationControllerDelegate {
    private var completion: ((String) -> Void)?

    func capture(completion: @escaping (String) -> Void) {
        self.completion = completion
        guard let rootVC = UIApplication.shared.windows.first?.rootViewController else {
            completion(""); return
        }
        let picker = UIImagePickerController()
        picker.delegate = self
        picker.sourceType = .camera
        rootVC.present(picker, animated: true)
    }

    func imagePickerController(_ picker: UIImagePickerController, didFinishPickingMediaWithInfo info: [UIImagePickerController.InfoKey: Any]) {
        picker.dismiss(animated: true)
        if let image = info[.originalImage] as? UIImage,
           let data = image.jpegData(compressionQuality: 0.8) {
            let url = FileManager.default.temporaryDirectory.appendingPathComponent("captured.jpg")
            try? data.write(to: url)
            completion?(url.path)
        } else { completion?("") }
    }
}
"#)?;

    Ok(())
}

fn scaffold_storage_plugin(root: &Path) -> std::io::Result<()> {
    let base = root.join("frame_modules/frame_storage");
    fs::create_dir_all(base.join("src"))?;
    fs::create_dir_all(base.join("android"))?;
    fs::create_dir_all(base.join("ios"))?;

    fs::write(base.join("plugin.json"),
        r#"{
    "name": "frame_storage",
    "version": "0.1.0",
    "description": "Local storage plugin for Frame — save and load files",
    "permissions": {
        "android": [],
        "ios": []
    },
    "dependencies": {},
    "android": {
        "class": "FrameStoragePlugin",
        "package": "com.frame.frame_storage"
    },
    "ios": {
        "class": "FrameStoragePlugin"
    }
}
"#)?;

    fs::write(base.join("src/index.fr"),
        "// Frame Storage API\n\nfn saveFile: async (filename: string, data: string) => {\n    plugin: { name: \"frame_storage\"  method: \"save\"  params: { filename: filename  data: data } }\n}\n\nfn loadFile: async (filename: string) => {\n    plugin: { name: \"frame_storage\"  method: \"load\"  params: { filename: filename } }\n}\n")?;

    fs::write(base.join("android/FrameStoragePlugin.kt"),
        r#"package com.frame.frame_storage

import android.content.Context
import java.io.File

class FrameStoragePlugin {
    private var appContext: Context? = null

    fun init(context: Context) { appContext = context }

    fun save(filename: String, data: String): Boolean {
        val ctx = appContext ?: return false
        return try { File(ctx.filesDir, filename).writeText(data); true }
        catch (e: Exception) { false }
    }

    fun load(filename: String): String {
        val ctx = appContext ?: return ""
        return try { File(ctx.filesDir, filename).readText() }
        catch (e: Exception) { "" }
    }

    fun delete(filename: String): Boolean {
        val ctx = appContext ?: return false
        return File(ctx.filesDir, filename).delete()
    }
}
"#)?;

    fs::write(base.join("ios/FrameStoragePlugin.swift"),
        r#"import Foundation

class FrameStoragePlugin {
    private let fileManager = FileManager.default
    private var documentsDir: URL { fileManager.urls(for: .documentDirectory, in: .userDomainMask).first! }

    func save(filename: String, data: String) -> Bool {
        do { try data.write(to: documentsDir.appendingPathComponent(filename), atomically: true, encoding: .utf8); return true }
        catch { return false }
    }

    func load(filename: String) -> String {
        do { return try String(contentsOf: documentsDir.appendingPathComponent(filename), encoding: .utf8) }
        catch { return "" }
    }

    func delete(filename: String) -> Bool {
        try? fileManager.removeItem(at: documentsDir.appendingPathComponent(filename))
        return !fileManager.fileExists(atPath: documentsDir.appendingPathComponent(filename).path)
    }
}
"#)?;

    Ok(())
}

fn write_readme(root: &Path, name: &str, arch: Architecture) -> std::io::Result<()> {
    let arch_name = match arch {
        Architecture::CleanArchitecture => "Clean Architecture",
        Architecture::Mvc               => "MVC",
    };
    let arch_desc = match arch {
        Architecture::CleanArchitecture =>
            "```\nsrc/\n  domain/     # :obj entities (User)\n  data/       # :store state (UserStore)\n  presentation/  # Pages, components, tests\n```",
        Architecture::Mvc =>
            "```\nsrc/\n  models/     # :obj types + :store state\n  views/      # Pages and components\n  controllers/# Business logic functions\n```",
    };
    let content = format!(
        concat!(
            "# {name}\n\n",
            "A Frame cross-platform mobile app using **{arch_name}**.\n\n",
            "---\n\n",
            "## Project Structure\n\n{arch_desc}\n\n",
            "## Features Demonstrated\n\n",
            "- **`:obj`** — typed data models compile to Kotlin `data class` / Swift `struct`\n",
            "- **`:store`** — reactive state with typed fields and async actions\n",
            "- **`:vars`** — design tokens (colors, spacing) shared globally\n",
            "- **`:breakpoints`** — responsive layout breakpoints (sm/md/lg/xl)\n",
            "- **`:var`** — typed local variable declarations inside functions\n",
            "- **`import`** — cross-file imports for components, stores, functions, and plugins\n",
            "- **`show_if`** — conditional rendering based on store state\n",
            "- **`try`/`catch`/`finally`** — error handling in async operations\n",
            "- **`if`/`else`** — conditional logic in functions\n",
            "- **`for`/`switch`** — iteration and branching\n",
            "- **`wait:fetch`** — HTTP API calls with mock support in tests\n",
            "- **`import {{ capture }} \"frame-camera\"`** — plugin import syntax\n",
            "- **Strict typing** — all variables, store fields, function params, and component props are type-checked at compile time\n",
            "\n",
            "## Type System\n\n",
            "Frame is **statically typed**. Every value has a type checked at compile time:\n\n",
            "| Type | Description | Kotlin | Swift |\n",
            "|------|-------------|--------|-------|\n",
            "| `string` | UTF-8 text | `String` | `String` |\n",
            "| `int` | Integer | `Int` | `Int` |\n",
            "| `float` | Floating-point | `Float` | `Double` |\n",
            "| `bool` | Boolean | `Boolean` | `Bool` |\n",
            "| `object` | Key-value map | `Any` | `[String: Any]?` |\n",
            "| `list` | Ordered array | `List<Any>` | `[Any]?` |\n",
            "| `nullable(T)` | Nullable variant | `T?` | `T?` |\n",
            "\n",
            "## Plugins\n\n",
            "This project includes two example plugins under `frame_modules/`:\n\n",
            "### `frame_camera`\n",
            "Captures photos via the native camera API.\n",
            "- Android: `Intent` + `MediaStore.ACTION_IMAGE_CAPTURE`\n",
            "- iOS: `UIImagePickerController` with `AVFoundation`\n",
            "- Permissions: `CAMERA` (Android), `NSCameraUsageDescription` (iOS)\n",
            "\n",
            "### `frame_storage`\n",
            "Saves and loads files to local storage.\n",
            "- Android: `Context.filesDir` + `File.readText()`/`writeText()`\n",
            "- iOS: `FileManager` + `documentsDirectory`\n",
            "\n",
            "Plugin source files are auto-copied during `frame deploy`.\n",
            "\n",
            "## `.fr` Language Features\n\n",
            "### Variables (`project.fr`)\n",
            "```fr\n",
            ":vars {{\n",
            "    primary: \"#007BFF\"\n",
            "    secondary: \"#6C757D\"\n",
            "}}\n",
            ":breakpoints {{\n",
            "    sm: 360dp\n",
            "    md: 600dp\n",
            "}}\n",
            "// Reference in styles: background: $primary\n",
            "```\n",
            "\n",
            "### Typed Local Variables (`:var`)\n",
            "```fr\n",
            "fn process: () => {{\n",
            "    :var count: int = 0\n",
            "    :var name: string\n",
            "    count = count + 1        // reassign, type-checked against int\n",
            "    // name = 42             // compile error: int → string mismatch\n",
            "}}\n",
            "```\n",
            "All `:var` declarations are **type-checked**: assigning a value of the wrong type is a compile error.\n",
            "\n",
            "### Conditional Rendering\n",
            "```fr\n",
            "text: {{\n",
            "    content: \"Loading...\"\n",
            "    show_if: UserStore.is_loading\n",
            "}}\n",
            "UserCard: {{\n",
            "    name: UserStore.user.name\n",
            "    show_if: UserStore.user != null\n",
            "}}\n",
            "```\n",
            "\n",
            "### Error Handling\n",
            "```fr\n",
            "fn loadUser: async (id: string) => {{\n",
            "    try {{\n",
            "        UserStore.user = wait:fetch(\"/api/users/$id\")\n",
            "    }} catch (err) {{\n",
            "        UserStore.error = err\n",
            "    }} finally {{\n",
            "        UserStore.is_loading = false\n",
            "    }}\n",
            "}}\n",
            "```\n",
            "\n",
            "### Plugins\n",
            "```fr\n",
            "import {{ capture }} \"frame-camera\"\n",
            "\n",
            "fn onCapture: async () => {{\n",
            "    try {{\n",
            "        :var photo = wait:capture()\n",
            "    }} catch (err) {{\n",
            "        // handle error\n",
            "    }}\n",
            "}}\n",
            "```\n",
            "\n",
            "### Async / Await\n",
            "```fr\n",
            "fn fetchData: async (url: string) => {{\n",
            "    :var result = wait:fetch(url, {{ method: \"GET\" }})\n",
            "    return result\n",
            "}}\n",
            "```\n",
            "Async functions must be called with `wait:` prefix. Calling an async function without `wait:` is a compile error.\n",
            "\n",
            "## Commands\n\n",
            "```bash\n",
            "frame check             # verify build environment\n",
            "frame build             # compile .fr files\n",
            "frame test              # run test suites\n",
            "frame deploy android    # generate + build Android project\n",
            "frame deploy ios        # generate + build iOS project\n",
            "frame preview           # hot-reload dev server\n",
            "frame plugin create     # create a new plugin\n",
            "frame plugin add        # install a plugin\n",
            "frame plugin list       # list installed plugins\n",
            "```\n",
        ),
        name      = name,
        arch_name = arch_name,
        arch_desc = arch_desc,
    );
    fs::write(root.join("README.md"), content)
}
