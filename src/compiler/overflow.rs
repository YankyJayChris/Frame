//! Overflow System Integration for the Frame framework.
//!
//! Injects per-component overflow defaults and emits compiler warnings
//! for invalid overflow combinations. Used by both Android and iOS generators.

use crate::parser::ast::{AST, ComponentNode, OverflowValue};
use crate::parser::{FrameError, ErrorCategory};

// ─── Default overflow per component kind ─────────────────────────────────────

/// Container components default to `overflow: hidden`.
const CONTAINER_KINDS: &[&str] = &[
    "container", "row", "column", "card", "scaffold", "form", "modal", "banner",
    "accordion", "timeline", "tab_bar", "bottom_sheet",
];

/// Scroll components default to `overflow: scroll_y`.
const SCROLL_KINDS: &[&str] = &["scroll_view", "list", "table", "refresh"];

/// Leaf/interactive components default to `overflow: visible`.
#[allow(dead_code)]
const VISIBLE_KINDS: &[&str] = &[
    "text", "image", "icon", "button", "input", "dropdown", "divider",
    "spacer", "progress_bar", "progress_circle", "switch", "checkbox", "radio",
    "slider", "stepper", "text_area", "search_bar", "date_picker", "time_picker",
    "color_picker", "rating", "otp_input", "avatar", "chip", "tag", "badge",
    "skeleton", "toast", "tooltip",
    "video_player", "audio_player", "lottie", "web_view", "map_view",
    "camera_view", "qr_scanner",
    "swipeable", "draggable", "long_press",
];

/// Returns the default overflow value for a component kind.
pub fn default_overflow(kind: &str) -> OverflowValue {
    if CONTAINER_KINDS.contains(&kind) {
        return OverflowValue::Hidden;
    }
    if SCROLL_KINDS.contains(&kind) {
        // list with direction: row → scroll_x, otherwise scroll_y
        return OverflowValue::ScrollY;
    }
    // Leaf and all unrecognised → visible
    OverflowValue::Visible
}

/// Returns the default overflow for the page root.
pub fn page_root_overflow() -> OverflowValue {
    OverflowValue::Hidden
}

// ─── Overflow injection pass ──────────────────────────────────────────────────

/// Walk the AST and inject default overflow values wherever `overflow` was not
/// explicitly declared (i.e. still at its Rust default of `Visible`).
///
/// Also collects compiler warnings for invalid combinations.
pub fn inject_overflow_defaults(ast: &mut AST) -> Vec<FrameError> {
    let mut warnings = Vec::new();

    for page in &mut ast.pages {
        // Page root default
        if matches!(page.styles.overflow, OverflowValue::Visible) {
            page.styles.overflow = page_root_overflow();
        }
        // Walk children
        inject_nodes(&mut page.children, &mut warnings);
    }

    for comp in ast.components.values_mut() {
        inject_nodes(&mut comp.children, &mut warnings);
    }

    warnings
}

fn inject_nodes(nodes: &mut Vec<ComponentNode>, warnings: &mut Vec<FrameError>) {
    for node in nodes.iter_mut() {
        // Only inject if the user left overflow at its zero-value (Visible)
        // AND the component kind has a non-Visible default.
        if matches!(node.styles.overflow, OverflowValue::Visible) {
            let default = default_overflow(&node.kind);
            if !matches!(default, OverflowValue::Visible) {
                node.styles.overflow = default;
            }
        }

        // list: + overflow:hidden → warning + upgrade
        if node.kind == "list" && matches!(node.styles.overflow, OverflowValue::Hidden) {
            warnings.push(FrameError {
                category: ErrorCategory::ParseError,
                file: "<project>".to_string(),
                line: 0, column: 0,
                message: "overflow: hidden has no effect on list: — list always scrolls. \
                          Remove the property or use overflow: scroll_y explicitly."
                    .to_string(),
            });
            node.styles.overflow = OverflowValue::ScrollY;
        }

        // Recurse
        inject_nodes(&mut node.children, warnings);
    }
}

// ─── Android overflow code generation ────────────────────────────────────────

/// Returns the Compose Modifier fragment for a component's overflow value.
///
/// Called by `emit_modifier` in android.rs (already implemented there).
/// This function provides the canonical mapping for reference.
pub fn android_overflow_modifier(overflow: &OverflowValue, border_radius: Option<f64>) -> String {
    match overflow {
        OverflowValue::Hidden => {
            match border_radius {
                Some(r) => format!("clip(RoundedCornerShape({r}.dp))"),
                None    => "clipToBounds()".to_string(),
            }
        }
        OverflowValue::ScrollY => "verticalScroll(rememberScrollState())".to_string(),
        OverflowValue::ScrollX => "horizontalScroll(rememberScrollState())".to_string(),
        OverflowValue::Scroll  => "verticalScroll(rememberScrollState())".to_string(),
        OverflowValue::Auto    => "/* auto-scroll — BoxWithConstraints */".to_string(),
        OverflowValue::Visible => String::new(),
    }
}

// ─── iOS overflow code generation ────────────────────────────────────────────

/// Returns the UIKit Swift lines for a component's overflow value.
///
/// `var` is the Swift variable name of the UIView.
pub fn ios_overflow_code(overflow: &OverflowValue, var: &str, border_radius: Option<f64>) -> String {
    match overflow {
        OverflowValue::Hidden => {
            let clip = format!("{var}.clipsToBounds = true\n");
            let radius = border_radius.map(|r| format!("{var}.layer.cornerRadius = {r}\n"))
                .unwrap_or_default();
            format!("{clip}{radius}")
        }
        OverflowValue::ScrollY => {
            format!("let {var}ScrollView = UIScrollView()\n\
                     {var}ScrollView.showsVerticalScrollIndicator = true\n\
                     {var}ScrollView.addSubview({var})\n")
        }
        OverflowValue::ScrollX => {
            format!("let {var}ScrollView = UIScrollView()\n\
                     {var}ScrollView.showsHorizontalScrollIndicator = true\n\
                     {var}ScrollView.addSubview({var})\n")
        }
        OverflowValue::Scroll => {
            format!("let {var}ScrollView = UIScrollView()\n\
                     {var}ScrollView.addSubview({var})\n")
        }
        OverflowValue::Auto | OverflowValue::Visible => String::new(),
    }
}

// ─── scroll_snap platform mappings ───────────────────────────────────────────

use crate::parser::ast::ScrollSnap;

pub fn android_scroll_snap_code(snap: &ScrollSnap, list_var: &str) -> String {
    match snap {
        ScrollSnap::Start | ScrollSnap::End =>
            format!("val snapHelper{list_var} = LinearSnapHelper()\nsnapHelper{list_var}.attachToRecyclerView({list_var})\n"),
        ScrollSnap::Center =>
            format!("val snapHelper{list_var} = PagerSnapHelper()\nsnapHelper{list_var}.attachToRecyclerView({list_var})\n"),
        ScrollSnap::None_ => String::new(),
    }
}

pub fn ios_scroll_snap_code(snap: &ScrollSnap, scroll_var: &str) -> String {
    match snap {
        ScrollSnap::Center =>
            format!("{scroll_var}.isPagingEnabled = true\n"),
        ScrollSnap::Start | ScrollSnap::End =>
            format!("// scroll_snap {snap:?}: implement UIScrollViewDelegate.targetContentOffset\n"),
        ScrollSnap::None_ => String::new(),
    }
}

// ─── image fit platform mappings ─────────────────────────────────────────────

use crate::parser::ast::ImageFitValue;

pub fn android_image_content_scale(fit: &ImageFitValue) -> &'static str {
    match fit {
        ImageFitValue::Cover     => "ContentScale.Crop",
        ImageFitValue::Contain   => "ContentScale.Fit",
        ImageFitValue::Fill      => "ContentScale.FillBounds",
        ImageFitValue::None_     => "ContentScale.None",
        ImageFitValue::ScaleDown => "ContentScale.Inside",
    }
}

pub fn ios_image_content_mode(fit: &ImageFitValue) -> &'static str {
    match fit {
        ImageFitValue::Cover     => "scaleAspectFill",
        ImageFitValue::Contain   => "scaleAspectFit",
        ImageFitValue::Fill      => "scaleToFill",
        ImageFitValue::None_     => "center",
        ImageFitValue::ScaleDown => "topLeft",
    }
}

// ─── text overflow platform mappings ─────────────────────────────────────────

use crate::parser::ast::TextOverflowValue;

pub fn android_text_overflow(overflow: &TextOverflowValue) -> &'static str {
    match overflow {
        TextOverflowValue::Ellipsis => "TextOverflow.Ellipsis",
        TextOverflowValue::Fade     => "TextOverflow.Clip",
        TextOverflowValue::Clip     => "TextOverflow.Clip",
    }
}

pub fn ios_line_break_mode(overflow: &TextOverflowValue) -> &'static str {
    match overflow {
        TextOverflowValue::Ellipsis => ".byTruncatingTail",
        TextOverflowValue::Fade     => ".byClipping",
        TextOverflowValue::Clip     => ".byClipping",
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::*;

    // ── Property 14: overflow:hidden always emits clip ─────────────────────

    #[test]
    fn test_overflow_hidden_android_emits_clip_with_radius() {
        let code = android_overflow_modifier(&OverflowValue::Hidden, Some(8.0));
        assert!(code.contains("clip(RoundedCornerShape(8"), "got: {code}");
    }

    #[test]
    fn test_overflow_hidden_android_emits_clip_to_bounds_without_radius() {
        let code = android_overflow_modifier(&OverflowValue::Hidden, None);
        assert!(code.contains("clipToBounds()"), "got: {code}");
    }

    #[test]
    fn test_overflow_hidden_ios_emits_clip_to_bounds() {
        let code = ios_overflow_code(&OverflowValue::Hidden, "myView", None);
        assert!(code.contains("clipsToBounds = true"), "got: {code}");
    }

    #[test]
    fn test_overflow_hidden_ios_emits_corner_radius() {
        let code = ios_overflow_code(&OverflowValue::Hidden, "card", Some(12.0));
        assert!(code.contains("clipsToBounds = true"), "got: {code}");
        assert!(code.contains("cornerRadius = 12"), "got: {code}");
    }

    #[test]
    fn test_overflow_visible_android_emits_nothing() {
        let code = android_overflow_modifier(&OverflowValue::Visible, None);
        assert!(code.is_empty(), "got: {code}");
    }

    // ── Property: scroll_y emits scroll modifier ───────────────────────────

    #[test]
    fn test_overflow_scroll_y_android() {
        let code = android_overflow_modifier(&OverflowValue::ScrollY, None);
        assert!(code.contains("verticalScroll"), "got: {code}");
    }

    #[test]
    fn test_overflow_scroll_x_android() {
        let code = android_overflow_modifier(&OverflowValue::ScrollX, None);
        assert!(code.contains("horizontalScroll"), "got: {code}");
    }

    #[test]
    fn test_overflow_scroll_y_ios() {
        let code = ios_overflow_code(&OverflowValue::ScrollY, "sv", None);
        assert!(code.contains("UIScrollView"), "got: {code}");
        assert!(code.contains("Vertical"), "got: {code}");
    }

    // ── Default overflow injection ─────────────────────────────────────────

    #[test]
    fn test_inject_defaults_container_gets_hidden() {
        let mut ast = AST::default();
        let mut page = Page { name: "P".to_string(), route: "/".to_string(), ..Default::default() };
        page.children.push(ComponentNode { kind: "container".to_string(), ..Default::default() });
        ast.pages.push(page);
        inject_overflow_defaults(&mut ast);
        assert_eq!(ast.pages[0].children[0].styles.overflow, OverflowValue::Hidden);
    }

    #[test]
    fn test_inject_defaults_list_gets_scroll_y() {
        let mut ast = AST::default();
        let mut page = Page { name: "P".to_string(), route: "/".to_string(), ..Default::default() };
        page.children.push(ComponentNode { kind: "list".to_string(), ..Default::default() });
        ast.pages.push(page);
        inject_overflow_defaults(&mut ast);
        assert_eq!(ast.pages[0].children[0].styles.overflow, OverflowValue::ScrollY);
    }

    #[test]
    fn test_inject_defaults_text_stays_visible() {
        let mut ast = AST::default();
        let mut page = Page { name: "P".to_string(), route: "/".to_string(), ..Default::default() };
        page.children.push(ComponentNode { kind: "text".to_string(), ..Default::default() });
        ast.pages.push(page);
        inject_overflow_defaults(&mut ast);
        assert_eq!(ast.pages[0].children[0].styles.overflow, OverflowValue::Visible);
    }

    #[test]
    fn test_inject_defaults_list_hidden_emits_warning_and_upgrades() {
        let mut ast = AST::default();
        let mut page = Page { name: "P".to_string(), route: "/".to_string(), ..Default::default() };
        let mut node = ComponentNode { kind: "list".to_string(), ..Default::default() };
        node.styles.overflow = OverflowValue::Hidden;
        page.children.push(node);
        ast.pages.push(page);
        let warnings = inject_overflow_defaults(&mut ast);
        assert!(!warnings.is_empty(), "expected warning for list+hidden");
        assert!(warnings[0].message.contains("overflow: hidden has no effect on list:"));
        assert_eq!(ast.pages[0].children[0].styles.overflow, OverflowValue::ScrollY);
    }

    // ── Property 15: text truncation completeness ──────────────────────────

    #[test]
    fn test_text_overflow_ellipsis_android() {
        assert_eq!(android_text_overflow(&TextOverflowValue::Ellipsis), "TextOverflow.Ellipsis");
    }

    #[test]
    fn test_text_overflow_ellipsis_ios() {
        assert_eq!(ios_line_break_mode(&TextOverflowValue::Ellipsis), ".byTruncatingTail");
    }

    // ── Image fit mappings ────────────────────────────────────────────────

    #[test]
    fn test_image_fit_cover_android() {
        assert_eq!(android_image_content_scale(&ImageFitValue::Cover), "ContentScale.Crop");
    }

    #[test]
    fn test_image_fit_cover_ios() {
        assert_eq!(ios_image_content_mode(&ImageFitValue::Cover), "scaleAspectFill");
    }

    #[test]
    fn test_image_fit_contain_android() {
        assert_eq!(android_image_content_scale(&ImageFitValue::Contain), "ContentScale.Fit");
    }

    #[test]
    fn test_image_fit_contain_ios() {
        assert_eq!(ios_image_content_mode(&ImageFitValue::Contain), "scaleAspectFit");
    }

    // ── Scroll snap ───────────────────────────────────────────────────────

    #[test]
    fn test_scroll_snap_center_android() {
        let code = android_scroll_snap_code(&ScrollSnap::Center, "rv");
        assert!(code.contains("PagerSnapHelper"), "got: {code}");
    }

    #[test]
    fn test_scroll_snap_start_android() {
        let code = android_scroll_snap_code(&ScrollSnap::Start, "rv");
        assert!(code.contains("LinearSnapHelper"), "got: {code}");
    }

    #[test]
    fn test_scroll_snap_center_ios() {
        let code = ios_scroll_snap_code(&ScrollSnap::Center, "sv");
        assert!(code.contains("isPagingEnabled = true"), "got: {code}");
    }

    #[test]
    fn test_scroll_snap_none_both_empty() {
        assert!(android_scroll_snap_code(&ScrollSnap::None_, "rv").is_empty());
        assert!(ios_scroll_snap_code(&ScrollSnap::None_, "sv").is_empty());
    }

    // ── default_overflow function ─────────────────────────────────────────

    #[test]
    fn test_default_overflow_container_hidden() {
        assert_eq!(default_overflow("container"), OverflowValue::Hidden);
        assert_eq!(default_overflow("row"), OverflowValue::Hidden);
        assert_eq!(default_overflow("column"), OverflowValue::Hidden);
        assert_eq!(default_overflow("card"), OverflowValue::Hidden);
    }

    #[test]
    fn test_default_overflow_list_scroll() {
        assert_eq!(default_overflow("list"), OverflowValue::ScrollY);
        assert_eq!(default_overflow("scroll_view"), OverflowValue::ScrollY);
    }

    #[test]
    fn test_default_overflow_leaf_visible() {
        assert_eq!(default_overflow("text"), OverflowValue::Visible);
        assert_eq!(default_overflow("image"), OverflowValue::Visible);
        assert_eq!(default_overflow("button"), OverflowValue::Visible);
    }
}
