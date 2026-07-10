//! Android / Jetpack Compose code generator for the Frame framework.
//!
//! Entry point: `gen_android(ast, config) -> Vec<OutputFile>`

use crate::parser::ast::*;
use std::collections::HashMap;

// ─── Config / OutputFile ──────────────────────────────────────────────────────

/// Android project configuration.
#[derive(Debug, Clone)]
pub struct AndroidConfig {
    pub application_id: String,
    pub min_sdk: u32,
    pub target_sdk: u32,
    pub version_code: u32,
    pub version_name: String,
    pub app_name: String,
}

impl Default for AndroidConfig {
    fn default() -> Self {
        AndroidConfig {
            application_id: "com.example.frameapp".to_string(),
            min_sdk: 24,
            target_sdk: 34,
            version_code: 1,
            version_name: "1.0".to_string(),
            app_name: "Frame App".to_string(),
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

/// Generate all Android project files from an AST + config.
pub fn gen_android(ast: &AST, config: &AndroidConfig) -> Vec<OutputFile> {
    gen_android_with_plugins(ast, config, &[])
}

/// Like [`gen_android`] but also accepts extra Kotlin source files from plugins.
/// Each entry is `(package_dir, filename, source_content)` — files are placed at
/// `app/src/main/java/{package_dir}/{filename}`.
pub fn gen_android_with_plugins(
    ast: &AST,
    config: &AndroidConfig,
    extra_kt_sources: &[(&str, &str, &str)],
) -> Vec<OutputFile> {
    let mut files: Vec<OutputFile> = Vec::new();
    let pkg = &config.application_id;
    let pkg_path = pkg.replace('.', "/");

    // ── Feature detection ──────────────────────────────────────────────────────
    // Network
    let uses_fetch          = ast_uses_fetch(ast);
    // Camera & media
    let uses_camera         = ast_uses_call(ast, "camera:capture");
    let uses_audio_record   = ast_uses_call(ast, "audio:record");
    // Location
    let uses_location_fine  = ast_uses_call(ast, "location:get")
                           || ast_uses_call(ast, "location:watch");
    let uses_location_coarse= ast_uses_call(ast, "location:coarse");
    let uses_location_bg    = ast_uses_call(ast, "location:background");
    // Notifications & connectivity
    let uses_notification   = ast_uses_call(ast, "notification:send");
    let uses_bt_connect     = ast_uses_call(ast, "bluetooth:connect");
    let uses_bt_scan        = ast_uses_call(ast, "bluetooth:scan");
    let uses_activity_rec   = ast_uses_call(ast, "health:steps");
    // Storage
    let uses_read_images    = ast_uses_call(ast, "storage:images") || uses_camera;
    let uses_read_video     = ast_uses_call(ast, "storage:video");
    let uses_read_audio     = ast_uses_call(ast, "storage:audio") || uses_audio_record;
    let uses_manage_storage = ast_uses_call(ast, "storage:manage");
    // Contacts & calendar
    let uses_read_contacts  = ast_uses_call(ast, "contacts:read");
    let uses_write_contacts = ast_uses_call(ast, "contacts:write");
    let uses_read_calendar  = ast_uses_call(ast, "calendar:read");
    let uses_write_calendar = ast_uses_call(ast, "calendar:write");
    // Telephony
    let uses_phone_state    = ast_uses_call(ast, "phone:state");
    let uses_call_phone     = ast_uses_call(ast, "phone:call");
    let uses_read_sms       = ast_uses_call(ast, "sms:read");
    let uses_send_sms       = ast_uses_call(ast, "sms:send");
    let uses_call_log       = ast_uses_call(ast, "phone:call_log");

    files.push(gen_settings_gradle(&config.app_name));
    files.push(gen_top_level_build_gradle());
    files.push(gen_app_build_gradle(config, uses_fetch));
    files.push(gen_gradle_wrapper());
    files.push(gen_gradlew_script());
    files.push(gen_gradlew_bat_script());
    files.push(gen_res_themes(config));
    files.push(gen_res_colors());
    files.push(gen_res_strings(config));
    files.push(gen_proguard_rules());
    files.push(gen_gitignore_android());
    files.push(gen_manifest_full(
        config,
        uses_fetch,
        uses_camera, uses_audio_record,
        uses_location_fine, uses_location_coarse, uses_location_bg,
        uses_notification, uses_bt_connect, uses_bt_scan, uses_activity_rec,
        uses_read_images, uses_read_video, uses_read_audio, uses_manage_storage,
        uses_read_contacts, uses_write_contacts, uses_read_calendar, uses_write_calendar,
        uses_phone_state, uses_call_phone, uses_read_sms, uses_send_sms, uses_call_log,
    ));
    files.push(gen_main_application(pkg));
    files.push(gen_main_activity(ast, pkg));

    for page in &ast.pages {
        files.push(gen_page_screen(page, ast, pkg, &pkg_path));
    }

    for (name, comp) in &ast.components {
        files.push(gen_component_file(name, comp, pkg, &pkg_path));
    }

    for (name, store) in &ast.stores {
        files.push(gen_store_viewmodel(name, store, pkg, &pkg_path));
    }

    // :obj type declarations → Kotlin data classes
    for obj in ast.objects.values() {
        files.push(gen_obj_data_class(obj, pkg, &pkg_path));
    }

    if uses_camera        { files.push(gen_camera_helper(pkg, &pkg_path)); }
    if uses_location_fine { files.push(gen_location_helper(pkg, &pkg_path)); }
    if uses_notification  { files.push(gen_notification_helper(pkg, &pkg_path)); }

    // Plugin Kotlin sources — placed in their own package directory
    for (pkg_dir, fname, content) in extra_kt_sources {
        files.push(OutputFile {
            path: format!("app/src/main/java/{pkg_dir}/{fname}"),
            content: content.to_string(),
        });
    }

    files
}

// ─── Feature detection helpers ────────────────────────────────────────────────

/// Returns true if any function in the AST contains a WaitFetch statement.
pub fn ast_uses_fetch(ast: &AST) -> bool {
    ast.functions.values().any(|f| stmts_use_fetch(&f.body))
        || ast.pages.iter().any(|p| {
            p.children.iter().any(|n| node_funcs_use_fetch(n))
        })
        || ast.components.values().any(|c| {
            c.functions.values().any(|f| stmts_use_fetch(&f.body))
                || c.children.iter().any(|n| node_funcs_use_fetch(n))
        })
        || ast.stores.values().any(|s| {
            s.actions.values().any(|f| stmts_use_fetch(&f.body))
        })
}

fn stmts_use_fetch(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|s| stmt_uses_fetch(s))
}

fn stmt_uses_fetch(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::WaitFetch(_) => true,
        Stmt::If(_, then, else_) => {
            stmts_use_fetch(then)
                || else_.as_ref().map(|e| stmts_use_fetch(e)).unwrap_or(false)
        }
        Stmt::For(_, _, body) => stmts_use_fetch(body),
        Stmt::Switch(_, cases) => cases.iter().any(|(_, b)| stmts_use_fetch(b)),
        Stmt::TryCatch { body, catch_body, finally_body, .. } => {
            stmts_use_fetch(body)
                || stmts_use_fetch(catch_body)
                || finally_body.as_ref().map(|f| stmts_use_fetch(f)).unwrap_or(false)
        }
        _ => false,
    }
}

fn node_funcs_use_fetch(node: &ComponentNode) -> bool {
    if let Some(build) = &node.build {
        if stmts_use_fetch(&build.body) {
            return true;
        }
    }
    node.children.iter().any(|c| node_funcs_use_fetch(c))
}

/// Returns true if any CallExpr in the AST matches `func_name`.
pub fn ast_uses_call(ast: &AST, func_name: &str) -> bool {
    ast.functions.values().any(|f| stmts_use_call(&f.body, func_name))
        || ast.pages.iter().any(|p| {
            p.children.iter().any(|n| node_uses_call(n, func_name))
        })
        || ast.components.values().any(|c| {
            c.functions.values().any(|f| stmts_use_call(&f.body, func_name))
                || c.children.iter().any(|n| node_uses_call(n, func_name))
        })
}

fn stmts_use_call(stmts: &[Stmt], func_name: &str) -> bool {
    stmts.iter().any(|s| stmt_uses_call(s, func_name))
}

fn stmt_uses_call(stmt: &Stmt, func_name: &str) -> bool {
    match stmt {
        Stmt::Call(c) | Stmt::Wait(c) => c.func == func_name,
        Stmt::If(_, then, else_) => {
            stmts_use_call(then, func_name)
                || else_.as_ref().map(|e| stmts_use_call(e, func_name)).unwrap_or(false)
        }
        Stmt::For(_, _, body) => stmts_use_call(body, func_name),
        Stmt::Switch(_, cases) => cases.iter().any(|(_, b)| stmts_use_call(b, func_name)),
        Stmt::TryCatch { body, catch_body, finally_body, .. } => {
            stmts_use_call(body, func_name)
                || stmts_use_call(catch_body, func_name)
                || finally_body.as_ref().map(|f| stmts_use_call(f, func_name)).unwrap_or(false)
        }
        _ => false,
    }
}

fn node_uses_call(node: &ComponentNode, func_name: &str) -> bool {
    // Check event handlers
    let events: [Option<&Expr>; 3] = [
        node.events.on_click.as_ref(),
        node.events.on_change.as_ref(),
        node.events.on_submit.as_ref(),
    ];
    for ev in events.iter().flatten() {
        if expr_uses_call(ev, func_name) {
            return true;
        }
    }
    if let Some(build) = &node.build {
        if stmts_use_call(&build.body, func_name) {
            return true;
        }
    }
    node.children.iter().any(|c| node_uses_call(c, func_name))
}

fn expr_uses_call(expr: &Expr, func_name: &str) -> bool {
    match expr {
        Expr::Call(c) => c.func == func_name || c.args.iter().any(|a| expr_uses_call(a, func_name)),
        Expr::Lambda(_, stmts) => stmts_use_call(stmts, func_name),
        _ => false,
    }
}

// ─── Gradle / manifest helpers ────────────────────────────────────────────────

fn gen_settings_gradle(app_name: &str) -> OutputFile {
    // Sanitise: spaces → empty (gradle rootProject.name should be identifier-safe)
    let safe_name = app_name.replace(' ', "");
    OutputFile {
        path: "settings.gradle".to_string(),
        content: format!(
            r#"pluginManagement {{
    repositories {{
        google()
        mavenCentral()
        gradlePluginPortal()
    }}
}}
dependencyResolutionManagement {{
    repositories {{
        google()
        mavenCentral()
    }}
}}
rootProject.name = "{safe_name}"
include ':app'
"#
        ),
    }
}

fn gen_top_level_build_gradle() -> OutputFile {
    OutputFile {
        path: "build.gradle".to_string(),
        content: r#"plugins {
    id 'com.android.application' version '8.1.0' apply false
    id 'org.jetbrains.kotlin.android' version '1.9.0' apply false
}
"#
        .to_string(),
    }
}

fn gen_app_build_gradle(config: &AndroidConfig, uses_fetch: bool) -> OutputFile {
    let okhttp_dep = if uses_fetch {
        "    implementation 'com.squareup.okhttp3:okhttp:4.12.0'"
    } else {
        ""
    };
    OutputFile {
        path: "app/build.gradle".to_string(),
        content: format!(
            r#"plugins {{
    id 'com.android.application'
    id 'org.jetbrains.kotlin.android'
}}

android {{
    namespace '{app_id}'
    compileSdk {target_sdk}
    defaultConfig {{
        applicationId "{app_id}"
        minSdk {min_sdk}
        targetSdk {target_sdk}
        versionCode {version_code}
        versionName "{version_name}"
    }}
    buildFeatures {{ compose true }}
    composeOptions {{ kotlinCompilerExtensionVersion '1.5.3' }}
    compileOptions {{
        sourceCompatibility JavaVersion.VERSION_17
        targetCompatibility JavaVersion.VERSION_17
    }}
    kotlinOptions {{ jvmTarget = '17' }}
}}

dependencies {{
    implementation 'androidx.core:core-ktx:1.12.0'
    implementation 'androidx.lifecycle:lifecycle-runtime-ktx:2.6.2'
    implementation 'androidx.activity:activity-compose:1.8.0'
    implementation platform('androidx.compose:compose-bom:2023.10.01')
    implementation 'androidx.compose.ui:ui'
    implementation 'androidx.compose.material3:material3'
    implementation 'androidx.navigation:navigation-compose:2.7.4'
    implementation 'io.coil-kt:coil-compose:2.4.0'
{okhttp_dep}
}}
"#,
            app_id = config.application_id,
            target_sdk = config.target_sdk,
            min_sdk = config.min_sdk,
            version_code = config.version_code,
            version_name = config.version_name,
        ),
    }
}

fn gen_gradle_wrapper() -> OutputFile {
    OutputFile {
        path: "gradle/wrapper/gradle-wrapper.properties".to_string(),
        content: r#"distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-8.4-bin.zip
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists
"#
        .to_string(),
    }
}

/// Unix gradlew shell script — must be marked executable after writing (chmod +x).
fn gen_gradlew_script() -> OutputFile {
    OutputFile {
        path: "gradlew".to_string(),
        content: r#"#!/bin/sh
# Gradle wrapper script for Unix/macOS
# Generated by Frame framework

APP_HOME="$(cd "$(dirname "$0")" && pwd -P)"
CLASSPATH="$APP_HOME/gradle/wrapper/gradle-wrapper.jar"
JAVACMD="${JAVA_HOME:+$JAVA_HOME/bin/java}"
JAVACMD="${JAVACMD:-java}"

exec "$JAVACMD" -classpath "$CLASSPATH" \
  org.gradle.wrapper.GradleWrapperMain "$@"
"#
        .to_string(),
    }
}

/// Windows gradlew.bat script.
fn gen_gradlew_bat_script() -> OutputFile {
    OutputFile {
        path: "gradlew.bat".to_string(),
        content: r#"@rem Gradle wrapper script for Windows
@rem Generated by Frame framework
@if "%DEBUG%"=="" @echo off
setlocal

set DIRNAME=%~dp0
if "%DIRNAME%"=="" set DIRNAME=.
set APP_BASE_NAME=%~n0
set APP_HOME=%DIRNAME%

set CLASSPATH=%APP_HOME%\gradle\wrapper\gradle-wrapper.jar

set JAVACMD=java
if defined JAVA_HOME set JAVACMD=%JAVA_HOME%\bin\java.exe

%JAVACMD% -classpath "%CLASSPATH%" org.gradle.wrapper.GradleWrapperMain %*
"#
        .to_string(),
    }
}

/// res/values/themes.xml — required for Theme.FrameApp referenced in AndroidManifest.xml.
fn gen_res_themes(config: &AndroidConfig) -> OutputFile {
    let safe = config.app_name.replace(' ', "");
    OutputFile {
        path: "app/src/main/res/values/themes.xml".to_string(),
        content: format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
    <!-- Base app theme. Compose apps use this as the Activity window background only. -->
    <style name="Theme.FrameApp" parent="Theme.MaterialComponents.DayNight.NoActionBar">
        <item name="android:statusBarColor">@color/colorPrimary</item>
    </style>
    <style name="Theme.{safe}" parent="Theme.FrameApp" />
</resources>
"#
        ),
    }
}

/// res/values/colors.xml — standard Material color palette.
fn gen_res_colors() -> OutputFile {
    OutputFile {
        path: "app/src/main/res/values/colors.xml".to_string(),
        content: r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
    <color name="colorPrimary">#6200EE</color>
    <color name="colorPrimaryVariant">#3700B3</color>
    <color name="colorOnPrimary">#FFFFFF</color>
    <color name="colorSecondary">#03DAC5</color>
    <color name="colorSecondaryVariant">#018786</color>
    <color name="colorOnSecondary">#000000</color>
    <color name="colorError">#B00020</color>
    <color name="colorOnError">#FFFFFF</color>
    <color name="colorBackground">#FFFFFF</color>
    <color name="colorOnBackground">#000000</color>
    <color name="colorSurface">#FFFFFF</color>
    <color name="colorOnSurface">#000000</color>
</resources>
"#
        .to_string(),
    }
}

/// res/values/strings.xml — app name and base strings.
fn gen_res_strings(config: &AndroidConfig) -> OutputFile {
    OutputFile {
        path: "app/src/main/res/values/strings.xml".to_string(),
        content: format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="app_name">{}</string>
</resources>
"#,
            config.app_name
        ),
    }
}

/// proguard-rules.pro — required by app/build.gradle even when not used.
fn gen_proguard_rules() -> OutputFile {
    OutputFile {
        path: "app/proguard-rules.pro".to_string(),
        content: r#"# Generated by Frame framework
# Add project-specific ProGuard rules here.
# See: https://developer.android.com/studio/build/shrink-code

# Keep all Frame-generated classes
-keep class com.frame.** { *; }
"#
        .to_string(),
    }
}

/// .gitignore for the generated Android project.
fn gen_gitignore_android() -> OutputFile {
    OutputFile {
        path: ".gitignore".to_string(),
        content: r#"# Generated by Frame framework
*.iml
.gradle
/local.properties
/.idea
.DS_Store
/build
/captures
.externalNativeBuild
.cxx
local.properties
"#
        .to_string(),
    }
}

fn gen_manifest(
    config: &AndroidConfig,
    uses_fetch: bool,
    uses_camera: bool,
    uses_location: bool,
    uses_notification: bool,
) -> OutputFile {
    gen_manifest_full(config, uses_fetch, uses_camera, false, uses_location, false, false,
        uses_notification, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false)
}

/// Full manifest generator covering every permission category.
/// Called directly by gen_android; the simplified wrapper above exists for
/// backward-compatible tests. New callers should detect features and call this.
#[allow(clippy::too_many_arguments)]
pub fn gen_manifest_full(
    config: &AndroidConfig,
    // Network
    uses_fetch: bool,
    // Camera & media
    uses_camera: bool,
    uses_audio_record: bool,
    // Location
    uses_location_fine: bool,
    uses_location_coarse: bool,
    uses_location_background: bool,
    // Notifications & connectivity
    uses_notification: bool,
    uses_bluetooth_connect: bool,
    uses_bluetooth_scan: bool,
    uses_activity_recognition: bool,
    // Storage
    uses_read_images: bool,
    uses_read_video: bool,
    uses_read_audio_files: bool,
    uses_manage_storage: bool,
    // Contacts & calendar
    uses_read_contacts: bool,
    uses_write_contacts: bool,
    uses_read_calendar: bool,
    uses_write_calendar: bool,
    // Telephony
    uses_read_phone_state: bool,
    uses_call_phone: bool,
    uses_read_sms: bool,
    uses_send_sms: bool,
    uses_read_call_log: bool,
) -> OutputFile {
    let mut perms = String::new();

    // ── Network ───────────────────────────────────────────────────────────────
    if uses_fetch {
        perms.push_str("    <uses-permission android:name=\"android.permission.INTERNET\" />\n");
    }

    // ── Camera & Media Capture ────────────────────────────────────────────────
    if uses_camera {
        perms.push_str("    <uses-permission android:name=\"android.permission.CAMERA\" />\n");
    }
    if uses_audio_record {
        perms.push_str("    <uses-permission android:name=\"android.permission.RECORD_AUDIO\" />\n");
    }

    // ── Location ──────────────────────────────────────────────────────────────
    if uses_location_fine {
        perms.push_str("    <uses-permission android:name=\"android.permission.ACCESS_FINE_LOCATION\" />\n");
        // Coarse is implied by fine but explicit is better practice
        perms.push_str("    <uses-permission android:name=\"android.permission.ACCESS_COARSE_LOCATION\" />\n");
    } else if uses_location_coarse {
        perms.push_str("    <uses-permission android:name=\"android.permission.ACCESS_COARSE_LOCATION\" />\n");
    }
    if uses_location_background {
        perms.push_str("    <uses-permission android:name=\"android.permission.ACCESS_BACKGROUND_LOCATION\" />\n");
    }

    // ── Notifications & System ────────────────────────────────────────────────
    if uses_notification {
        perms.push_str("    <uses-permission android:name=\"android.permission.POST_NOTIFICATIONS\" />\n");
    }
    if uses_bluetooth_connect {
        perms.push_str("    <uses-permission android:name=\"android.permission.BLUETOOTH_CONNECT\" />\n");
    }
    if uses_bluetooth_scan {
        perms.push_str("    <uses-permission android:name=\"android.permission.BLUETOOTH_SCAN\" />\n");
    }
    if uses_activity_recognition {
        perms.push_str("    <uses-permission android:name=\"android.permission.ACTIVITY_RECOGNITION\" />\n");
    }

    // ── Storage & Media Files ─────────────────────────────────────────────────
    if uses_read_images {
        perms.push_str("    <uses-permission android:name=\"android.permission.READ_MEDIA_IMAGES\" />\n");
    }
    if uses_read_video {
        perms.push_str("    <uses-permission android:name=\"android.permission.READ_MEDIA_VIDEO\" />\n");
    }
    if uses_read_audio_files {
        perms.push_str("    <uses-permission android:name=\"android.permission.READ_MEDIA_AUDIO\" />\n");
    }
    if uses_manage_storage {
        perms.push_str("    <uses-permission android:name=\"android.permission.MANAGE_EXTERNAL_STORAGE\" />\n");
    }

    // ── Contacts & Calendar ───────────────────────────────────────────────────
    if uses_read_contacts {
        perms.push_str("    <uses-permission android:name=\"android.permission.READ_CONTACTS\" />\n");
    }
    if uses_write_contacts {
        perms.push_str("    <uses-permission android:name=\"android.permission.WRITE_CONTACTS\" />\n");
    }
    if uses_read_calendar {
        perms.push_str("    <uses-permission android:name=\"android.permission.READ_CALENDAR\" />\n");
    }
    if uses_write_calendar {
        perms.push_str("    <uses-permission android:name=\"android.permission.WRITE_CALENDAR\" />\n");
    }

    // ── Telephony & Communications ────────────────────────────────────────────
    if uses_read_phone_state {
        perms.push_str("    <uses-permission android:name=\"android.permission.READ_PHONE_STATE\" />\n");
    }
    if uses_call_phone {
        perms.push_str("    <uses-permission android:name=\"android.permission.CALL_PHONE\" />\n");
    }
    if uses_read_sms {
        perms.push_str("    <uses-permission android:name=\"android.permission.READ_SMS\" />\n");
    }
    if uses_send_sms {
        perms.push_str("    <uses-permission android:name=\"android.permission.SEND_SMS\" />\n");
    }
    if uses_read_call_log {
        perms.push_str("    <uses-permission android:name=\"android.permission.READ_CALL_LOG\" />\n");
    }

    OutputFile {
        path: "app/src/main/AndroidManifest.xml".to_string(),
        content: format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
{perms}    <application
        android:name=".MainApplication"
        android:label="{app_name}"
        android:theme="@style/Theme.FrameApp">
        <activity
            android:name=".MainActivity"
            android:exported="true">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>
    </application>
</manifest>
"#,
            app_name = config.app_name,
        ),
    }
}

// ─── Kotlin entry-point files ─────────────────────────────────────────────────

fn gen_main_application(pkg: &str) -> OutputFile {
    let pkg_path = pkg.replace('.', "/");
    OutputFile {
        path: format!("app/src/main/java/{pkg_path}/MainApplication.kt"),
        content: format!(
            r#"package {pkg}

import android.app.Application

class MainApplication : Application() {{
    override fun onCreate() {{
        super.onCreate()
    }}
}}
"#
        ),
    }
}

fn gen_main_activity(ast: &AST, pkg: &str) -> OutputFile {
    let pkg_path = pkg.replace('.', "/");
    let first_route = ast
        .pages
        .first()
        .map(|p| p.route.clone())
        .unwrap_or_else(|| "/".to_string());

    // Build import lines for each screen
    let screen_imports: String = ast
        .pages
        .iter()
        .map(|p| format!("import {pkg}.{}Screen\n", p.name))
        .collect();

    // Build nav routes
    let nav_routes: String = ast
        .pages
        .iter()
        .map(|p| gen_nav_route(p))
        .collect();

    OutputFile {
        path: format!("app/src/main/java/{pkg_path}/MainActivity.kt"),
        content: format!(
            r#"package {pkg}

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.rememberNavController
import androidx.navigation.compose.composable
{screen_imports}
class MainActivity : ComponentActivity() {{
    override fun onCreate(savedInstanceState: Bundle?) {{
        super.onCreate(savedInstanceState)
        setContent {{
            val navController = rememberNavController()
            NavHost(navController = navController, startDestination = "{first_route}") {{
{nav_routes}            }}
        }}
    }}
}}
"#
        ),
    }
}

// ─── Per-page screen ──────────────────────────────────────────────────────────

fn gen_page_screen(page: &Page, _ast: &AST, pkg: &str, pkg_path: &str) -> OutputFile {
    let state_vars = gen_state_vars(&page.state);

    let before_enter = if let Some(func_name) = &page.before_enter {
        format!(
            "    LaunchedEffect(Unit) {{\n        {func_name}()\n    }}\n"
        )
    } else {
        String::new()
    };

    let before_leave = if let Some(func_name) = &page.before_leave {
        format!(
            "    DisposableEffect(Unit) {{\n        onDispose {{ {func_name}() }}\n    }}\n"
        )
    } else {
        String::new()
    };

    let children_code: String = page
        .children
        .iter()
        .map(|n| emit_composable(n, 1))
        .collect::<Vec<_>>()
        .join("\n");

    // Collect per-page fetch blocks
    let fetch_blocks = collect_fetch_stmts_from_page(page);
    let fetch_code: String = fetch_blocks
        .iter()
        .map(|fe| emit_fetch_block(fe, 1))
        .collect::<Vec<_>>()
        .join("\n");

    // Detect if animations are used
    let anim_import = if page.children.iter().any(|n| !n.animate.is_empty()) {
        "import android.animation.ValueAnimator\n"
    } else {
        ""
    };

    let has_fetch = !fetch_blocks.is_empty();
    let fetch_imports = if has_fetch {
        "import kotlinx.coroutines.Dispatchers\nimport kotlinx.coroutines.withContext\nimport okhttp3.OkHttpClient\nimport okhttp3.Request\n"
    } else {
        ""
    };

    OutputFile {
        path: format!("app/src/main/java/{pkg_path}/{}Screen.kt", page.name),
        content: format!(
            r#"package {pkg}

import androidx.compose.runtime.*
import androidx.compose.material3.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.background
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.draw.clip
import androidx.navigation.NavController
import coil.compose.AsyncImage
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
{fetch_imports}{anim_import}
@Composable
fun {name}Screen(navController: NavController) {{
{state_vars}{before_enter}{before_leave}{fetch_code}{children_code}
}}
"#,
            name = page.name,
        ),
    }
}

fn gen_state_vars(state: &HashMap<String, StateField>) -> String {
    let mut lines: Vec<String> = state
        .values()
        .map(|f| {
            let default = f
                .default
                .as_ref()
                .map(|e| emit_expr(e))
                .unwrap_or_else(|| default_for_type(&f.type_));
            format!("    var {} by remember {{ mutableStateOf({}) }}\n", f.name, default)
        })
        .collect();
    lines.sort(); // stable output
    lines.join("")
}

fn collect_fetch_stmts_from_page(page: &Page) -> Vec<FetchExpr> {
    // Collect from children's build functions
    let mut result = Vec::new();
    for child in &page.children {
        collect_fetch_from_node(child, &mut result);
    }
    result
}

fn collect_fetch_from_node(node: &ComponentNode, out: &mut Vec<FetchExpr>) {
    if let Some(build) = &node.build {
        collect_fetch_from_stmts(&build.body, out);
    }
    for child in &node.children {
        collect_fetch_from_node(child, out);
    }
}

fn collect_fetch_from_stmts(stmts: &[Stmt], out: &mut Vec<FetchExpr>) {
    for stmt in stmts {
        match stmt {
            Stmt::WaitFetch(fe) => out.push(fe.clone()),
            Stmt::If(_, then, else_) => {
                collect_fetch_from_stmts(then, out);
                if let Some(e) = else_ {
                    collect_fetch_from_stmts(e, out);
                }
            }
            Stmt::For(_, _, body) => collect_fetch_from_stmts(body, out),
            Stmt::TryCatch { body, catch_body, finally_body, .. } => {
                collect_fetch_from_stmts(body, out);
                collect_fetch_from_stmts(catch_body, out);
                if let Some(f) = finally_body {
                    collect_fetch_from_stmts(f, out);
                }
            }
            _ => {}
        }
    }
}

// ─── Custom component files ───────────────────────────────────────────────────

fn gen_component_file(name: &str, comp: &ComponentDef, pkg: &str, pkg_path: &str) -> OutputFile {
    let state_vars = gen_state_vars(&comp.state);

    // Props as function parameters
    let props_params: String = if comp.props.is_empty() {
        String::new()
    } else {
        let mut params: Vec<String> = comp
            .props
            .values()
            .map(|p| {
                let kt_type = frtype_to_kotlin(&p.type_);
                let default_val = p
                    .default
                    .as_ref()
                    .map(|e| format!(" = {}", emit_expr(e)))
                    .unwrap_or_default();
                format!("{}: {}{}", p.name, kt_type, default_val)
            })
            .collect();
        params.sort();
        params.join(", ")
    };

    let children_code: String = comp
        .children
        .iter()
        .map(|n| emit_composable(n, 1))
        .collect::<Vec<_>>()
        .join("\n");

    // Animation blocks for each animated child
    let anim_code: String = comp
        .animate
        .iter()
        .map(|a| emit_animation(a, 1))
        .collect::<Vec<_>>()
        .join("\n");

    let has_anim = !comp.animate.is_empty();
    let anim_import = if has_anim {
        "import android.animation.ValueAnimator\n"
    } else {
        ""
    };

    OutputFile {
        path: format!("app/src/main/java/{pkg_path}/{name}.kt"),
        content: format!(
            r#"package {pkg}

import androidx.compose.runtime.*
import androidx.compose.material3.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.background
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.draw.clip
import coil.compose.AsyncImage
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
{anim_import}
@Composable
fun {name}({props_params}) {{
{state_vars}{anim_code}{children_code}
}}
"#
        ),
    }
}

// ─── Navigation route helper ──────────────────────────────────────────────────

/// Generate a composable nav route entry for a page, supporting route params.
///
/// For a route like `/profile/:userId`, generates:
/// ```kotlin
/// composable(
///     route = "/profile/{userId}",
///     arguments = listOf(navArgument("userId") { type = NavType.StringType })
/// ) { backStackEntry ->
///     val userId = backStackEntry.arguments?.getString("userId")
///     ProfileScreen(navController = navController, userId = userId)
/// }
/// ```
fn gen_nav_route(page: &Page) -> String {
    let raw_route = &page.route;
    let name = &page.name;

    // Extract param names from `:paramName` patterns
    let params: Vec<String> = raw_route
        .split('/')
        .filter(|seg| seg.starts_with(':'))
        .map(|seg| seg.trim_start_matches(':').to_string())
        .collect();

    if params.is_empty() {
        // Simple route — no params
        return format!(
            "                composable(\"{raw_route}\") {{ {name}Screen(navController = navController) }}\n"
        );
    }

    // Replace :param → {param}
    let kotlin_route = raw_route
        .split('/')
        .map(|seg| {
            if seg.starts_with(':') {
                format!("{{{}}}", seg.trim_start_matches(':'))
            } else {
                seg.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("/");

    // navArgument list
    let nav_args: String = params
        .iter()
        .map(|p| format!("navArgument(\"{p}\") {{ type = NavType.StringType }}"))
        .collect::<Vec<_>>()
        .join(", ");

    // backStackEntry extractions
    let extractions: String = params
        .iter()
        .map(|p| format!("                    val {p} = backStackEntry.arguments?.getString(\"{p}\")\n"))
        .collect();

    // extra params for Screen composable
    let extra_params: String = params
        .iter()
        .map(|p| format!(", {p} = {p}"))
        .collect();

    format!(
        "                composable(\n                    route = \"{kotlin_route}\",\n                    arguments = listOf({nav_args})\n                ) {{ backStackEntry ->\n{extractions}                    {name}Screen(navController = navController{extra_params})\n                }}\n"
    )
}

// ─── Store ViewModel generator ────────────────────────────────────────────────

/// Generate a `*StoreViewModel.kt` file for a store slice.
fn gen_store_viewmodel(name: &str, store: &StoreSlice, pkg: &str, pkg_path: &str) -> OutputFile {
    let class_name = format!("{}ViewModel", pascal_case(name));

    // ── State flow fields ─────────────────────────────────────────────────────
    let mut state_flow_lines: Vec<String> = store
        .fields
        .values()
        .map(|f| {
            let kt_type = frtype_to_kotlin(&f.type_);
            let default = f
                .default
                .as_ref()
                .map(|e| emit_expr(e))
                .unwrap_or_else(|| default_for_type(&f.type_));
            format!(
                "    private val _{field} = MutableStateFlow({default})\n    val {field}: StateFlow<{kt_type}> = _{field}.asStateFlow()\n",
                field = f.name,
            )
        })
        .collect();
    state_flow_lines.sort();
    let state_flow_fields = state_flow_lines.join("\n");

    // ── Restore logic ─────────────────────────────────────────────────────────
    let mut restore_lines: Vec<String> = Vec::new();

    // Determine which fields have persist strategies
    let local_fields: Vec<&str> = store
        .persist
        .iter()
        .filter(|(_, s)| **s == PersistStrategy::Local)
        .map(|(k, _)| k.as_str())
        .collect();
    let secure_fields: Vec<&str> = store
        .persist
        .iter()
        .filter(|(_, s)| **s == PersistStrategy::Secure)
        .map(|(k, _)| k.as_str())
        .collect();

    if !local_fields.is_empty() {
        restore_lines.push(format!(
            "        val localPrefs = context.getSharedPreferences(\"{name}\", Context.MODE_PRIVATE)"
        ));
        let mut sorted_local = local_fields.clone();
        sorted_local.sort();
        for field in sorted_local {
            if let Some(sf) = store.fields.get(field) {
                let default = sf
                    .default
                    .as_ref()
                    .map(|e| emit_expr(e))
                    .unwrap_or_else(|| default_for_type(&sf.type_));
                restore_lines.push(format!(
                    "        _{field}.value = localPrefs.getString(\"{field}\", {default}) ?: {default}"
                ));
            }
        }
    }

    if !secure_fields.is_empty() {
        restore_lines.push(
            "        val masterKey = MasterKey.Builder(context).setKeyScheme(MasterKey.KeyScheme.AES256_GCM).build()".to_string(),
        );
        restore_lines.push(format!(
            "        val securePrefs = EncryptedSharedPreferences.create(context, \"{name}_secure\", masterKey,\n            EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,\n            EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM)"
        ));
        let mut sorted_secure = secure_fields.clone();
        sorted_secure.sort();
        for field in sorted_secure {
            if let Some(sf) = store.fields.get(field) {
                let default = sf
                    .default
                    .as_ref()
                    .map(|e| emit_expr(e))
                    .unwrap_or_else(|| default_for_type(&sf.type_));
                restore_lines.push(format!(
                    "        _{field}.value = securePrefs.getString(\"{field}\", {default}) ?: {default}"
                ));
            }
        }
    }

    let restore_code = if restore_lines.is_empty() {
        "        // nothing persisted".to_string()
    } else {
        restore_lines.join("\n")
    };

    // ── Action functions ──────────────────────────────────────────────────────
    let mut action_fns: Vec<String> = store
        .actions
        .values()
        .map(|func| {
            let params_str: String = func
                .params
                .iter()
                .map(|(pname, ptype)| format!("{pname}: {}", frtype_to_kotlin(ptype)))
                .collect::<Vec<_>>()
                .join(", ");

            // Split body into state-update stmts vs. other stmts
            // Heuristic: Assign stmts that target a known field → state updates
            let known_fields: Vec<&str> = store.fields.keys().map(|k| k.as_str()).collect();

            let (state_update_stmts, other_stmts): (Vec<&Stmt>, Vec<&Stmt>) =
                func.body.iter().partition(|s| {
                    if let Stmt::Assign(var_name, _) = s {
                        known_fields.contains(&var_name.as_str())
                    } else {
                        false
                    }
                });

            // Emit other stmts at indent 2
            let other_code: String = other_stmts
                .iter()
                .map(|s| emit_stmt(s, 2))
                .collect();

            // Emit state-update stmts — translate `Assign(field, expr)` →
            // `_field.value = expr` + optional persist write
            let state_update_code: String = state_update_stmts
                .iter()
                .map(|s| {
                    if let Stmt::Assign(var_name, expr) = s {
                        let val = emit_expr(expr);
                        let mut lines = vec![format!("            _{var_name}.value = {val}")];

                        // Persist write
                        match store.persist.get(var_name.as_str()) {
                            Some(PersistStrategy::Local) => {
                                lines.push(format!(
                                    "            val prefs = context.getSharedPreferences(\"{name}\", Context.MODE_PRIVATE)"
                                ));
                                lines.push(format!(
                                    "            prefs.edit().putString(\"{var_name}\", {val}).apply()"
                                ));
                            }
                            Some(PersistStrategy::Secure) => {
                                lines.push("            val masterKey = MasterKey.Builder(context).setKeyScheme(MasterKey.KeyScheme.AES256_GCM).build()".to_string());
                                lines.push(format!(
                                    "            val securePrefs = EncryptedSharedPreferences.create(context, \"{name}_secure\", masterKey,\n                EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,\n                EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM)"
                                ));
                                lines.push(format!(
                                    "            securePrefs.edit().putString(\"{var_name}\", {val}).apply()"
                                ));
                            }
                            None => {}
                        }
                        lines.join("\n")
                    } else {
                        emit_stmt(s, 3)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");

            if func.is_async {
                format!(
                    "    fun {func_name}({params_str}) {{\n        viewModelScope.launch {{\n{other_code}            withContext(Dispatchers.Main) {{\n{state_update_code}\n            }}\n        }}\n    }}\n",
                    func_name = func.name,
                )
            } else {
                // Sync action: emit body directly
                let body_code: String = func.body.iter().map(|s| emit_stmt(s, 2)).collect();
                format!(
                    "    fun {func_name}({params_str}) {{\n{body_code}    }}\n",
                    func_name = func.name,
                )
            }
        })
        .collect();
    action_fns.sort();
    let action_functions = action_fns.join("\n");

    // Only include crypto imports when secure persistence is actually used
    let crypto_imports = if !secure_fields.is_empty() {
        "import androidx.security.crypto.EncryptedSharedPreferences\nimport androidx.security.crypto.MasterKey\n"
    } else {
        ""
    };

    OutputFile {
        path: format!("app/src/main/java/{pkg_path}/{class_name}.kt"),
        content: format!(
            r#"package {pkg}

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import android.content.Context
{crypto_imports}
class {class_name}(private val context: Context) : ViewModel() {{

    // ── State fields ───────────────────────────────────────────────────────
{state_flow_fields}
    // ── Init: restore persisted fields ────────────────────────────────────
    init {{
        restoreState()
    }}

    private fun restoreState() {{
{restore_code}
    }}

    // ── Actions ────────────────────────────────────────────────────────────
{action_functions}}}
"#,
        ),
    }
}

// ─── Platform feature helpers ─────────────────────────────────────────────────

/// Generate CameraHelper.kt
fn gen_camera_helper(pkg: &str, pkg_path: &str) -> OutputFile {
    OutputFile {
        path: format!("app/src/main/java/{pkg_path}/CameraHelper.kt"),
        content: format!(
            r#"package {pkg}

import android.content.Context
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity

object CameraHelper {{
    fun createCaptureLauncher(
        activity: AppCompatActivity,
        onCapture: (Boolean) -> Unit
    ): ActivityResultLauncher<Void?> {{
        return activity.registerForActivityResult(
            ActivityResultContracts.TakePicture(),
            onCapture
        )
    }}
}}
"#
        ),
    }
}

/// Generate LocationHelper.kt
fn gen_location_helper(pkg: &str, pkg_path: &str) -> OutputFile {
    OutputFile {
        path: format!("app/src/main/java/{pkg_path}/LocationHelper.kt"),
        content: format!(
            r#"package {pkg}

import android.annotation.SuppressLint
import android.content.Context
import com.google.android.gms.location.FusedLocationProviderClient
import com.google.android.gms.location.LocationServices

object LocationHelper {{
    @SuppressLint("MissingPermission")
    fun getCurrentLocation(
        context: Context,
        onLocation: (Double, Double) -> Unit
    ) {{
        val client: FusedLocationProviderClient =
            LocationServices.getFusedLocationProviderClient(context)
        client.lastLocation.addOnSuccessListener {{ location ->
            location?.let {{ onLocation(it.latitude, it.longitude) }}
        }}
    }}
}}
"#
        ),
    }
}

/// Generate NotificationHelper.kt
fn gen_notification_helper(pkg: &str, pkg_path: &str) -> OutputFile {
    OutputFile {
        path: format!("app/src/main/java/{pkg_path}/NotificationHelper.kt"),
        content: format!(
            r#"package {pkg}

import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import android.os.Build
import androidx.core.app.NotificationCompat

object NotificationHelper {{
    fun sendNotification(
        context: Context,
        channelId: String,
        title: String,
        message: String
    ) {{
        val manager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {{
            val channel = NotificationChannel(channelId, "Frame Notifications", NotificationManager.IMPORTANCE_DEFAULT)
            manager.createNotificationChannel(channel)
        }}
        val notification = NotificationCompat.Builder(context, channelId)
            .setContentTitle(title)
            .setContentText(message)
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .build()
        manager.notify(System.currentTimeMillis().toInt(), notification)
    }}
}}
"#
        ),
    }
}



/// Emit a single component node as a Compose call.
pub fn emit_composable(node: &ComponentNode, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    let inner_pad = "    ".repeat(indent + 1);

    // show_if wrapper
    let (open_if, close_if) = if let Some(cond) = &node.show_if {
        (
            format!("{pad}if ({}) {{\n", emit_expr(cond)),
            format!("{pad}}}\n"),
        )
    } else {
        (String::new(), String::new())
    };

    // Animation blocks
    let anim_code: String = node
        .animate
        .iter()
        .map(|a| emit_animation(a, indent))
        .collect::<Vec<_>>()
        .join("\n");

    let body = match node.kind.as_str() {
        "text" => emit_text_node(node, &pad),
        "button" => emit_button_node(node, &pad, &inner_pad),
        "image" => emit_image_node(node, &pad),
        "icon" => emit_icon_node(node, &pad),
        "row" => emit_container_node(node, "Row", &pad, &inner_pad, indent),
        "column" => emit_container_node(node, "Column", &pad, &inner_pad, indent),
        "container" => emit_box_node(node, "Box", &pad, &inner_pad, indent),
        "stack" => emit_stack_node(node, &pad, &inner_pad, indent),
        "list" => emit_list_node(node, &pad, &inner_pad, indent),
        "input" => emit_input_node(node, &pad),
        "dropdown" => emit_dropdown_node(node, &pad, &inner_pad, indent),
        "form" => emit_container_node(node, "Column", &pad, &inner_pad, indent),
        "app_bar" => emit_app_bar_node(node, &pad),
        "bottom_navigation_bar" => emit_bottom_nav_node(node, &pad, &inner_pad, indent),
        "scaffold" => emit_scaffold_node(node, &pad, &inner_pad, indent),
        "card" => emit_card_node(node, &pad, &inner_pad, indent),
        "divider" => format!("{pad}Divider()\n"),
        "spacer" => emit_spacer_node(node, &pad),
        "modal" => emit_modal_node(node, &pad, &inner_pad, indent),
        "scroll_view" => emit_scroll_view_node(node, &pad, &inner_pad, indent),
        "grid" => emit_grid_node(node, &pad, &inner_pad, indent),
        // ── Feedback ────────────────────────────────────────────────────────
        "toast"           => emit_toast_node(node, &pad),
        "tooltip"         => emit_tooltip_node(node, &pad, &inner_pad, indent),
        "badge"           => emit_badge_node(node, &pad, &inner_pad, indent),
        "progress_bar"    => emit_progress_bar_node(node, &pad),
        "progress_circle" => emit_progress_circle_node(node, &pad),
        // ── Navigation ──────────────────────────────────────────────────────
        "tab_bar"         => emit_tab_bar_node(node, &pad, &inner_pad, indent),
        "tab"             => emit_tab_node(node, &pad),
        "bottom_sheet"    => emit_bottom_sheet_node(node, &pad, &inner_pad, indent),
        // ── Inputs ──────────────────────────────────────────────────────────
        "switch"          => emit_switch_node(node, &pad),
        "checkbox"        => emit_checkbox_node(node, &pad),
        "radio"           => emit_radio_node(node, &pad),
        "slider"          => emit_slider_node(node, &pad),
        "stepper"         => emit_stepper_node(node, &pad, &inner_pad),
        "text_area"       => emit_text_area_node(node, &pad),
        "search_bar"      => emit_search_bar_node(node, &pad),
        "date_picker"     => emit_date_picker_node(node, &pad),
        "time_picker"     => emit_time_picker_node(node, &pad),
        "color_picker"    => emit_color_picker_node(node, &pad),
        "rating"          => emit_rating_node(node, &pad),
        "otp_input"       => emit_otp_input_node(node, &pad),
        // ── Display ─────────────────────────────────────────────────────────
        "avatar"          => emit_avatar_node(node, &pad),
        "chip"            => emit_chip_node(node, &pad),
        "tag"             => emit_tag_node(node, &pad),
        "banner"          => emit_banner_node(node, &pad, &inner_pad, indent),
        "table"           => emit_table_node(node, &pad, &inner_pad, indent),
        "accordion"       => emit_accordion_node(node, &pad, &inner_pad, indent),
        "timeline"        => emit_timeline_node(node, &pad, &inner_pad, indent),
        "skeleton"        => emit_skeleton_node(node, &pad),
        // ── Media ────────────────────────────────────────────────────────────
        "video_player"    => emit_video_player_node(node, &pad),
        "audio_player"    => emit_audio_player_node(node, &pad),
        "lottie"          => emit_lottie_node(node, &pad),
        "web_view"        => emit_web_view_node(node, &pad),
        "map_view"        => emit_map_view_node(node, &pad),
        "camera_view"     => emit_camera_view_node(node, &pad),
        "qr_scanner"      => emit_qr_scanner_node(node, &pad),
        // ── Gestures ────────────────────────────────────────────────────────
        "swipeable"       => emit_swipeable_node(node, &pad, &inner_pad, indent),
        "draggable"       => emit_draggable_node(node, &pad, &inner_pad, indent),
        "refresh"         => emit_refresh_node(node, &pad, &inner_pad, indent),
        "long_press"      => emit_long_press_node(node, &pad, &inner_pad, indent),
        _ => {
            // User-defined component
            let args = node
                .props
                .iter()
                .map(|(k, v)| format!("{k} = {}", emit_expr(v)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{pad}{}({args})\n", pascal_case(&node.kind))
        }
    };

    format!("{open_if}{anim_code}{body}{close_if}")
}

fn emit_modifier(styles: &Styles) -> String {
    let mut mods: Vec<String> = Vec::new();

    if let Some(w) = &styles.width {
        if w == "100%" {
            mods.push("fillMaxWidth()".to_string());
        } else if let Some(dp) = parse_dp(w) {
            mods.push(format!("width({dp}.dp)"));
        }
    }
    if let Some(h) = &styles.height {
        if h == "100%" {
            mods.push("fillMaxHeight()".to_string());
        } else if let Some(dp) = parse_dp(h) {
            mods.push(format!("height({dp}.dp)"));
        }
    }
    if let Some(bg) = &styles.background {
        let color = color_to_compose(bg);
        mods.push(format!("background({color})"));
    }
    if let Some(p) = &styles.padding {
        if let Some(dp) = parse_dp(p) {
            mods.push(format!("padding({dp}.dp)"));
        }
    }

    // overflow: hidden + border_radius → clip to rounded shape
    let has_border_radius = styles.border_radius.is_some();

    match styles.overflow {
        OverflowValue::Hidden => {
            if has_border_radius {
                if let Some(br) = &styles.border_radius {
                    if let Some(dp) = parse_dp(br) {
                        mods.push(format!("clip(RoundedCornerShape({dp}.dp))"));
                    }
                }
            } else {
                mods.push("clipToBounds()".to_string());
            }
        }
        _ => {
            // non-hidden: still apply border_radius clip if present (for visual rounding)
            if let Some(br) = &styles.border_radius {
                if let Some(dp) = parse_dp(br) {
                    mods.push(format!("clip(RoundedCornerShape({dp}.dp))"));
                }
            }
        }
    }

    // scroll modifiers
    let scroll_mod = match &styles.overflow {
        OverflowValue::ScrollY => Some("verticalScroll(rememberScrollState())".to_string()),
        OverflowValue::ScrollX => Some("horizontalScroll(rememberScrollState())".to_string()),
        OverflowValue::Scroll  => Some("verticalScroll(rememberScrollState())".to_string()),
        _ => None,
    };
    if let Some(sm) = scroll_mod {
        mods.push(sm);
    }
    // overflow_x / overflow_y take precedence
    if let Some(ov_x) = &styles.overflow_x {
        if *ov_x == OverflowValue::ScrollX || *ov_x == OverflowValue::Scroll {
            mods.push("horizontalScroll(rememberScrollState())".to_string());
        }
    }
    if let Some(ov_y) = &styles.overflow_y {
        if *ov_y == OverflowValue::ScrollY || *ov_y == OverflowValue::Scroll {
            mods.push("verticalScroll(rememberScrollState())".to_string());
        }
    }

    if mods.is_empty() {
        String::new()
    } else {
        format!("Modifier.{}", mods.join("."))
    }
}

fn emit_modifier_arg(styles: &Styles) -> String {
    let m = emit_modifier(styles);
    if m.is_empty() {
        String::new()
    } else {
        format!("modifier = {m}")
    }
}

// ─── Individual component emitters ───────────────────────────────────────────

fn emit_text_node(node: &ComponentNode, pad: &str) -> String {
    let content = node
        .props
        .get("content")
        .map(|e| emit_expr(e))
        .or_else(|| node.props.get("text").map(|e| emit_expr(e)))
        .unwrap_or_else(|| "\"\"".to_string());
    let mod_arg = emit_modifier_arg(&node.styles);

    // Text overflow: Ellipsis / Fade / Clip
    let text_overflow_arg = match node.styles.text_overflow {
        TextOverflowValue::Ellipsis => Some("overflow = TextOverflow.Ellipsis".to_string()),
        TextOverflowValue::Fade     => Some("overflow = TextOverflow.Clip".to_string()),
        TextOverflowValue::Clip     => None, // default — skip
    };

    // max_lines: prefer line_clamp alias
    let max_lines_val = node.styles.line_clamp.or(node.styles.max_lines);
    let max_lines_arg = max_lines_val.map(|n| format!("maxLines = {n}"));

    let mut args_parts = vec![format!("text = {content}")];
    if let Some(ref ml) = max_lines_arg   { args_parts.push(ml.clone()); }
    if let Some(ref to) = text_overflow_arg { args_parts.push(to.clone()); }
    if !mod_arg.is_empty()               { args_parts.push(mod_arg); }

    format!("{pad}Text({})\n", args_parts.join(", "))
}

fn emit_button_node(node: &ComponentNode, pad: &str, inner_pad: &str) -> String {
    let on_click = node
        .events
        .on_click
        .as_ref()
        .map(|e| emit_expr(e))
        .unwrap_or_else(|| "{}".to_string());
    let content = node
        .props
        .get("content")
        .map(|e| emit_expr(e))
        .unwrap_or_else(|| "\"\"".to_string());
    let mod_arg = emit_modifier_arg(&node.styles);
    let args = if mod_arg.is_empty() {
        format!("onClick = {{ {on_click} }}")
    } else {
        format!("onClick = {{ {on_click} }}, {mod_arg}")
    };
    format!("{pad}Button({args}) {{\n{inner_pad}Text(text = {content})\n{pad}}}\n")
}

fn emit_image_node(node: &ComponentNode, pad: &str) -> String {
    let src = node
        .props
        .get("src")
        .map(|e| emit_expr(e))
        .unwrap_or_else(|| "\"\"".to_string());
    let mod_arg = emit_modifier_arg(&node.styles);

    // fit: → ContentScale (skip Contain which is the AsyncImage default)
    let content_scale_arg = match node.styles.image_fit {
        ImageFitValue::Cover     => Some("contentScale = ContentScale.Crop".to_string()),
        ImageFitValue::Fill      => Some("contentScale = ContentScale.FillBounds".to_string()),
        ImageFitValue::None_     => Some("contentScale = ContentScale.None".to_string()),
        ImageFitValue::ScaleDown => Some("contentScale = ContentScale.Inside".to_string()),
        ImageFitValue::Contain   => None, // default — skip
    };

    let mut args_parts = vec![
        format!("model = {src}"),
        "contentDescription = null".to_string(),
    ];
    if let Some(cs) = content_scale_arg { args_parts.push(cs); }
    if !mod_arg.is_empty()              { args_parts.push(mod_arg); }

    format!("{pad}AsyncImage({})\n", args_parts.join(", "))
}

fn emit_icon_node(node: &ComponentNode, pad: &str) -> String {
    let icon = node
        .props
        .get("icon")
        .map(|e| {
            let raw = emit_expr(e);
            // strip quotes
            raw.trim_matches('"').to_string()
        })
        .unwrap_or_else(|| "Star".to_string());
    let pascal = pascal_case(&icon);
    let mod_arg = emit_modifier_arg(&node.styles);
    let args = if mod_arg.is_empty() {
        format!("imageVector = Icons.Default.{pascal}, contentDescription = null")
    } else {
        format!("imageVector = Icons.Default.{pascal}, contentDescription = null, {mod_arg}")
    };
    format!("{pad}Icon({args})\n")
}

fn emit_container_node(
    node: &ComponentNode,
    compose_name: &str,
    pad: &str,
    _inner_pad: &str,
    indent: usize,
) -> String {
    let mod_arg = emit_modifier_arg(&node.styles);
    let args = if mod_arg.is_empty() {
        String::new()
    } else {
        mod_arg
    };
    let children: String = node
        .children
        .iter()
        .map(|c| emit_composable(c, indent + 1))
        .collect();
    format!("{pad}{compose_name}({args}) {{\n{children}{pad}}}\n")
}

fn emit_box_node(
    node: &ComponentNode,
    compose_name: &str,
    pad: &str,
    _inner_pad: &str,
    indent: usize,
) -> String {
    let mod_arg = emit_modifier_arg(&node.styles);
    let args = if mod_arg.is_empty() {
        String::new()
    } else {
        mod_arg
    };
    let children: String = node
        .children
        .iter()
        .map(|c| emit_composable(c, indent + 1))
        .collect();
    format!("{pad}{compose_name}({args}) {{\n{children}{pad}}}\n")
}

fn emit_stack_node(node: &ComponentNode, pad: &str, _inner_pad: &str, indent: usize) -> String {
    let alignment = stack_alignment_to_compose(&node.alignment);
    let mod_str = emit_modifier(&node.styles);
    let mod_arg = if mod_str.is_empty() {
        format!("contentAlignment = {alignment}")
    } else {
        format!("modifier = {mod_str}, contentAlignment = {alignment}")
    };

    let inner_pad = "    ".repeat(indent + 1);
    let _inner_inner = "    ".repeat(indent + 2);

    let children: String = node
        .children
        .iter()
        .map(|c| {
            if let Some(pos) = &c.positioned {
                // Positioned child: wrap in Box with offset
                let offset_x = pos.left.as_deref().and_then(parse_dp).unwrap_or(0);
                let offset_y = pos.top.as_deref().and_then(parse_dp).unwrap_or(0);
                let child_alignment = stack_alignment_to_compose(&c.alignment);
                let inner_child = emit_composable(c, indent + 2);
                format!(
                    "{inner_pad}Box(modifier = Modifier.align({child_alignment}).offset(x = {offset_x}.dp, y = {offset_y}.dp)) {{\n{inner_child}{inner_pad}}}\n"
                )
            } else {
                emit_composable(c, indent + 1)
            }
        })
        .collect();

    format!("{pad}Box({mod_arg}) {{\n{children}{pad}}}\n")
}

fn emit_list_node(node: &ComponentNode, pad: &str, inner_pad: &str, indent: usize) -> String {
    let data = node
        .data
        .as_ref()
        .map(|e| emit_expr(e))
        .unwrap_or_else(|| "emptyList<Any>()".to_string());

    let item_body: String = if let Some(build) = &node.build {
        // Use build function body children
        build
            .body
            .iter()
            .map(|s| emit_stmt(s, indent + 2))
            .collect()
    } else {
        node.children
            .iter()
            .map(|c| emit_composable(c, indent + 2))
            .collect()
    };

    let item_param = node
        .build
        .as_ref()
        .and_then(|b| b.params.first())
        .map(|(name, _)| name.clone())
        .unwrap_or_else(|| "item".to_string());

    format!(
        "{pad}LazyColumn {{\n{inner_pad}items({data}) {{ {item_param} ->\n{item_body}{inner_pad}}}\n{pad}}}\n"
    )
}

fn emit_input_node(node: &ComponentNode, pad: &str) -> String {
    let value = node
        .props
        .get("value")
        .map(|e| emit_expr(e))
        .unwrap_or_else(|| "\"\"".to_string());
    let on_change = node
        .events
        .on_change
        .as_ref()
        .map(|e| emit_expr(e))
        .unwrap_or_else(|| "{}".to_string());
    let mod_arg = emit_modifier_arg(&node.styles);
    let extra = if mod_arg.is_empty() {
        String::new()
    } else {
        format!(", {mod_arg}")
    };
    format!("{pad}TextField(value = {value}, onValueChange = {{ {on_change} }}{extra})\n")
}

fn emit_dropdown_node(node: &ComponentNode, pad: &str, _inner_pad: &str, indent: usize) -> String {
    let items: String = node
        .children
        .iter()
        .map(|c| emit_composable(c, indent + 1))
        .collect();
    format!("{pad}DropdownMenu(expanded = true, onDismissRequest = {{}}) {{\n{items}{pad}}}\n")
}

fn emit_app_bar_node(node: &ComponentNode, pad: &str) -> String {
    let title = node
        .props
        .get("title")
        .map(|e| emit_expr(e))
        .unwrap_or_else(|| "\"\"".to_string());
    format!("{pad}TopAppBar(title = {{ Text(text = {title}) }})\n")
}

fn emit_bottom_nav_node(node: &ComponentNode, pad: &str, _inner_pad: &str, indent: usize) -> String {
    let items: String = node
        .children
        .iter()
        .map(|c| emit_composable(c, indent + 1))
        .collect();
    format!("{pad}NavigationBar {{\n{items}{pad}}}\n")
}

fn emit_scaffold_node(node: &ComponentNode, pad: &str, _inner_pad: &str, indent: usize) -> String {
    // Find top_bar and bottom_bar children by kind
    let top_bar = node
        .children
        .iter()
        .find(|c| c.kind == "app_bar")
        .map(|c| emit_composable(c, 0).trim_end().to_string())
        .unwrap_or_default();
    let bottom_bar = node
        .children
        .iter()
        .find(|c| c.kind == "bottom_navigation_bar")
        .map(|c| emit_composable(c, 0).trim_end().to_string())
        .unwrap_or_default();
    let content: String = node
        .children
        .iter()
        .filter(|c| c.kind != "app_bar" && c.kind != "bottom_navigation_bar")
        .map(|c| emit_composable(c, indent + 2))
        .collect();

    let top_bar_arg = if top_bar.is_empty() {
        String::new()
    } else {
        format!("topBar = {{ {top_bar} }}, ")
    };
    let bottom_bar_arg = if bottom_bar.is_empty() {
        String::new()
    } else {
        format!("bottomBar = {{ {bottom_bar} }}")
    };

    let inner_pad = "    ".repeat(indent + 1);
    format!(
        "{pad}Scaffold({top_bar_arg}{bottom_bar_arg}) {{ innerPadding ->\n{inner_pad}Column(modifier = Modifier.padding(innerPadding)) {{\n{content}{inner_pad}}}\n{pad}}}\n"
    )
}

fn emit_card_node(node: &ComponentNode, pad: &str, _inner_pad: &str, indent: usize) -> String {
    let mod_arg = emit_modifier_arg(&node.styles);
    let args = if mod_arg.is_empty() { String::new() } else { mod_arg };
    let children: String = node
        .children
        .iter()
        .map(|c| emit_composable(c, indent + 1))
        .collect();
    format!("{pad}Card({args}) {{\n{children}{pad}}}\n")
}

fn emit_spacer_node(node: &ComponentNode, pad: &str) -> String {
    let size = node
        .styles
        .width
        .as_ref()
        .or(node.styles.height.as_ref())
        .and_then(|s| parse_dp(s))
        .map(|dp| format!("size({dp}.dp)"))
        .unwrap_or_else(|| "height(8.dp)".to_string());
    format!("{pad}Spacer(modifier = Modifier.{size})\n")
}

fn emit_modal_node(node: &ComponentNode, pad: &str, _inner_pad: &str, indent: usize) -> String {
    let content: String = node
        .children
        .iter()
        .map(|c| emit_composable(c, indent + 1))
        .collect();
    format!(
        "{pad}AlertDialog(\n{pad}    onDismissRequest = {{}},\n{pad}    confirmButton = {{}},\n{pad}    text = {{\n{content}{pad}    }}\n{pad})\n"
    )
}

fn emit_scroll_view_node(node: &ComponentNode, pad: &str, _inner_pad: &str, indent: usize) -> String {
    let children: String = node
        .children
        .iter()
        .map(|c| emit_composable(c, indent + 1))
        .collect();
    format!(
        "{pad}Column(modifier = Modifier.verticalScroll(rememberScrollState())) {{\n{children}{pad}}}\n"
    )
}

fn emit_grid_node(node: &ComponentNode, pad: &str, inner_pad: &str, indent: usize) -> String {
    let columns = node
        .props
        .get("columns")
        .map(|e| emit_expr(e))
        .unwrap_or_else(|| "2".to_string());
    let data = node
        .data
        .as_ref()
        .map(|e| emit_expr(e))
        .unwrap_or_else(|| "emptyList<Any>()".to_string());
    let build_body: String = if let Some(build) = &node.build {
        build
            .body
            .iter()
            .map(|s| emit_stmt(s, indent + 2))
            .collect()
    } else {
        node.children
            .iter()
            .map(|c| emit_composable(c, indent + 2))
            .collect()
    };
    let item_param = node
        .build
        .as_ref()
        .and_then(|b| b.params.first())
        .map(|(n, _)| n.clone())
        .unwrap_or_else(|| "item".to_string());

    format!(
        "{pad}LazyVerticalGrid(columns = GridCells.Fixed({columns})) {{\n{inner_pad}items({data}) {{ {item_param} ->\n{build_body}{inner_pad}}}\n{pad}}}\n"
    )
}

// ─── New built-in component emitters ─────────────────────────────────────────

// ── Feedback ──────────────────────────────────────────────────────────────────

fn emit_toast_node(node: &ComponentNode, pad: &str) -> String {
    let message = node.props.get("message").or_else(|| node.props.get("content"))
        .map(|e| emit_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let duration = node.props.get("duration")
        .map(|e| emit_expr(e)).unwrap_or_else(|| "Toast.LENGTH_SHORT".to_string());
    // Compose doesn't have native Toast in LazyComposables; emit a LaunchedEffect Snackbar trigger
    format!("{pad}// toast: {message}\n{pad}LaunchedEffect(Unit) {{ /* show Snackbar: {message} duration={duration} */ }}\n")
}

fn emit_tooltip_node(node: &ComponentNode, pad: &str, inner_pad: &str, indent: usize) -> String {
    let text = node.props.get("text").or_else(|| node.props.get("content"))
        .map(|e| emit_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let children: String = node.children.iter().map(|c| emit_composable(c, indent + 1)).collect();
    format!("{pad}TooltipBox(positionProvider = TooltipDefaults.rememberPlainTooltipPositionProvider(), tooltip = {{ {inner_pad}Text({text}) }}, state = rememberTooltipState()) {{\n{children}{pad}}}\n")
}

fn emit_badge_node(node: &ComponentNode, pad: &str, inner_pad: &str, indent: usize) -> String {
    let count = node.props.get("count").or_else(|| node.props.get("value"))
        .map(|e| emit_expr(e)).unwrap_or_else(|| "0".to_string());
    let children: String = node.children.iter().map(|c| emit_composable(c, indent + 1)).collect();
    format!("{pad}BadgedBox(badge = {{ {inner_pad}Badge {{ Text({count}) }} }}) {{\n{children}{pad}}}\n")
}

fn emit_progress_bar_node(node: &ComponentNode, pad: &str) -> String {
    let value = node.props.get("value").map(|e| emit_expr(e));
    let mod_arg = emit_modifier_arg(&node.styles);
    match value {
        Some(v) => {
            let args = if mod_arg.is_empty() { format!("progress = {{ {v}.toFloat() }}") }
                       else { format!("progress = {{ {v}.toFloat() }}, {mod_arg}") };
            format!("{pad}LinearProgressIndicator({args})\n")
        }
        None => {
            let args = if mod_arg.is_empty() { String::new() } else { mod_arg };
            format!("{pad}LinearProgressIndicator({args})\n")
        }
    }
}

fn emit_progress_circle_node(node: &ComponentNode, pad: &str) -> String {
    let value = node.props.get("value").map(|e| emit_expr(e));
    let mod_arg = emit_modifier_arg(&node.styles);
    match value {
        Some(v) => {
            let args = if mod_arg.is_empty() { format!("progress = {{ {v}.toFloat() }}") }
                       else { format!("progress = {{ {v}.toFloat() }}, {mod_arg}") };
            format!("{pad}CircularProgressIndicator({args})\n")
        }
        None => {
            let args = if mod_arg.is_empty() { String::new() } else { mod_arg };
            format!("{pad}CircularProgressIndicator({args})\n")
        }
    }
}

// ── Navigation ────────────────────────────────────────────────────────────────

fn emit_tab_bar_node(node: &ComponentNode, pad: &str, _inner_pad: &str, indent: usize) -> String {
    let selected = node.props.get("selected").or_else(|| node.props.get("current"))
        .map(|e| emit_expr(e)).unwrap_or_else(|| "0".to_string());
    let tabs: String = node.children.iter().map(|c| emit_composable(c, indent + 1)).collect();
    format!("{pad}TabRow(selectedTabIndex = {selected}) {{\n{tabs}{pad}}}\n")
}

fn emit_tab_node(node: &ComponentNode, pad: &str) -> String {
    let text = node.props.get("content").or_else(|| node.props.get("title"))
        .map(|e| emit_expr(e)).unwrap_or_else(|| "\"Tab\"".to_string());
    let selected = node.props.get("selected").map(|e| emit_expr(e)).unwrap_or_else(|| "false".to_string());
    let on_click = node.events.on_click.as_ref().map(|e| emit_expr(e)).unwrap_or_else(|| "{}".to_string());
    let icon_arg = node.props.get("icon").map(|e| {
        let raw = emit_expr(e);
        let name = raw.trim_matches('"');
        format!(", icon = {{ Icon(Icons.Default.{}, contentDescription = null) }}", pascal_case(name))
    }).unwrap_or_default();
    format!("{pad}Tab(selected = {selected}, onClick = {{ {on_click} }}, text = {{ Text({text}) }}{icon_arg})\n")
}

fn emit_bottom_sheet_node(node: &ComponentNode, pad: &str, _inner_pad: &str, indent: usize) -> String {
    let on_dismiss = node.events.on_unmount.as_ref().map(|e| emit_expr(e)).unwrap_or_else(|| "{}".to_string());
    let children: String = node.children.iter().map(|c| emit_composable(c, indent + 1)).collect();
    format!("{pad}ModalBottomSheet(onDismissRequest = {{ {on_dismiss} }}) {{\n{children}{pad}}}\n")
}

// ── Inputs ────────────────────────────────────────────────────────────────────

fn emit_switch_node(node: &ComponentNode, pad: &str) -> String {
    let checked = node.props.get("value").or_else(|| node.props.get("checked"))
        .map(|e| emit_expr(e)).unwrap_or_else(|| "false".to_string());
    let on_change = node.events.on_change.as_ref().map(|e| emit_expr(e)).unwrap_or_else(|| "{}".to_string());
    format!("{pad}Switch(checked = {checked}, onCheckedChange = {{ {on_change} }})\n")
}

fn emit_checkbox_node(node: &ComponentNode, pad: &str) -> String {
    let checked = node.props.get("value").or_else(|| node.props.get("checked"))
        .map(|e| emit_expr(e)).unwrap_or_else(|| "false".to_string());
    let on_change = node.events.on_change.as_ref().map(|e| emit_expr(e)).unwrap_or_else(|| "{}".to_string());
    format!("{pad}Checkbox(checked = {checked}, onCheckedChange = {{ {on_change} }})\n")
}

fn emit_radio_node(node: &ComponentNode, pad: &str) -> String {
    let selected = node.props.get("selected").map(|e| emit_expr(e)).unwrap_or_else(|| "false".to_string());
    let on_click = node.events.on_click.as_ref().map(|e| emit_expr(e)).unwrap_or_else(|| "{}".to_string());
    format!("{pad}RadioButton(selected = {selected}, onClick = {{ {on_click} }})\n")
}

fn emit_slider_node(node: &ComponentNode, pad: &str) -> String {
    let value = node.props.get("value").map(|e| emit_expr(e)).unwrap_or_else(|| "0f".to_string());
    let on_change = node.events.on_change.as_ref().map(|e| emit_expr(e)).unwrap_or_else(|| "{}".to_string());
    let min = node.props.get("min").map(|e| emit_expr(e)).unwrap_or_else(|| "0f".to_string());
    let max = node.props.get("max").map(|e| emit_expr(e)).unwrap_or_else(|| "100f".to_string());
    format!("{pad}Slider(value = {value}.toFloat(), onValueChange = {{ {on_change} }}, valueRange = {min}.toFloat()..{max}.toFloat())\n")
}

fn emit_stepper_node(node: &ComponentNode, pad: &str, inner_pad: &str) -> String {
    let value = node.props.get("value").map(|e| emit_expr(e)).unwrap_or_else(|| "0".to_string());
    let on_increment = node.props.get("on_increment").or_else(|| node.props.get("on_increase"))
        .map(|e| emit_expr(e)).unwrap_or_else(|| "{}".to_string());
    let on_decrement = node.props.get("on_decrement").or_else(|| node.props.get("on_decrease"))
        .map(|e| emit_expr(e)).unwrap_or_else(|| "{}".to_string());
    format!("{pad}Row(verticalAlignment = Alignment.CenterVertically) {{\n\
             {inner_pad}IconButton(onClick = {{ {on_decrement} }}) {{ Icon(Icons.Default.Remove, null) }}\n\
             {inner_pad}Text(text = {value}.toString())\n\
             {inner_pad}IconButton(onClick = {{ {on_increment} }}) {{ Icon(Icons.Default.Add, null) }}\n\
             {pad}}}\n")
}

fn emit_text_area_node(node: &ComponentNode, pad: &str) -> String {
    let value = node.props.get("value").map(|e| emit_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let on_change = node.events.on_change.as_ref().map(|e| emit_expr(e)).unwrap_or_else(|| "{}".to_string());
    let lines = node.props.get("lines").map(|e| emit_expr(e)).unwrap_or_else(|| "4".to_string());
    let mod_arg = emit_modifier_arg(&node.styles);
    let extra = if mod_arg.is_empty() { String::new() } else { format!(", {mod_arg}") };
    format!("{pad}TextField(value = {value}, onValueChange = {{ {on_change} }}, minLines = {lines}, maxLines = {lines}{extra})\n")
}

fn emit_search_bar_node(node: &ComponentNode, pad: &str) -> String {
    let value = node.props.get("value").map(|e| emit_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let on_change = node.events.on_change.as_ref().map(|e| emit_expr(e)).unwrap_or_else(|| "{}".to_string());
    let placeholder = node.props.get("placeholder").map(|e| emit_expr(e)).unwrap_or_else(|| "\"Search\"".to_string());
    format!("{pad}TextField(value = {value}, onValueChange = {{ {on_change} }}, placeholder = {{ Text({placeholder}) }}, leadingIcon = {{ Icon(Icons.Default.Search, null) }}, singleLine = true)\n")
}

fn emit_date_picker_node(_node: &ComponentNode, pad: &str) -> String {
    format!("{pad}DatePicker(state = rememberDatePickerState())\n")
}

fn emit_time_picker_node(_node: &ComponentNode, pad: &str) -> String {
    format!("{pad}TimeInput(state = rememberTimePickerState())\n")
}

fn emit_color_picker_node(node: &ComponentNode, pad: &str) -> String {
    let on_change = node.events.on_change.as_ref().map(|e| emit_expr(e)).unwrap_or_else(|| "{}".to_string());
    format!("{pad}// color_picker: no standard Compose component — use a custom colour picker library\n{pad}// on_change: {on_change}\n")
}

fn emit_rating_node(node: &ComponentNode, pad: &str) -> String {
    let value = node.props.get("value").map(|e| emit_expr(e)).unwrap_or_else(|| "0".to_string());
    let max = node.props.get("max").map(|e| emit_expr(e)).unwrap_or_else(|| "5".to_string());
    let on_change = node.events.on_change.as_ref().map(|e| emit_expr(e)).unwrap_or_else(|| "{}".to_string());
    format!("{pad}Row {{\n{pad}    repeat({max}) {{ index ->\n\
             {pad}        IconButton(onClick = {{ {on_change} }}) {{\n\
             {pad}            Icon(if (index < {value}) Icons.Default.Star else Icons.Default.StarBorder, null)\n\
             {pad}        }}\n{pad}    }}\n{pad}}}\n")
}

fn emit_otp_input_node(node: &ComponentNode, pad: &str) -> String {
    let length = node.props.get("length").map(|e| emit_expr(e)).unwrap_or_else(|| "6".to_string());
    let on_complete = node.props.get("on_complete").map(|e| emit_expr(e)).unwrap_or_else(|| "{}".to_string());
    format!("{pad}// otp_input: length={length} on_complete={on_complete}\n\
             {pad}Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {{\n\
             {pad}    repeat({length}) {{\n\
             {pad}        OutlinedTextField(value = \"\", onValueChange = {{}}, modifier = Modifier.width(48.dp), singleLine = true)\n\
             {pad}    }}\n{pad}}}\n")
}

// ── Display ───────────────────────────────────────────────────────────────────

fn emit_avatar_node(node: &ComponentNode, pad: &str) -> String {
    let src = node.props.get("src").map(|e| emit_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let size_dp = node.styles.width.as_ref().and_then(|w| parse_dp(w)).unwrap_or(48);
    let mod_arg = format!("Modifier.size({size_dp}.dp).clip(CircleShape)");
    format!("{pad}AsyncImage(model = {src}, contentDescription = null, modifier = {mod_arg}, contentScale = ContentScale.Crop)\n")
}

fn emit_chip_node(node: &ComponentNode, pad: &str) -> String {
    let label = node.props.get("content").or_else(|| node.props.get("label"))
        .map(|e| emit_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let on_click = node.events.on_click.as_ref().map(|e| emit_expr(e));
    match on_click {
        Some(handler) => format!("{pad}AssistChip(onClick = {{ {handler} }}, label = {{ Text({label}) }})\n"),
        None           => format!("{pad}SuggestionChip(onClick = {{}}, label = {{ Text({label}) }})\n"),
    }
}

fn emit_tag_node(node: &ComponentNode, pad: &str) -> String {
    let label = node.props.get("content").or_else(|| node.props.get("label"))
        .map(|e| emit_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    format!("{pad}SuggestionChip(onClick = {{}}, label = {{ Text({label}) }})\n")
}

fn emit_banner_node(node: &ComponentNode, pad: &str, _inner_pad: &str, indent: usize) -> String {
    let children: String = node.children.iter().map(|c| emit_composable(c, indent + 1)).collect();
    let mod_arg = emit_modifier_arg(&node.styles);
    let args = if mod_arg.is_empty() { String::new() } else { mod_arg };
    format!("{pad}Card({args}) {{\n{children}{pad}}}\n")
}

fn emit_table_node(node: &ComponentNode, pad: &str, inner_pad: &str, indent: usize) -> String {
    let data = node.data.as_ref().map(|e| emit_expr(e)).unwrap_or_else(|| "emptyList<Any>()".to_string());
    let header: String = node.children.iter().filter(|c| c.kind == "item" || c.kind == "table_header")
        .map(|c| emit_composable(c, indent + 1)).collect();
    let rows: String = node.children.iter().filter(|c| c.kind != "item" && c.kind != "table_header")
        .map(|c| emit_composable(c, indent + 1)).collect();
    format!("{pad}LazyColumn {{\n\
             {inner_pad}item {{ Row {{ {header} }} }}\n\
             {inner_pad}items({data}) {{ row -> {rows} }}\n\
             {pad}}}\n")
}

fn emit_accordion_node(node: &ComponentNode, pad: &str, _inner_pad: &str, indent: usize) -> String {
    let title = node.props.get("title").or_else(|| node.props.get("content"))
        .map(|e| emit_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let children: String = node.children.iter().map(|c| emit_composable(c, indent + 2)).collect();
    let inner_pad = "    ".repeat(indent + 1);
    format!("{pad}var expanded by remember {{ mutableStateOf(false) }}\n\
             {pad}Column {{\n\
             {inner_pad}Row(modifier = Modifier.clickable {{ expanded = !expanded }}, verticalAlignment = Alignment.CenterVertically) {{\n\
             {inner_pad}    Text({title}, modifier = Modifier.weight(1f))\n\
             {inner_pad}    Icon(if (expanded) Icons.Default.ExpandLess else Icons.Default.ExpandMore, null)\n\
             {inner_pad}}}\n\
             {inner_pad}AnimatedVisibility(visible = expanded) {{\n\
             {children}{inner_pad}}}\n\
             {pad}}}\n")
}

fn emit_timeline_node(node: &ComponentNode, pad: &str, _inner_pad: &str, indent: usize) -> String {
    let items: String = node.children.iter().map(|c| emit_composable(c, indent + 1)).collect();
    format!("{pad}LazyColumn {{\n{items}{pad}}}\n")
}

fn emit_skeleton_node(node: &ComponentNode, pad: &str) -> String {
    let mod_str = emit_modifier(&node.styles);
    let mod_part = if mod_str.is_empty() { "Modifier.fillMaxWidth().height(16.dp)".to_string() } else { mod_str };
    // Uses shimmer animation via background + animate
    format!("{pad}Box(modifier = {mod_part}.background(Color.LightGray.copy(alpha = 0.5f), RoundedCornerShape(4.dp)))\n")
}

// ── Media ─────────────────────────────────────────────────────────────────────

fn emit_video_player_node(node: &ComponentNode, pad: &str) -> String {
    let src = node.props.get("src").map(|e| emit_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let mod_arg = emit_modifier_arg(&node.styles);
    let mod_part = if mod_arg.is_empty() { "modifier = Modifier.fillMaxWidth()".to_string() }
                   else { format!("{mod_arg}") };
    format!("{pad}AndroidView(\n{pad}    factory = {{ ctx ->\n\
             {pad}        androidx.media3.ui.PlayerView(ctx).also {{ pv ->\n\
             {pad}            val player = androidx.media3.exoplayer.ExoPlayer.Builder(ctx).build()\n\
             {pad}            player.setMediaItem(androidx.media3.common.MediaItem.fromUri({src}))\n\
             {pad}            player.prepare()\n\
             {pad}            pv.player = player\n\
             {pad}        }}\n\
             {pad}    }},\n\
             {pad}    {mod_part}\n{pad})\n")
}

fn emit_audio_player_node(node: &ComponentNode, pad: &str) -> String {
    let src = node.props.get("src").map(|e| emit_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    format!("{pad}// audio_player: src={src}\n\
             {pad}LaunchedEffect(Unit) {{\n\
             {pad}    val mp = android.media.MediaPlayer()\n\
             {pad}    mp.setDataSource({src})\n\
             {pad}    mp.prepare()\n\
             {pad}    mp.start()\n\
             {pad}}}\n")
}

fn emit_lottie_node(node: &ComponentNode, pad: &str) -> String {
    let src = node.props.get("src").map(|e| emit_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let mod_arg = emit_modifier_arg(&node.styles);
    let mod_part = if mod_arg.is_empty() { "Modifier.size(200.dp)".to_string() } else { emit_modifier(&node.styles) };
    format!("{pad}// lottie: requires com.airbnb.android:lottie-compose\n\
             {pad}val composition by rememberLottieComposition(LottieCompositionSpec.Url({src}))\n\
             {pad}LottieAnimation(composition, {{ it }}, modifier = {mod_part})\n")
}

fn emit_web_view_node(node: &ComponentNode, pad: &str) -> String {
    let url = node.props.get("url").or_else(|| node.props.get("src"))
        .map(|e| emit_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    let mod_arg = emit_modifier_arg(&node.styles);
    let mod_part = if mod_arg.is_empty() { "modifier = Modifier.fillMaxSize()".to_string() } else { mod_arg };
    format!("{pad}AndroidView(\n{pad}    factory = {{ ctx ->\n\
             {pad}        android.webkit.WebView(ctx).also {{ it.loadUrl({url}) }}\n\
             {pad}    }},\n\
             {pad}    {mod_part}\n{pad})\n")
}

fn emit_map_view_node(node: &ComponentNode, pad: &str) -> String {
    let lat = node.props.get("lat").map(|e| emit_expr(e)).unwrap_or_else(|| "0.0".to_string());
    let lng = node.props.get("lng").map(|e| emit_expr(e)).unwrap_or_else(|| "0.0".to_string());
    let mod_arg = emit_modifier_arg(&node.styles);
    let mod_part = if mod_arg.is_empty() { "modifier = Modifier.fillMaxSize()".to_string() } else { mod_arg };
    format!("{pad}// map_view: requires com.google.maps.android:maps-compose\n\
             {pad}GoogleMap({mod_part}, cameraPositionState = rememberCameraPositionState {{\n\
             {pad}    position = CameraPosition.fromLatLngZoom(LatLng({lat}, {lng}), 12f)\n\
             {pad}}})\n")
}

fn emit_camera_view_node(node: &ComponentNode, pad: &str) -> String {
    let mod_arg = emit_modifier_arg(&node.styles);
    let mod_part = if mod_arg.is_empty() { "Modifier.fillMaxSize()".to_string() } else { emit_modifier(&node.styles) };
    format!("{pad}// camera_view: requires androidx.camera:camera-compose\n\
             {pad}CameraPreview(modifier = {mod_part})\n")
}

fn emit_qr_scanner_node(node: &ComponentNode, pad: &str) -> String {
    let on_scan = node.props.get("on_scan").map(|e| emit_expr(e)).unwrap_or_else(|| "{}".to_string());
    format!("{pad}// qr_scanner: requires CameraX + ML Kit barcode scanning\n\
             {pad}AndroidView(factory = {{ ctx -> createQrScannerView(ctx) {{ result -> {on_scan} }} }}, modifier = Modifier.fillMaxSize())\n")
}

// ── Gestures ──────────────────────────────────────────────────────────────────

fn emit_swipeable_node(node: &ComponentNode, pad: &str, _inner_pad: &str, indent: usize) -> String {
    let on_swipe = node.events.on_change.as_ref().map(|e| emit_expr(e)).unwrap_or_else(|| "{}".to_string());
    let children: String = node.children.iter().map(|c| emit_composable(c, indent + 1)).collect();
    format!("{pad}SwipeToDismissBox(state = rememberSwipeToDismissBoxState(confirmValueChange = {{ {on_swipe}; true }}), backgroundContent = {{}}) {{\n{children}{pad}}}\n")
}

fn emit_draggable_node(node: &ComponentNode, pad: &str, _inner_pad: &str, indent: usize) -> String {
    let on_drag = node.events.on_change.as_ref().map(|e| emit_expr(e)).unwrap_or_else(|| "{}".to_string());
    let children: String = node.children.iter().map(|c| emit_composable(c, indent + 1)).collect();
    format!("{pad}Box(modifier = Modifier.draggable(rememberDraggableState {{ delta -> {on_drag} }}, orientation = Orientation.Vertical)) {{\n{children}{pad}}}\n")
}

fn emit_refresh_node(node: &ComponentNode, pad: &str, _inner_pad: &str, indent: usize) -> String {
    let on_refresh = node.events.on_change.as_ref().map(|e| emit_expr(e)).unwrap_or_else(|| "{}".to_string());
    let is_refreshing = node.props.get("refreshing").map(|e| emit_expr(e)).unwrap_or_else(|| "false".to_string());
    let children: String = node.children.iter().map(|c| emit_composable(c, indent + 1)).collect();
    format!("{pad}PullToRefreshBox(isRefreshing = {is_refreshing}, onRefresh = {{ {on_refresh} }}) {{\n{children}{pad}}}\n")
}

fn emit_long_press_node(node: &ComponentNode, pad: &str, _inner_pad: &str, indent: usize) -> String {
    let on_long_press = node.events.on_change.as_ref()
        .or(node.events.on_click.as_ref())
        .map(|e| emit_expr(e)).unwrap_or_else(|| "{}".to_string());
    let children: String = node.children.iter().map(|c| emit_composable(c, indent + 1)).collect();
    format!("{pad}Box(modifier = Modifier.combinedClickable(onLongClick = {{ {on_long_press} }}, onClick = {{}})) {{\n{children}{pad}}}\n")
}

// ─── Responsive / overflow helpers ───────────────────────────────────────────

/// Emit BoxWithConstraints conditional style overrides for @breakpoint rules.
/// Wraps `inner_content` with width-based conditions.
pub fn emit_breakpoint_overrides(styles: &Styles, inner_content: &str, indent: usize) -> String {
    if styles.breakpoint_overrides.is_empty() {
        return inner_content.to_string();
    }

    let pad = "    ".repeat(indent);
    let i1  = "    ".repeat(indent + 1);
    let i2  = "    ".repeat(indent + 2);

    let mut bps: Vec<(&String, &Box<Styles>)> = styles.breakpoint_overrides.iter().collect();
    bps.sort_by_key(|(k, _)| k.as_str());

    let mut bp_code = String::new();
    for (bp_name, bp_styles) in &bps {
        let threshold = match bp_name.as_str() {
            "sm" => 360, "md" => 600, "lg" => 900, "xl" => 1200, _ => 600,
        };
        let mut override_lines = String::new();
        if let Some(w)  = &bp_styles.width     { override_lines.push_str(&format!("{i2}// width: {w} at @{bp_name}\n")); }
        if let Some(fs) = &bp_styles.font_size { override_lines.push_str(&format!("{i2}// font_size: {fs} at @{bp_name}\n")); }
        if !override_lines.is_empty() {
            bp_code.push_str(&format!("{i1}if (maxWidth >= {threshold}.dp) {{\n{override_lines}{i1}}}\n"));
        }
    }

    if bp_code.is_empty() {
        return inner_content.to_string();
    }

    format!("{pad}BoxWithConstraints {{\n{bp_code}{inner_content}{pad}}}\n")
}

/// Generate `LocalConfiguration.current` screen utility bindings.
pub fn emit_screen_utilities(indent: usize) -> String {
    let pad = "    ".repeat(indent);
    format!(
        "{pad}val configuration = LocalConfiguration.current\n\
         {pad}val screenWidth = configuration.screenWidthDp.dp\n\
         {pad}val screenHeight = configuration.screenHeightDp.dp\n\
         {pad}val isPhone = configuration.screenWidthDp < 600\n\
         {pad}val isTablet = configuration.screenWidthDp in 600..899\n\
         {pad}val isLarge = configuration.screenWidthDp >= 900\n\
         {pad}val orientation = if (configuration.orientation == android.content.res.Configuration.ORIENTATION_LANDSCAPE) \"landscape\" else \"portrait\"\n"
    )
}

/// Emit LaunchedEffect-based scroll event handlers.
#[allow(dead_code)]
fn emit_scroll_config(styles: &Styles, state_name: &str, pad: &str) -> String {
    let mut out = String::new();
    let inner = format!("{pad}    ");
    if let Some(handler) = &styles.on_scroll {
        out.push_str(&format!(
            "{pad}LaunchedEffect({state_name}) {{\n{inner}snapshotFlow {{ {state_name}.value }}.collect {{ offset ->\n{inner}    {handler}(offset)\n{inner}}}\n{pad}}}\n"
        ));
    }
    if let Some(handler) = &styles.on_scroll_end {
        out.push_str(&format!("{pad}// on_scroll_end: {handler} — wire via scroll state listener\n"));
    }
    out
}

// ─── OkHttp fetch block emitter ───────────────────────────────────────────────

fn emit_fetch_block(fe: &FetchExpr, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    let i1 = "    ".repeat(indent + 1);
    let i2 = "    ".repeat(indent + 2);
    let i3 = "    ".repeat(indent + 3);

    let url = emit_expr(&fe.url);
    let method = fe.method.to_lowercase();

    let then_code: String = fe
        .then_branch
        .iter()
        .map(|s| emit_stmt(s, indent + 3))
        .collect();
    let catch_code: String = fe
        .catch_branch
        .iter()
        .map(|s| emit_stmt(s, indent + 3))
        .collect();

    format!(
        r#"{pad}LaunchedEffect(Unit) {{
{i1}withContext(Dispatchers.IO) {{
{i2}val client = OkHttpClient()
{i2}val request = Request.Builder()
{i3}.url({url})
{i3}.{method}()
{i3}.build()
{i2}try {{
{i3}val response = client.newCall(request).execute()
{i3}val body = response.body?.string()
{then_code}{i2}}} catch (e: Exception) {{
{catch_code}{i2}}}
{i1}}}
{pad}}}
"#
    )
}

// ─── Animation emitter ────────────────────────────────────────────────────────

fn emit_animation(anim: &Animation, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    let i1 = "    ".repeat(indent + 1);
    let i2 = "    ".repeat(indent + 2);

    let interp = easing_to_interpolator(&anim.easing);
    let repeat_mode = if anim.repeat {
        format!("{i2}repeatCount = ValueAnimator.INFINITE\n")
    } else {
        String::new()
    };
    let auto_reverse = if anim.auto_reverse {
        format!("{i2}repeatMode = ValueAnimator.REVERSE\n")
    } else {
        String::new()
    };

    let var_name = format!(
        "{}{}Animator",
        anim.property,
        anim.duration_ms
    );

    format!(
        r#"{pad}val {var_name} = remember {{
{i1}ValueAnimator.ofFloat({from}f, {to}f).apply {{
{i2}duration = {dur}
{i2}interpolator = {interp}
{repeat_mode}{auto_reverse}{i2}start()
{i1}}}
{pad}}}
"#,
        from = anim.from,
        to = anim.to,
        dur = anim.duration_ms,
    )
}

fn easing_to_interpolator(easing: &EasingType) -> &'static str {
    match easing {
        EasingType::Linear => "android.view.animation.LinearInterpolator()",
        EasingType::EaseIn => "android.view.animation.AccelerateInterpolator()",
        EasingType::EaseOut => "android.view.animation.DecelerateInterpolator()",
        EasingType::EaseInOut => "android.view.animation.AccelerateDecelerateInterpolator()",
        EasingType::Bounce => "android.view.animation.BounceInterpolator()",
        EasingType::Spring => "android.view.animation.OvershootInterpolator()",
    }
}

// ─── Expression emitter ───────────────────────────────────────────────────────

/// Emit an AST Expr as a Kotlin expression string.
pub fn emit_expr(expr: &Expr) -> String {
    match expr {
        Expr::Literal(v) => emit_value(v),
        Expr::Var(name) => name.clone(),
        Expr::StateField(name) => name.clone(),
        Expr::StoreField(store, field) => format!("{store}.{field}"),
        Expr::BinOp(left, op, right) => {
            let l = emit_expr(left);
            let r = emit_expr(right);
            let op_str = match op {
                Op::Add => "+",
                Op::Sub => "-",
                Op::Mul => "*",
                Op::Div => "/",
                Op::Mod => "%",
                Op::Eq => "==",
                Op::Ne => "!=",
                Op::Lt => "<",
                Op::Le => "<=",
                Op::Gt => ">",
                Op::Ge => ">=",
                Op::And => "&&",
                Op::Or => "||",
                Op::Not => "!",
            };
            format!("{l} {op_str} {r}")
        }
        Expr::Call(c) if c.func == "navigate" => {
            let route = c.args.first().map(|a| emit_expr(a)).unwrap_or_default();
            format!("navController.navigate({route})")
        }
        Expr::Call(c) if c.func == "navigate_back" => {
            "navController.popBackStack()".to_string()
        }
        Expr::Call(c) => {
            let args: String = c.args.iter().map(|a| emit_expr(a)).collect::<Vec<_>>().join(", ");
            format!("{}({args})", c.func)
        }
        Expr::NullCoalesce(a, b) => format!("{} ?: {}", emit_expr(a), emit_expr(b)),
        Expr::SafeNav(parts) => parts.join("?."),
        Expr::MethodCall(receiver, method, args) => {
            let r = emit_expr(receiver);
            let a: String = args.iter().map(|a| emit_expr(a)).collect::<Vec<_>>().join(", ");
            format!("{r}.{method}({a})")
        }
        Expr::Lambda(params, body) => {
            let p = params.join(", ");
            let stmts: String = body.iter().map(|s| emit_stmt(s, 1)).collect();
            format!("{{ {p} -> {stmts} }}")
        }
    }
}

fn emit_value(v: &Value) -> String {
    match v {
        Value::Str(s) => format!("\"{s}\""),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => format!("{f}f"),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::List(items) => {
            let inner: String = items
                .iter()
                .map(|i| emit_value(i))
                .collect::<Vec<_>>()
                .join(", ");
            format!("listOf({inner})")
        }
        Value::Object(fields) => {
            let inner: String = fields
                .iter()
                .map(|(k, v)| format!("{k} to {}", emit_value(v)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("mapOf({inner})")
        }
    }
}

// ─── Statement emitter ────────────────────────────────────────────────────────

/// Emit a Stmt as a Kotlin statement string with indentation.
pub fn emit_stmt(stmt: &Stmt, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    match stmt {
        Stmt::VarDecl(vd) => {
            let kt_type = frtype_to_kotlin(&vd.type_);
            match &vd.initializer {
                Some(init) => format!("{pad}var {}: {kt_type} = {}\n", vd.name, emit_expr(init)),
                None => format!("{pad}var {}: {kt_type} = {}\n", vd.name, default_for_type(&vd.type_)),
            }
        }
        Stmt::Assign(name, expr) => format!("{pad}{name} = {}\n", emit_expr(expr)),
        Stmt::Return(expr) => format!("{pad}return {}\n", emit_expr(expr)),
        Stmt::Call(c) => {
            let args: String = c.args.iter().map(|a| emit_expr(a)).collect::<Vec<_>>().join(", ");
            format!("{pad}{}({args})\n", c.func)
        }
        Stmt::Wait(c) => {
            let args: String = c.args.iter().map(|a| emit_expr(a)).collect::<Vec<_>>().join(", ");
            format!("{pad}{}({args})\n", c.func)
        }
        Stmt::WaitFetch(fe) => emit_fetch_block(fe, indent),
        Stmt::If(cond, then, else_) => {
            let then_stmts: String = then.iter().map(|s| emit_stmt(s, indent + 1)).collect();
            let else_part = if let Some(e) = else_ {
                let else_stmts: String = e.iter().map(|s| emit_stmt(s, indent + 1)).collect();
                format!(" else {{\n{else_stmts}{pad}}}")
            } else {
                String::new()
            };
            format!("{pad}if ({}) {{\n{then_stmts}{pad}}}{else_part}\n", emit_expr(cond))
        }
        Stmt::For(var, iter, body) => {
            let body_stmts: String = body.iter().map(|s| emit_stmt(s, indent + 1)).collect();
            format!(
                "{pad}for ({var} in {}) {{\n{body_stmts}{pad}}}\n",
                emit_expr(iter)
            )
        }
        Stmt::Switch(expr, cases) => {
            let cases_str: String = cases
                .iter()
                .map(|(cond, body)| {
                    let body_stmts: String = body.iter().map(|s| emit_stmt(s, indent + 2)).collect();
                    format!(
                        "{}    {} -> {{\n{}{pad}    }}\n",
                        pad,
                        emit_expr(cond),
                        body_stmts
                    )
                })
                .collect();
            format!("{pad}when ({}) {{\n{cases_str}{pad}}}\n", emit_expr(expr))
        }
        Stmt::TryCatch { body, catch_param, catch_body, finally_body } => {
            let body_stmts: String = body.iter().map(|s| emit_stmt(s, indent + 1)).collect();
            let catch_stmts: String = catch_body.iter().map(|s| emit_stmt(s, indent + 1)).collect();
            let finally_part = if let Some(f) = finally_body {
                let f_stmts: String = f.iter().map(|s| emit_stmt(s, indent + 1)).collect();
                format!(" finally {{\n{f_stmts}{pad}}}")
            } else {
                String::new()
            };
            format!(
                "{pad}try {{\n{body_stmts}{pad}}} catch ({catch_param}: Exception) {{\n{catch_stmts}{pad}}}{finally_part}\n"
            )
        }
        Stmt::PluginCall(pc) => {
            // Emit plugin bridge call: resolve to the Kotlin method from the plugin's android/ bridge
            let params_str: String = pc.params.iter()
                .map(|(k, v)| format!("{} = {}", k, emit_expr(v)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{pad}{}Plugin.{}({params_str})\n", pascal_case(&pc.plugin_name), pc.method)
        }
    }
}

// ─── Alignment mapping ────────────────────────────────────────────────────────

/// Convert a Frame StackAlignment to a Compose Alignment constant string.
pub fn stack_alignment_to_compose(alignment: &StackAlignment) -> &'static str {
    match alignment {
        StackAlignment::TopLeft => "Alignment.TopStart",
        StackAlignment::TopCenter => "Alignment.TopCenter",
        StackAlignment::TopRight => "Alignment.TopEnd",
        StackAlignment::CenterLeft => "Alignment.CenterStart",
        StackAlignment::Center => "Alignment.Center",
        StackAlignment::CenterRight => "Alignment.CenterEnd",
        StackAlignment::BottomLeft => "Alignment.BottomStart",
        StackAlignment::BottomCenter => "Alignment.BottomCenter",
        StackAlignment::BottomRight => "Alignment.BottomEnd",
    }
}

// ─── Utility helpers ──────────────────────────────────────────────────────────

/// Parse a dp string like "16dp", "16", "100%" → Some(16) or None for %.
fn parse_dp(s: &str) -> Option<i64> {
    let s = s.trim().trim_end_matches("dp").trim();
    if s.ends_with('%') {
        return None;
    }
    s.parse::<i64>().ok()
}

/// Convert a color string (hex #RRGGBB or named) to a Compose Color literal.
fn color_to_compose(color: &str) -> String {
    let c = color.trim().trim_start_matches('#');
    if c.len() == 6 {
        format!("Color(0xFF{c})")
    } else if c.len() == 8 {
        format!("Color(0x{c})")
    } else {
        // Named color fallback
        format!("Color.{}", pascal_case(color))
    }
}

/// Convert a Frame type to a Kotlin type name.
fn frtype_to_kotlin(t: &FRType) -> &'static str {
    match t {
        FRType::String_ => "String",
        FRType::Int => "Int",
        FRType::Float => "Float",
        FRType::Bool => "Boolean",
        FRType::Object => "Any",
        FRType::List => "List<Any>",
        FRType::Nullable(_) => "Any?",
    }
}

/// Return a sensible Kotlin default value for a Frame type.
fn default_for_type(t: &FRType) -> String {
    match t {
        FRType::String_ => "\"\"".to_string(),
        FRType::Int => "0".to_string(),
        FRType::Float => "0f".to_string(),
        FRType::Bool => "false".to_string(),
        FRType::List => "emptyList<Any>()".to_string(),
        FRType::Object => "Any()".to_string(),
        FRType::Nullable(_) => "null".to_string(),
    }
}

/// Convert snake_case or kebab-case to PascalCase.
fn pascal_case(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-' || c == ':')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

// ─── :obj code generator (Android) ───────────────────────────────────────────

/// Generate a Kotlin data class for a `:obj` declaration.
/// e.g. `:obj User { id: string  name: string  email: string? }`
/// → `data class User(val id: String, val name: String, val email: String?)`
pub fn gen_obj_data_class(obj: &ObjDef, pkg: &str, pkg_path: &str) -> OutputFile {
    let fields: Vec<String> = obj.fields.iter().map(|f| {
        let kt = frtype_to_kotlin(&f.type_);
        if f.optional {
            format!("    val {}: {}?", f.name, kt)
        } else {
            format!("    val {}: {}", f.name, kt)
        }
    }).collect();

    let fields_str = fields.join(",\n");

    OutputFile {
        path: format!("app/src/main/java/{pkg_path}/{}.kt", obj.name),
        content: format!(
r#"package {pkg}

import com.google.gson.annotations.SerializedName

/**
 * {name} — generated from :obj declaration in Frame source.
 * Do not edit manually; re-run `frame build` to regenerate.
 */
data class {name}(
{fields}
)
"#,
            pkg   = pkg,
            name  = obj.name,
            fields = fields_str,
        ),
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_config() -> AndroidConfig {
        AndroidConfig::default()
    }

    #[test]
    fn test_gen_android_empty_ast_produces_core_files() {
        let ast = AST::default();
        let files = gen_android(&ast, &minimal_config());
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.iter().any(|p| p.contains("settings.gradle")));
        assert!(paths.iter().any(|p| p.contains("AndroidManifest.xml")));
        assert!(paths.iter().any(|p| p.contains("MainActivity.kt")));
        assert!(paths.iter().any(|p| p.contains("MainApplication.kt")));
    }

    #[test]
    fn test_gen_page_screen_file_created() {
        let mut ast = AST::default();
        ast.pages.push(Page {
            name: "Home".to_string(),
            route: "/".to_string(),
            ..Default::default()
        });
        let files = gen_android(&ast, &minimal_config());
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.iter().any(|p| p.contains("HomeScreen.kt")));
    }

    #[test]
    fn test_manifest_includes_internet_permission_when_fetch_used() {
        let mut ast = AST::default();
        let mut func = Function {
            name: "load".to_string(),
            is_async: true,
            params: vec![],
            return_type: None,
            body: vec![],
        };
        func.body.push(Stmt::WaitFetch(FetchExpr {
            url: Expr::Literal(Value::Str("https://api.example.com".to_string())),
            method: "GET".to_string(),
            ..Default::default()
        }));
        ast.functions.insert("load".to_string(), func);
        let files = gen_android(&ast, &minimal_config());
        let manifest = files
            .iter()
            .find(|f| f.path.contains("AndroidManifest.xml"))
            .unwrap();
        assert!(manifest.content.contains("INTERNET"));
    }

    #[test]
    fn test_stack_alignment_mapping() {
        assert_eq!(stack_alignment_to_compose(&StackAlignment::TopLeft), "Alignment.TopStart");
        assert_eq!(stack_alignment_to_compose(&StackAlignment::Center), "Alignment.Center");
        assert_eq!(stack_alignment_to_compose(&StackAlignment::BottomRight), "Alignment.BottomEnd");
    }

    #[test]
    fn test_component_def_generates_kt_file() {
        let mut ast = AST::default();
        let comp = ComponentDef {
            name: "Card".to_string(),
            ..Default::default()
        };
        ast.components.insert("Card".to_string(), comp);
        let files = gen_android(&ast, &minimal_config());
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.iter().any(|p| p.contains("Card.kt")));
    }

    #[test]
    fn test_app_build_gradle_contains_okhttp_when_fetch_used() {
        let file = gen_app_build_gradle(&minimal_config(), true);
        assert!(file.content.contains("okhttp"));
    }

    #[test]
    fn test_app_build_gradle_no_okhttp_when_not_used() {
        let file = gen_app_build_gradle(&minimal_config(), false);
        assert!(!file.content.contains("okhttp"));
    }

    #[test]
    fn test_gradle_wrapper_targets_gradle_8() {
        let file = gen_gradle_wrapper();
        assert!(file.content.contains("gradle-8."));
    }

    #[test]
    fn test_emit_expr_literal_string() {
        let expr = Expr::Literal(Value::Str("hello".to_string()));
        assert_eq!(emit_expr(&expr), "\"hello\"");
    }

    #[test]
    fn test_emit_expr_binop() {
        let expr = Expr::BinOp(
            Box::new(Expr::Literal(Value::Int(1))),
            Op::Add,
            Box::new(Expr::Literal(Value::Int(2))),
        );
        assert_eq!(emit_expr(&expr), "1 + 2");
    }

    #[test]
    fn test_parse_dp() {
        assert_eq!(parse_dp("16dp"), Some(16));
        assert_eq!(parse_dp("100"), Some(100));
        assert_eq!(parse_dp("100%"), None);
    }

    #[test]
    fn test_color_to_compose_hex() {
        assert_eq!(color_to_compose("#FF5733"), "Color(0xFFFF5733)");
    }

    #[test]
    fn test_pascal_case() {
        assert_eq!(pascal_case("snake_case"), "SnakeCase");
        assert_eq!(pascal_case("kebab-case"), "KebabCase");
    }

    #[test]
    fn test_manifest_no_permissions_when_nothing_used() {
        let config = minimal_config();
        let file = gen_manifest(&config, false, false, false, false);
        assert!(!file.content.contains("uses-permission"));
    }

    #[test]
    fn test_main_activity_uses_first_page_as_start_destination() {
        let mut ast = AST::default();
        ast.pages.push(Page {
            name: "Login".to_string(),
            route: "/login".to_string(),
            ..Default::default()
        });
        ast.pages.push(Page {
            name: "Home".to_string(),
            route: "/home".to_string(),
            ..Default::default()
        });
        let file = gen_main_activity(&ast, "com.example.app");
        assert!(file.content.contains("startDestination = \"/login\""));
    }

    // ── new tests ──────────────────────────────────────────────────────

    #[test]
    fn test_store_viewmodel_generated_for_each_store() {
        let mut ast = AST::default();
        let store = StoreSlice {
            name: "Auth".to_string(),
            fields: HashMap::new(),
            actions: HashMap::new(),
            persist: HashMap::new(),
        };
        ast.stores.insert("Auth".to_string(), store);

        let files = gen_android(&ast, &minimal_config());
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(
            paths.iter().any(|p| p.contains("AuthViewModel.kt")),
            "Expected AuthViewModel.kt in output; got: {paths:?}"
        );
    }

    #[test]
    fn test_store_viewmodel_has_secure_prefs_for_token_field() {
        let mut store = StoreSlice {
            name: "Auth".to_string(),
            fields: HashMap::new(),
            actions: HashMap::new(),
            persist: HashMap::new(),
        };
        store.fields.insert(
            "token".to_string(),
            StoreField {
                name: "token".to_string(),
                type_: FRType::String_,
                default: Some(Expr::Literal(Value::Str("".to_string()))),
            },
        );
        store.persist.insert("token".to_string(), PersistStrategy::Secure);

        let file = gen_store_viewmodel("Auth", &store, "com.example.app", "com/example/app");
        assert!(
            file.content.contains("EncryptedSharedPreferences"),
            "Expected EncryptedSharedPreferences in content:\n{}",
            file.content
        );
        assert!(
            file.content.contains("MasterKey"),
            "Expected MasterKey in content"
        );
    }

    #[test]
    fn test_store_viewmodel_has_shared_prefs_for_local_field() {
        let mut store = StoreSlice {
            name: "User".to_string(),
            fields: HashMap::new(),
            actions: HashMap::new(),
            persist: HashMap::new(),
        };
        store.fields.insert(
            "name".to_string(),
            StoreField {
                name: "name".to_string(),
                type_: FRType::String_,
                default: Some(Expr::Literal(Value::Str("".to_string()))),
            },
        );
        store.persist.insert("name".to_string(), PersistStrategy::Local);

        let file = gen_store_viewmodel("User", &store, "com.example.app", "com/example/app");
        assert!(
            file.content.contains("getSharedPreferences"),
            "Expected getSharedPreferences in content:\n{}",
            file.content
        );
        assert!(
            !file.content.contains("EncryptedSharedPreferences"),
            "Should NOT contain EncryptedSharedPreferences for local persist"
        );
    }

    #[test]
    fn test_navigate_expr_emits_navcontroller() {
        let expr = Expr::Call(CallExpr {
            func: "navigate".to_string(),
            args: vec![Expr::Literal(Value::Str("/home".to_string()))],
        });
        let result = emit_expr(&expr);
        assert_eq!(result, "navController.navigate(\"/home\")");
    }

    #[test]
    fn test_navigate_back_expr_emits_popbackstack() {
        let expr = Expr::Call(CallExpr {
            func: "navigate_back".to_string(),
            args: vec![],
        });
        let result = emit_expr(&expr);
        assert_eq!(result, "navController.popBackStack()");
    }

    #[test]
    fn test_camera_helper_generated_when_camera_used() {
        let mut ast = AST::default();
        let mut func = Function {
            name: "capture".to_string(),
            is_async: false,
            params: vec![],
            return_type: None,
            body: vec![Stmt::Call(CallExpr {
                func: "camera:capture".to_string(),
                args: vec![],
            })],
        };
        // Put the call inside a component event handler so ast_uses_call finds it
        let mut node = ComponentNode::default();
        node.events.on_click = Some(Expr::Call(CallExpr {
            func: "camera:capture".to_string(),
            args: vec![],
        }));
        let mut page = Page {
            name: "CamPage".to_string(),
            route: "/cam".to_string(),
            ..Default::default()
        };
        page.children.push(node);
        ast.pages.push(page);
        // Also put it in a top-level function so ast_uses_call finds it via stmts
        ast.functions.insert("capture".to_string(), func);

        let files = gen_android(&ast, &minimal_config());
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(
            paths.iter().any(|p| p.contains("CameraHelper.kt")),
            "Expected CameraHelper.kt; got: {paths:?}"
        );
        let camera_file = files.iter().find(|f| f.path.contains("CameraHelper.kt")).unwrap();
        assert!(camera_file.content.contains("ActivityResultLauncher"));
    }

    #[test]
    fn test_location_helper_generated_when_location_used() {
        let mut ast = AST::default();
        let func = Function {
            name: "getloc".to_string(),
            is_async: false,
            params: vec![],
            return_type: None,
            body: vec![Stmt::Call(CallExpr {
                func: "location:get".to_string(),
                args: vec![],
            })],
        };
        ast.functions.insert("getloc".to_string(), func);

        let files = gen_android(&ast, &minimal_config());
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(
            paths.iter().any(|p| p.contains("LocationHelper.kt")),
            "Expected LocationHelper.kt; got: {paths:?}"
        );
        let loc_file = files.iter().find(|f| f.path.contains("LocationHelper.kt")).unwrap();
        assert!(loc_file.content.contains("FusedLocationProviderClient"));
    }

    #[test]
    fn test_notification_helper_generated_when_notification_used() {
        let mut ast = AST::default();
        let func = Function {
            name: "notify".to_string(),
            is_async: false,
            params: vec![],
            return_type: None,
            body: vec![Stmt::Call(CallExpr {
                func: "notification:send".to_string(),
                args: vec![],
            })],
        };
        ast.functions.insert("notify".to_string(), func);

        let files = gen_android(&ast, &minimal_config());
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(
            paths.iter().any(|p| p.contains("NotificationHelper.kt")),
            "Expected NotificationHelper.kt; got: {paths:?}"
        );
        let notif_file = files.iter().find(|f| f.path.contains("NotificationHelper.kt")).unwrap();
        assert!(notif_file.content.contains("NotificationChannel"));
        assert!(notif_file.content.contains("NotificationManager"));
    }

    #[test]
    fn test_nav_route_with_params_generates_nav_argument() {
        let page = Page {
            name: "Profile".to_string(),
            route: "/profile/:userId".to_string(),
            ..Default::default()
        };
        let route_str = gen_nav_route(&page);
        assert!(route_str.contains("navArgument(\"userId\")"), "Expected navArgument: {route_str}");
        assert!(route_str.contains("/profile/{userId}"), "Expected kotlin-style param: {route_str}");
        assert!(route_str.contains("backStackEntry"), "Expected backStackEntry: {route_str}");
        assert!(route_str.contains("getString(\"userId\")"), "Expected getString: {route_str}");
    }

    #[test]
    fn test_nav_route_without_params_is_simple() {
        let page = Page {
            name: "Home".to_string(),
            route: "/home".to_string(),
            ..Default::default()
        };
        let route_str = gen_nav_route(&page);
        assert!(!route_str.contains("navArgument"), "Should have no navArgument: {route_str}");
        assert!(route_str.contains("composable(\"/home\")"), "Expected simple composable: {route_str}");
    }

    #[test]
    fn test_store_viewmodel_has_state_flows() {
        let mut store = StoreSlice {
            name: "Counter".to_string(),
            fields: HashMap::new(),
            actions: HashMap::new(),
            persist: HashMap::new(),
        };
        store.fields.insert(
            "count".to_string(),
            StoreField {
                name: "count".to_string(),
                type_: FRType::Int,
                default: Some(Expr::Literal(Value::Int(0))),
            },
        );

        let file = gen_store_viewmodel("Counter", &store, "com.example.app", "com/example/app");
        assert!(file.content.contains("MutableStateFlow(0)"), "Expected MutableStateFlow(0): {}", file.content);
        assert!(file.content.contains("StateFlow<Int>"), "Expected StateFlow<Int>: {}", file.content);
        assert!(file.content.contains("asStateFlow()"), "Expected asStateFlow(): {}", file.content);
    }

    #[test]
    fn test_store_viewmodel_async_action_uses_viewmodelscope() {
        let mut store = StoreSlice {
            name: "Auth".to_string(),
            fields: HashMap::new(),
            actions: HashMap::new(),
            persist: HashMap::new(),
        };
        let func = Function {
            name: "login".to_string(),
            is_async: true,
            params: vec![],
            return_type: None,
            body: vec![],
        };
        store.actions.insert("login".to_string(), func);

        let file = gen_store_viewmodel("Auth", &store, "com.example.app", "com/example/app");
        assert!(file.content.contains("viewModelScope.launch"), "Expected viewModelScope.launch: {}", file.content);
        assert!(file.content.contains("withContext(Dispatchers.Main)"), "Expected withContext: {}", file.content);
    }

    // ── Overflow & Responsive tests ──────────────────────────────────

    #[test]
    fn test_overflow_hidden_with_border_radius_emits_clip() {
        let mut styles = Styles::default();
        styles.overflow = OverflowValue::Hidden;
        styles.border_radius = Some("8dp".to_string());
        let m = emit_modifier(&styles);
        assert!(m.contains("clip(RoundedCornerShape(8.dp))"), "got: {m}");
    }

    #[test]
    fn test_overflow_hidden_without_border_radius_emits_clip_to_bounds() {
        let mut styles = Styles::default();
        styles.overflow = OverflowValue::Hidden;
        let m = emit_modifier(&styles);
        assert!(m.contains("clipToBounds()"), "got: {m}");
    }

    #[test]
    fn test_overflow_scroll_y_emits_vertical_scroll() {
        let mut styles = Styles::default();
        styles.overflow = OverflowValue::ScrollY;
        let m = emit_modifier(&styles);
        assert!(m.contains("verticalScroll(rememberScrollState())"), "got: {m}");
    }

    #[test]
    fn test_overflow_scroll_x_emits_horizontal_scroll() {
        let mut styles = Styles::default();
        styles.overflow = OverflowValue::ScrollX;
        let m = emit_modifier(&styles);
        assert!(m.contains("horizontalScroll(rememberScrollState())"), "got: {m}");
    }

    #[test]
    fn test_text_overflow_ellipsis_emits_text_overflow() {
        let mut node = ComponentNode::default();
        node.kind = "text".to_string();
        node.styles.text_overflow = TextOverflowValue::Ellipsis;
        node.styles.max_lines = Some(3);
        let result = emit_text_node(&node, "    ");
        assert!(result.contains("TextOverflow.Ellipsis"), "got: {result}");
        assert!(result.contains("maxLines = 3"), "got: {result}");
    }

    #[test]
    fn test_text_max_lines_ellipsis_both_present() {
        // Property 15: max_lines + ellipsis → both present in output
        let mut node = ComponentNode::default();
        node.kind = "text".to_string();
        node.styles.max_lines = Some(2);
        node.styles.text_overflow = TextOverflowValue::Ellipsis;
        let result = emit_text_node(&node, "");
        assert!(result.contains("maxLines = 2"), "missing maxLines: {result}");
        assert!(result.contains("TextOverflow.Ellipsis"), "missing TextOverflow.Ellipsis: {result}");
    }

    #[test]
    fn test_image_fit_cover_emits_content_scale_crop() {
        let mut node = ComponentNode::default();
        node.kind = "image".to_string();
        node.styles.image_fit = ImageFitValue::Cover;
        let result = emit_image_node(&node, "    ");
        assert!(result.contains("ContentScale.Crop"), "got: {result}");
    }

    #[test]
    fn test_image_fit_fill_emits_fill_bounds() {
        let mut node = ComponentNode::default();
        node.kind = "image".to_string();
        node.styles.image_fit = ImageFitValue::Fill;
        let result = emit_image_node(&node, "");
        assert!(result.contains("ContentScale.FillBounds"), "got: {result}");
    }

    #[test]
    fn test_image_fit_contain_does_not_emit_content_scale() {
        // Contain is AsyncImage default — skip emitting it
        let node = ComponentNode::default();
        let result = emit_image_node(&node, "");
        assert!(!result.contains("contentScale"), "should not emit default contentScale: {result}");
    }

    #[test]
    fn test_emit_screen_utilities_contains_configuration() {
        let result = emit_screen_utilities(1);
        assert!(result.contains("LocalConfiguration.current"), "got: {result}");
        assert!(result.contains("screenWidth"), "got: {result}");
        assert!(result.contains("isPhone"), "got: {result}");
    }

    #[test]
    fn test_emit_breakpoint_overrides_wraps_in_box_with_constraints() {
        let mut styles = Styles::default();
        let mut bp_styles = Styles::default();
        bp_styles.width = Some("75%".to_string());
        styles.breakpoint_overrides.insert("md".to_string(), Box::new(bp_styles));
        let result = emit_breakpoint_overrides(&styles, "    Text(\"hi\")\n", 0);
        assert!(result.contains("BoxWithConstraints"), "got: {result}");
        assert!(result.contains("600"), "expected md threshold 600dp: {result}");
    }

    #[test]
    fn test_emit_breakpoint_overrides_no_wrap_when_empty() {
        let styles = Styles::default();
        let inner = "    Text(\"hi\")\n";
        let result = emit_breakpoint_overrides(&styles, inner, 0);
        assert_eq!(result, inner, "should pass through unchanged when no breakpoints");
    }

    #[test]
    fn test_show_if_generates_if_block_not_zero_size() {
        let mut node = ComponentNode::default();
        node.kind = "text".to_string();
        node.show_if = Some(Expr::Literal(Value::Bool(true)));
        node.props.insert("content".to_string(), Expr::Literal(Value::Str("hi".to_string())));
        let result = emit_composable(&node, 0);
        assert!(result.contains("if (true)"), "expected if block: {result}");
        assert!(!result.contains("Modifier.size(0"), "should not have zero-size wrapper: {result}");
    }

    #[test]
    fn test_overflow_hidden_invariant_in_generated_output() {
        // Property 14: overflow:hidden always emits clip in output
        let mut ast = AST::default();
        let mut styles = Styles::default();
        styles.overflow = OverflowValue::Hidden;
        let mut node = ComponentNode::default();
        node.kind = "container".to_string();
        node.styles = styles;
        let mut page = Page { name: "P".to_string(), route: "/p".to_string(), ..Default::default() };
        page.children.push(node);
        ast.pages.push(page);
        let files = gen_android(&ast, &minimal_config());
        let screen = files.iter().find(|f| f.path.contains("PScreen.kt")).unwrap();
        assert!(
            screen.content.contains("clipToBounds()") || screen.content.contains("clip("),
            "overflow:hidden must emit clip: {}", screen.content
        );
    }
}
