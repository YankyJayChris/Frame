//! iOS / UIKit code generator for the Frame framework.
//!
//! Entry point: `gen_ios(ast, config) -> Vec<OutputFile>`
//! Mirrors android.rs — every built-in component has a UIKit mapping.

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
    let mut files: Vec<OutputFile> = Vec::new();
    let bundle_id = &config.bundle_id;
    let app_name = &config.app_name;

    // Detect features
    let uses_fetch       = ios_uses_fetch(ast);
    let uses_camera      = ios_uses_call(ast, "camera:capture");
    let uses_location    = ios_uses_call(ast, "location:get");
    let uses_notification = ios_uses_call(ast, "notification:send");
    let uses_http        = uses_fetch; // NSAppTransportSecurity for HTTP

    // Project scaffolding
    files.push(gen_info_plist(config, uses_camera, uses_location, uses_http));
    files.push(gen_app_delegate(bundle_id, app_name));
    files.push(gen_scene_delegate(bundle_id));
    files.push(gen_assets_xcassets());
    files.push(gen_podfile(config, ast));

    // Entry point: main screen controller wiring
    files.push(gen_main_view_controller(ast, bundle_id));

    // Per-page ViewControllers
    for page in &ast.pages {
        files.push(gen_page_view_controller(page, ast, bundle_id));
    }

    // Custom components → UIView subclasses
    for (name, comp) in &ast.components {
        files.push(gen_component_view(name, comp, bundle_id));
    }

    // Store ObservableObjects
    for (name, store) in &ast.stores {
        files.push(gen_store_swift(name, store, bundle_id));
    }

    // KeychainHelper (always generated — stores may need it)
    files.push(gen_keychain_helper());

    // Route helper (navigate() calls in Swift use this)
    files.push(gen_route_helper(ast));

    // Platform feature helpers
    if uses_camera      { files.push(gen_camera_helper_swift(bundle_id)); }
    if uses_location    { files.push(gen_location_helper_swift(bundle_id)); }
    if uses_notification { files.push(gen_notification_helper_swift(bundle_id)); }

    files
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

fn gen_info_plist(config: &IosConfig, uses_camera: bool, uses_location: bool, uses_http: bool) -> OutputFile {
    let mut extras = String::new();
    if uses_camera {
        extras.push_str("\t<key>NSCameraUsageDescription</key>\n\t<string>This app uses the camera.</string>\n");
    }
    if uses_location {
        extras.push_str("\t<key>NSLocationWhenInUseUsageDescription</key>\n\t<string>This app uses your location.</string>\n");
    }
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
	<key>UILaunchStoryboardName</key>
	<string>LaunchScreen</string>
	<key>UIMainStoryboardFile</key>
	<string>Main</string>
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

fn gen_app_delegate(bundle_id: &str, app_name: &str) -> OutputFile {
    OutputFile {
        path: format!("{}/AppDelegate.swift", app_name.replace(' ', "")),
        content: format!(r#"// AppDelegate.swift — {app_name}
import UIKit

@main
class AppDelegate: UIResponder, UIApplicationDelegate {{
    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {{
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
"#, app_name = app_name),
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
    let before_appear = page.before_enter.as_ref()
        .map(|f| format!("        {}()\n", f)).unwrap_or_default();
    let before_disappear = page.before_leave.as_ref()
        .map(|f| format!("        {}()\n", f)).unwrap_or_default();

    let setup_ui: String = page.children.iter()
        .map(|n| emit_uikit_view(n, "view", 2))
        .collect();

    let has_fetch = page.children.iter().any(|c| c.build.is_some());

    OutputFile {
        path: format!("{}ViewController.swift", page.name),
        content: format!(r#"import UIKit

class {name}ViewController: UIViewController {{
{state_props}
    override func viewDidLoad() {{
        super.viewDidLoad()
        view.backgroundColor = .systemBackground
        title = "{title}"
        setupUI()
    }}

    override func viewWillAppear(_ animated: Bool) {{
        super.viewWillAppear(animated)
{before_appear}    }}

    override func viewWillDisappear(_ animated: Bool) {{
        super.viewWillDisappear(animated)
{before_disappear}    }}

    private func setupUI() {{
{setup_ui}    }}
}}
"#,
            name  = page.name,
            title = page.name,
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
            .map(|(pn, pt)| { let (t, _) = swift_type_default(pt); format!("{pn}: {t}") })
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
        "app_bar"          => emit_ios_navigation_bar(node, parent, &pad),
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
        body
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

fn emit_ios_navigation_bar(node: &ComponentNode, parent: &str, pad: &str) -> String {
    let title = node.props.get("title").map(|e| emit_swift_expr(e)).unwrap_or_else(|| "\"\"".to_string());
    format!("{pad}navigationItem.title = {title}\n")
}

fn emit_ios_tab_bar(node: &ComponentNode, parent: &str, pad: &str, _i1: &str, indent: usize) -> String {
    let var = fresh_var("tabBar");
    format!("{pad}let {var} = UITabBar()\n{pad}{parent}.addSubview({var})\n")
}

fn emit_ios_scaffold(node: &ComponentNode, parent: &str, pad: &str, _i1: &str, indent: usize) -> String {
    let children: String = node.children.iter().map(|c| emit_uikit_view(c, parent, indent + 1)).collect();
    children
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
    format!("{pad}// tooltip: {text} (use UIPopoverPresentationController or a custom tooltip library)\n")
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
    let on_change = node.events.on_change.as_ref().map(|e| emit_swift_expr(e)).unwrap_or_else(|| "{}".to_string());
    format!("{pad}// color_picker: use UIColorPickerViewController (iOS 14+)\n\
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
    format!("{pad}// audio_player: AVAudioPlayer from {src}\n")
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
    format!("{pad}// qr_scanner: AVFoundation metadataOutput\n\
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
        Expr::Call(c) => {
            let args: String = c.args.iter().map(|a| emit_swift_expr(a)).collect::<Vec<_>>().join(", ");
            format!("{}({})", c.func, args)
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
        Stmt::Assign(name, expr)  => format!("{pad}{name} = {}\n", emit_swift_expr(expr)),
        Stmt::Return(expr)        => format!("{pad}return {}\n", emit_swift_expr(expr)),
        Stmt::Call(c)             => format!("{pad}{}({})\n", c.func, c.args.iter().map(|a| emit_swift_expr(a)).collect::<Vec<_>>().join(", ")),
        Stmt::Wait(c)             => format!("{pad}await {}({})\n", c.func, c.args.iter().map(|a| emit_swift_expr(a)).collect::<Vec<_>>().join(", ")),
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
    }
}

fn emit_swift_fetch(fe: &FetchExpr, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    let url = emit_swift_expr(&fe.url);
    let method = fe.method.to_uppercase();
    let then_code: String = fe.then_branch.iter().map(|s| emit_swift_stmt(s, indent + 2)).collect();
    let catch_code: String = fe.catch_branch.iter().map(|s| emit_swift_stmt(s, indent + 2)).collect();
    format!("{pad}Task {{\n\
             {pad}    do {{\n\
             {pad}        var request = URLRequest(url: URL(string: {url})!)\n\
             {pad}        request.httpMethod = \"{method}\"\n\
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
fn swift_type_default(t: &FRType) -> (&'static str, &'static str) {
    match t {
        FRType::String_      => ("String", "\"\""),
        FRType::Int          => ("Int", "0"),
        FRType::Float        => ("Double", "0.0"),
        FRType::Bool         => ("Bool", "false"),
        FRType::Object       => ("[String: Any]", "[:]"),
        FRType::List         => ("[Any]", "[]"),
        FRType::Nullable(_)  => ("Any?", "nil"),
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
        let file = gen_info_plist(&config, true, false, false);
        assert!(file.content.contains("NSCameraUsageDescription"));
    }

    #[test]
    fn test_info_plist_contains_location_key_when_used() {
        let config = minimal_config();
        let file = gen_info_plist(&config, false, true, false);
        assert!(file.content.contains("NSLocationWhenInUseUsageDescription"));
    }

    #[test]
    fn test_info_plist_contains_ats_when_http_used() {
        let config = minimal_config();
        let file = gen_info_plist(&config, false, false, true);
        assert!(file.content.contains("NSAppTransportSecurity"));
    }

    #[test]
    fn test_camera_helper_generated_when_camera_used() {
        let mut ast = AST::default();
        ast.functions.insert("capture".to_string(), Function {
            name: "capture".to_string(), is_async: false, params: vec![],
            return_type: None,
            body: vec![Stmt::Call(CallExpr { func: "camera:capture".to_string(), args: vec![] })],
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
            body: vec![Stmt::Call(CallExpr { func: "location:get".to_string(), args: vec![] })],
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
            body: vec![Stmt::Call(CallExpr { func: "notification:send".to_string(), args: vec![] })],
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
        });
        let result = emit_swift_expr(&expr);
        assert!(result.contains("navigationController?.pushViewController"), "got: {result}");
    }

    #[test]
    fn test_swift_expr_navigate_back_emits_pop() {
        let expr = Expr::Call(CallExpr { func: "navigate_back".to_string(), args: vec![] });
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

/// Generate a route helper that maps route strings to ViewControllers.
/// Handles route params like /profile/:userId.
pub fn gen_route_helper(ast: &AST) -> OutputFile {
    let mut cases = String::new();
    for page in &ast.pages {
        let route = &page.route;
        let name  = &page.name;
        let params: Vec<&str> = route.split('/').filter(|s| s.starts_with(':')).collect();
        if params.is_empty() {
            cases.push_str(&format!("        case \"{route}\":\n            return {name}ViewController()\n"));
        } else {
            let pattern = route.split('/').map(|seg| {
                if seg.starts_with(':') { "([^/]+)".to_string() } else { seg.to_string() }
            }).collect::<Vec<_>>().join("/");
            let param_names: Vec<String> = params.iter()
                .map(|p| p.trim_start_matches(':').to_string()).collect();
            let names_arr = param_names.iter()
                .map(|p| format!("\"{}\"", p)).collect::<Vec<_>>().join(", ");
            let vc_args = param_names.iter()
                .map(|p| format!("{}: params[\"{}\"] ?? \"\"", p, p)).collect::<Vec<_>>().join(", ");
            cases.push_str(&format!(
                "        case let r where r.matches(regex: \"{pattern}\"):\n            let params = r.extractParams(pattern: \"{pattern}\", names: [{names_arr}])\n            return {name}ViewController({vc_args})\n"
            ));
        }
    }

    let mut content = String::new();
    content.push_str("import UIKit\n\n");
    content.push_str("extension String {\n");
    content.push_str("    func matches(regex pattern: String) -> Bool {\n");
    content.push_str("        return range(of: pattern, options: .regularExpression) != nil\n");
    content.push_str("    }\n");
    content.push_str("    func extractParams(pattern: String, names: [String]) -> [String: String] {\n");
    content.push_str("        var result = [String: String]()\n");
    content.push_str("        guard let regex = try? NSRegularExpression(pattern: pattern),\n");
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
