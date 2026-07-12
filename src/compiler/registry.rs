//! Built-in Component Registry for the Frame framework (plan §Phase 2).
//!
//! Replaces the flat `BUILTIN_COMPONENTS: &[&str]` list with a structured
//! registry that defines every component's props, allowed styles, children
//! rules, and events.

use std::collections::HashMap;
use std::sync::OnceLock;

// ─── Category ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentCategory {
    Layout,
    TextContent,
    Input,
    Navigation,
    Feedback,
    Media,
    Gesture,
}

// ─── PropDef ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BuiltInPropDef {
    pub name: &'static str,
    pub kind: &'static str,
    pub required: bool,
    pub default: Option<&'static str>,
}

impl BuiltInPropDef {
    fn req(name: &'static str, kind: &'static str) -> Self {
        BuiltInPropDef { name, kind, required: true, default: None }
    }
    fn opt(name: &'static str, kind: &'static str) -> Self {
        BuiltInPropDef { name, kind, required: false, default: None }
    }
    fn def(name: &'static str, kind: &'static str, default: &'static str) -> Self {
        BuiltInPropDef { name, kind, required: false, default: Some(default) }
    }
}

// ─── StyleProp ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StyleProp {
    pub name: &'static str,
}

fn sp(name: &'static str) -> StyleProp { StyleProp { name } }

// ─── BuiltInComponentDef ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BuiltInComponentDef {
    pub name: &'static str,
    pub props: Vec<BuiltInPropDef>,
    pub style_props: Vec<StyleProp>,
    pub allows_children: bool,
    pub allowed_children_kinds: Option<&'static [&'static str]>,
    pub events: Vec<&'static str>,
    pub category: ComponentCategory,
}

impl BuiltInComponentDef {
    pub fn has_prop(&self, name: &str) -> bool {
        self.props.iter().any(|p| p.name == name)
    }
    pub fn required_props(&self) -> Vec<&str> {
        self.props.iter().filter(|p| p.required).map(|p| p.name).collect()
    }
    pub fn allows_style(&self, name: &str) -> bool {
        if self.style_props.is_empty() { return true; }
        self.style_props.iter().any(|s| s.name == name)
    }
}

// ─── ComponentRegistry ───────────────────────────────────────────────────────

pub struct ComponentRegistry {
    pub components: HashMap<&'static str, BuiltInComponentDef>,
}

impl ComponentRegistry {
    pub fn get(&self, name: &str) -> Option<&BuiltInComponentDef> {
        self.components.get(name)
    }
    pub fn contains(&self, name: &str) -> bool {
        self.components.contains_key(name)
    }
    pub fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.components.keys().cloned().collect();
        names.sort();
        names
    }
}

// ─── Style sets ───────────────────────────────────────────────────────────────

fn layout_styles() -> Vec<StyleProp> {
    vec![
        sp("width"), sp("height"), sp("min_width"), sp("max_width"),
        sp("min_height"), sp("max_height"),
        sp("padding"), sp("padding_top"), sp("padding_bottom"),
        sp("padding_left"), sp("padding_right"),
        sp("margin"), sp("margin_top"), sp("margin_bottom"),
        sp("margin_left"), sp("margin_right"),
        sp("background"), sp("border_radius"), sp("border"),
        sp("opacity"), sp("overflow"), sp("flex"),
        sp("direction"), sp("align"), sp("justify"), sp("gap"),
    ]
}
fn text_styles() -> Vec<StyleProp> {
    vec![
        sp("color"), sp("font_size"), sp("font_weight"), sp("font_family"),
        sp("text_overflow"), sp("max_lines"), sp("line_clamp"),
        sp("width"), sp("height"), sp("margin"), sp("padding"), sp("opacity"),
    ]
}
fn icon_styles() -> Vec<StyleProp> {
    // plan §3a — icon: supports color, font_weight, width, height
    vec![sp("color"), sp("font_weight"), sp("width"), sp("height"), sp("opacity"), sp("margin")]
}
fn image_styles() -> Vec<StyleProp> {
    vec![sp("width"), sp("height"), sp("border_radius"), sp("opacity"),
         sp("margin"), sp("fit"), sp("clip_behavior")]
}
fn tiny_styles() -> Vec<StyleProp> { vec![sp("width"), sp("height")] }
fn line_styles() -> Vec<StyleProp> { vec![sp("color"), sp("margin")] }

// ─── Builder helper ───────────────────────────────────────────────────────────

fn def(
    name: &'static str,
    cat: ComponentCategory,
    props: Vec<BuiltInPropDef>,
    styles: Vec<StyleProp>,
    children: bool,
    only: Option<&'static [&'static str]>,
    events: Vec<&'static str>,
) -> BuiltInComponentDef {
    BuiltInComponentDef {
        name, props, style_props: styles, allows_children: children,
        allowed_children_kinds: only, events, category: cat,
    }
}

// ─── Build ────────────────────────────────────────────────────────────────────

pub fn build_registry() -> ComponentRegistry {
    use ComponentCategory::*;
    let mut m: HashMap<&'static str, BuiltInComponentDef> = HashMap::new();

    let mut add = |d: BuiltInComponentDef| { m.insert(d.name, d); };

    // ── Layout ────────────────────────────────────────────────────────────────
    add(def("row",      Layout,  vec![], layout_styles(), true,  None, vec!["on_click","on_scroll","on_scroll_end"]));
    add(def("column",   Layout,  vec![], layout_styles(), true,  None, vec!["on_click","on_scroll","on_scroll_end"]));
    add(def("container",Layout,  vec![], layout_styles(), true,  None, vec!["on_click"]));
    add(def("stack",    Layout,  vec![BuiltInPropDef::opt("alignment","string")], layout_styles(), true, None, vec!["on_click"]));
    add(def("scaffold", Layout,  vec![], layout_styles(), true,  None, vec![]));
    add(def("card",     Layout,  vec![], layout_styles(), true,  None, vec!["on_click"]));
    add(def("divider",  Layout,  vec![], line_styles(),   false, None, vec![]));
    add(def("spacer",   Layout,  vec![], tiny_styles(),   false, None, vec![]));
    add(def("scroll_view", Layout, vec![], layout_styles(), true, None, vec!["on_scroll","on_scroll_end"]));
    add(def("grid",     Layout,
        vec![BuiltInPropDef::opt("columns","int"), BuiltInPropDef::opt("data","expr")],
        layout_styles(), true, None, vec![]));
    add(def("list",     Layout,
        vec![BuiltInPropDef::opt("data","expr"), BuiltInPropDef::opt("build","expr")],
        layout_styles(), true, None, vec!["on_scroll","on_scroll_end"]));
    add(def("form",     Layout,
        vec![BuiltInPropDef::opt("schema","string")],
        layout_styles(), true, None, vec!["on_submit"]));
    add(def("item",     Layout,  vec![], layout_styles(), true,  None, vec![]));
    add(def("accordion",Layout,  vec![BuiltInPropDef::opt("title","string")], layout_styles(), true, None, vec![]));
    add(def("timeline", Layout,  vec![], layout_styles(), true,  None, vec![]));

    // ── Text / Content ────────────────────────────────────────────────────────
    add(def("text",    TextContent, vec![BuiltInPropDef::opt("content","string")], text_styles(), false, None, vec!["on_click"]));
    add(def("button",  TextContent, vec![BuiltInPropDef::opt("content","string")], layout_styles(), false, None, vec!["on_click"]));
    add(def("icon",    TextContent,
        vec![BuiltInPropDef::opt("name","string"), BuiltInPropDef::opt("path","string")],
        icon_styles(), false, None, vec!["on_click"]));
    add(def("image",   TextContent, vec![BuiltInPropDef::req("src","string")],   image_styles(), false, None, vec!["on_click"]));
    add(def("avatar",  TextContent, vec![BuiltInPropDef::req("src","string")],   image_styles(), false, None, vec!["on_click"]));
    add(def("badge",   TextContent, vec![BuiltInPropDef::opt("count","int")],    layout_styles(), true, None, vec![]));
    add(def("chip",    TextContent,
        vec![BuiltInPropDef::opt("content","string"), BuiltInPropDef::opt("label","string")],
        layout_styles(), false, None, vec!["on_click"]));
    add(def("tag",     TextContent,
        vec![BuiltInPropDef::opt("content","string"), BuiltInPropDef::opt("label","string")],
        layout_styles(), false, None, vec![]));
    add(def("banner",  TextContent, vec![], layout_styles(), true, None, vec!["on_click"]));
    add(def("skeleton",TextContent, vec![], layout_styles(), false, None, vec![]));
    add(def("table",   TextContent, vec![BuiltInPropDef::opt("data","expr")], layout_styles(), true, None, vec![]));

    // ── Input ─────────────────────────────────────────────────────────────────
    add(def("input",   Input,
        vec![BuiltInPropDef::opt("value","string"), BuiltInPropDef::opt("placeholder","string"),
             BuiltInPropDef::opt("validate","expr"), BuiltInPropDef::opt("on_error","string")],
        layout_styles(), false, None, vec!["on_change","on_submit","on_focus","on_blur"]));
    add(def("text_area", Input,
        vec![BuiltInPropDef::opt("value","string"), BuiltInPropDef::opt("placeholder","string"),
             BuiltInPropDef::opt("lines","int"), BuiltInPropDef::opt("validate","expr")],
        layout_styles(), false, None, vec!["on_change","on_submit"]));
    add(def("dropdown", Input,
        vec![BuiltInPropDef::opt("value","string"), BuiltInPropDef::opt("validate","expr")],
        layout_styles(), true, None, vec!["on_change","on_select"]));
    add(def("switch",   Input,
        vec![BuiltInPropDef::opt("value","bool"), BuiltInPropDef::opt("checked","bool")],
        layout_styles(), false, None, vec!["on_change"]));
    add(def("checkbox", Input,
        vec![BuiltInPropDef::opt("value","bool"), BuiltInPropDef::opt("checked","bool")],
        layout_styles(), false, None, vec!["on_change"]));
    add(def("radio",    Input, vec![BuiltInPropDef::opt("selected","bool")], layout_styles(), false, None, vec!["on_click"]));
    add(def("slider",   Input,
        vec![BuiltInPropDef::opt("value","float"), BuiltInPropDef::opt("min","float"), BuiltInPropDef::opt("max","float")],
        layout_styles(), false, None, vec!["on_change"]));
    add(def("stepper",  Input, vec![BuiltInPropDef::opt("value","int")], layout_styles(), false, None, vec!["on_increment","on_decrement"]));
    add(def("search_bar", Input,
        vec![BuiltInPropDef::opt("value","string"), BuiltInPropDef::opt("placeholder","string")],
        layout_styles(), false, None, vec!["on_change","on_submit"]));
    add(def("date_picker", Input,
        vec![BuiltInPropDef::opt("value","string"), BuiltInPropDef::opt("validate","expr")],
        layout_styles(), false, None, vec!["on_change"]));
    add(def("time_picker", Input,
        vec![BuiltInPropDef::opt("value","string"), BuiltInPropDef::opt("validate","expr")],
        layout_styles(), false, None, vec!["on_change"]));
    add(def("color_picker", Input, vec![BuiltInPropDef::opt("value","string")], layout_styles(), false, None, vec!["on_change"]));
    add(def("rating",   Input,
        vec![BuiltInPropDef::opt("value","int"), BuiltInPropDef::def("max","int","5")],
        layout_styles(), false, None, vec!["on_change"]));
    add(def("otp_input", Input,
        vec![BuiltInPropDef::def("length","int","6"), BuiltInPropDef::opt("validate","expr")],
        layout_styles(), false, None, vec!["on_complete"]));

    // ── Navigation ────────────────────────────────────────────────────────────
    add(def("app_bar",  Navigation,
        vec![BuiltInPropDef::opt("title","string"), BuiltInPropDef::opt("leading","string")],
        layout_styles(), true, None, vec![]));
    add(def("bottom_navigation_bar", Navigation, vec![], layout_styles(), true, None, vec![]));
    add(def("sidebar",  Navigation,
        vec![BuiltInPropDef::def("side","string","left"), BuiltInPropDef::def("width","string","260"),
             BuiltInPropDef::opt("collapsed","bool")],
        layout_styles(), true, None, vec![]));
    add(def("floating_action_button", Input,
        vec![BuiltInPropDef::opt("content","string"), BuiltInPropDef::opt("icon","string"),
             BuiltInPropDef::def("position","string","bottom_end")],
        layout_styles(), true, None, vec!["on_click"]));
    add(def("tab_bar",  Navigation,
        vec![BuiltInPropDef::opt("selected","int"), BuiltInPropDef::opt("current","int")],
        layout_styles(), true, Some(&["tab"]), vec![]));
    add(def("tab",      Navigation,
        vec![BuiltInPropDef::opt("content","string"), BuiltInPropDef::opt("icon","string")],
        layout_styles(), false, None, vec!["on_click","on_select"]));
    add(def("bottom_sheet", Navigation, vec![], layout_styles(), true, None, vec!["on_unmount"]));
    add(def("modal",    Navigation,
        vec![BuiltInPropDef::opt("title","string"), BuiltInPropDef::opt("message","string")],
        layout_styles(), true, None, vec!["on_unmount"]));

    // ── Feedback ──────────────────────────────────────────────────────────────
    add(def("toast",    Feedback,
        vec![BuiltInPropDef::opt("message","string"), BuiltInPropDef::opt("duration","int")],
        layout_styles(), false, None, vec![]));
    add(def("tooltip",  Feedback, vec![BuiltInPropDef::opt("text","string")], layout_styles(), true, None, vec![]));
    add(def("progress_bar",    Feedback, vec![BuiltInPropDef::opt("value","float")], layout_styles(), false, None, vec![]));
    add(def("progress_circle", Feedback, vec![BuiltInPropDef::opt("value","float")], layout_styles(), false, None, vec![]));

    // ── Media ─────────────────────────────────────────────────────────────────
    add(def("video_player", Media, vec![BuiltInPropDef::req("src","string")],  layout_styles(), false, None, vec!["on_complete"]));
    add(def("audio_player", Media, vec![BuiltInPropDef::req("src","string")],  layout_styles(), false, None, vec!["on_complete"]));
    add(def("lottie",       Media, vec![BuiltInPropDef::req("src","string")],  layout_styles(), false, None, vec!["on_complete"]));
    add(def("web_view",     Media, vec![BuiltInPropDef::req("url","string")],  layout_styles(), false, None, vec![]));
    add(def("map_view",     Media,
        vec![BuiltInPropDef::opt("lat","float"), BuiltInPropDef::opt("lng","float")],
        layout_styles(), false, None, vec![]));
    add(def("camera_view",  Media, vec![], layout_styles(), false, None, vec![]));
    add(def("qr_scanner",   Media, vec![], layout_styles(), false, None, vec!["on_scan"]));

    // ── Gesture ───────────────────────────────────────────────────────────────
    add(def("swipeable",  Gesture, vec![], layout_styles(), true, None, vec!["on_swipe"]));
    add(def("draggable",  Gesture, vec![], layout_styles(), true, None, vec!["on_drag"]));
    add(def("refresh",    Gesture, vec![BuiltInPropDef::opt("refreshing","bool")], layout_styles(), true, None, vec!["on_refresh"]));
    add(def("long_press", Gesture, vec![], layout_styles(), true, None, vec!["on_long_press"]));

    // ── Misc ──────────────────────────────────────────────────────────────────
    add(def("plugin", Layout,
        vec![BuiltInPropDef::req("name","string"), BuiltInPropDef::req("method","string")],
        vec![], false, None, vec![]));

    ComponentRegistry { components: m }
}

// ─── Lazy singleton ───────────────────────────────────────────────────────────

static REGISTRY: OnceLock<ComponentRegistry> = OnceLock::new();

pub fn registry() -> &'static ComponentRegistry {
    REGISTRY.get_or_init(build_registry)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_all_expected_components() {
        let reg = registry();
        for name in &["row","column","container","stack","scaffold","card",
                      "divider","spacer","scroll_view","grid","list",
                      "sidebar","floating_action_button",
            "input","text_area","dropdown","switch","checkbox","radio",
                      "slider","stepper","search_bar","date_picker","time_picker",
                      "color_picker","rating","otp_input",
                      "video_player","audio_player","lottie","web_view",
                      "map_view","camera_view","qr_scanner",
                      "swipeable","draggable","refresh","long_press"] {
            assert!(reg.contains(name), "missing: {name}");
        }
    }

    #[test]
    fn sidebar_has_default_props() {
        let reg = registry();
        let sb = reg.get("sidebar").unwrap();
        assert!(sb.has_prop("side"), "sidebar missing side prop");
        assert!(sb.has_prop("width"), "sidebar missing width prop");
        assert!(sb.allows_children);
    }

    #[test]
    fn fab_has_content_and_icon_props() {
        let reg = registry();
        let fab = reg.get("floating_action_button").unwrap();
        assert!(fab.has_prop("content"), "fab missing content prop");
        assert!(fab.has_prop("icon"), "fab missing icon prop");
        assert!(fab.has_prop("position"), "fab missing position prop");
        assert!(fab.events.contains(&"on_click"), "fab missing on_click event");
    }

    #[test]
    fn image_requires_src_prop() {
        let reg = registry();
        let img = reg.get("image").unwrap();
        assert!(img.required_props().contains(&"src"));
    }

    #[test]
    fn icon_has_name_and_path_props() {
        let reg = registry();
        let icon = reg.get("icon").unwrap();
        assert!(icon.has_prop("name"), "icon missing name prop");
        assert!(icon.has_prop("path"), "icon missing path prop (plan §3a)");
    }

    #[test]
    fn icon_allows_color_and_font_weight_styles() {
        let reg = registry();
        let icon = reg.get("icon").unwrap();
        assert!(icon.allows_style("color"),       "icon must allow color");
        assert!(icon.allows_style("font_weight"), "icon must allow font_weight");
        assert!(icon.allows_style("width"),       "icon must allow width");
        assert!(icon.allows_style("height"),      "icon must allow height");
    }

    #[test]
    fn tab_bar_only_allows_tab_children() {
        let reg = registry();
        let tab_bar = reg.get("tab_bar").unwrap();
        assert_eq!(tab_bar.allowed_children_kinds, Some(&["tab"][..]));
    }

    #[test]
    fn input_supports_validate_prop() {
        let reg = registry();
        let input = reg.get("input").unwrap();
        assert!(input.has_prop("validate"), "input must support validate prop");
    }

    #[test]
    fn otp_input_has_default_length_6() {
        let reg = registry();
        let otp = reg.get("otp_input").unwrap();
        let len = otp.props.iter().find(|p| p.name == "length").unwrap();
        assert_eq!(len.default, Some("6"));
    }

    #[test]
    fn total_component_count_at_least_55() {
        let reg = registry();
        assert!(reg.components.len() >= 57,
            "Registry has {} components, expected ≥57", reg.components.len());
    }
}
