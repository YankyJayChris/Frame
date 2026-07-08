//! Responsive layout engine for the Frame framework.
//!
//! Resolves breakpoint-based styles and screen utility values.
//! Used by both the code generators and the canvas renderer.

use crate::parser::ast::{Breakpoint, ScreenContext, Styles, OverflowValue};
use std::collections::HashMap;

// ─── Default breakpoints ─────────────────────────────────────────────────────

pub const DEFAULT_BREAKPOINTS: &[(&str, f32)] = &[
    ("sm", 360.0),
    ("md", 600.0),
    ("lg", 900.0),
    ("xl", 1200.0),
];

// ─── ResponsiveEngine ────────────────────────────────────────────────────────

/// Resolves breakpoint-aware styles for a given screen context.
#[derive(Debug, Clone, Default)]
pub struct ResponsiveEngine {
    /// Named breakpoints sorted by threshold ascending.
    pub breakpoints: Vec<Breakpoint>,
}

impl ResponsiveEngine {
    /// Create a new engine with the project's declared breakpoints.
    /// Falls back to defaults if `breakpoints` is empty.
    pub fn new(breakpoints: Vec<Breakpoint>) -> Self {
        let mut bps = if breakpoints.is_empty() {
            DEFAULT_BREAKPOINTS.iter().map(|(name, width)| Breakpoint {
                name: name.to_string(),
                min_width_dp: *width,
            }).collect()
        } else {
            breakpoints
        };
        // Sort ascending by threshold
        bps.sort_by(|a, b| a.min_width_dp.partial_cmp(&b.min_width_dp).unwrap());
        ResponsiveEngine { breakpoints: bps }
    }

    /// Returns the name of the active breakpoint for a given screen width.
    ///
    /// Algorithm: largest threshold ≤ width wins (monotonic).
    /// If no threshold matches, returns "sm" (or the first breakpoint name).
    pub fn active_breakpoint(&self, width_dp: f32) -> String {
        let mut active = self.breakpoints.first()
            .map(|bp| bp.name.clone())
            .unwrap_or_else(|| "sm".to_string());

        for bp in &self.breakpoints {
            if width_dp >= bp.min_width_dp {
                active = bp.name.clone();
            } else {
                break; // sorted ascending — no need to check further
            }
        }
        active
    }

    /// Build a ScreenContext for the given width/height/orientation.
    pub fn screen_context(&self, width_dp: f32, height_dp: f32, orientation: &str) -> ScreenContext {
        let breakpoint = self.active_breakpoint(width_dp);
        ScreenContext {
            width_dp,
            height_dp,
            breakpoint: breakpoint.clone(),
            is_phone:   width_dp < 600.0,
            is_tablet:  width_dp >= 600.0 && width_dp < 900.0,
            is_large:   width_dp >= 900.0,
            orientation: orientation.to_string(),
        }
    }

    /// Resolve styles for the active screen: apply base styles then overlay breakpoint overrides
    /// in ascending cascade order (sm → md → lg → xl).
    pub fn resolve(&self, base: &Styles, screen: &ScreenContext) -> Styles {
        let mut resolved = base.clone();

        for bp in &self.breakpoints {
            // Only apply overrides whose threshold ≤ current width
            if screen.width_dp >= bp.min_width_dp {
                if let Some(override_styles) = base.breakpoint_overrides.get(&bp.name) {
                    apply_styles_override(&mut resolved, override_styles);
                }
            }
        }
        resolved
    }

    /// Evaluate a `show_if` expression value for the given screen context.
    /// Accepts expressions like `screen.is_phone`, `screen.breakpoint == "md"`.
    /// For simple bool literals, pass `Some(true)` / `Some(false)` directly.
    pub fn evaluate_show_if(&self, expr_str: &str, screen: &ScreenContext) -> bool {
        match expr_str.trim() {
            "true"  => true,
            "false" => false,
            "screen.is_phone"   => screen.is_phone,
            "screen.is_tablet"  => screen.is_tablet,
            "screen.is_large"   => screen.is_large,
            s if s.starts_with("screen.breakpoint ==") => {
                let rhs = s.split("==").nth(1).unwrap_or("").trim().trim_matches('"');
                screen.breakpoint == rhs
            }
            _ => true, // default: show
        }
    }

    /// Expand a responsive array `[base, @md: v, @lg: v]` into a per-breakpoint map.
    /// Returns `HashMap<breakpoint_name, value_string>` plus `"base"` → base value.
    pub fn expand_responsive_array(items: &[String]) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for (i, item) in items.iter().enumerate() {
            let trimmed = item.trim();
            if i == 0 {
                map.insert("base".to_string(), trimmed.to_string());
            } else if let Some(rest) = trimmed.strip_prefix('@') {
                // "@md: 75%" → ("md", "75%")
                if let Some(colon) = rest.find(':') {
                    let bp_name = rest[..colon].trim().to_string();
                    let val     = rest[colon+1..].trim().to_string();
                    map.insert(bp_name, val);
                }
            } else {
                map.insert(format!("item_{}", i), trimmed.to_string());
            }
        }
        map
    }
}

/// Apply non-default fields from `overrides` onto `target`.
fn apply_styles_override(target: &mut Styles, overrides: &Styles) {
    if overrides.width.is_some()        { target.width        = overrides.width.clone(); }
    if overrides.height.is_some()       { target.height       = overrides.height.clone(); }
    if overrides.min_width.is_some()    { target.min_width    = overrides.min_width.clone(); }
    if overrides.max_width.is_some()    { target.max_width    = overrides.max_width.clone(); }
    if overrides.min_height.is_some()   { target.min_height   = overrides.min_height.clone(); }
    if overrides.max_height.is_some()   { target.max_height   = overrides.max_height.clone(); }
    if overrides.background.is_some()   { target.background   = overrides.background.clone(); }
    if overrides.color.is_some()        { target.color        = overrides.color.clone(); }
    if overrides.font_size.is_some()    { target.font_size    = overrides.font_size.clone(); }
    if overrides.font_weight.is_some()  { target.font_weight  = overrides.font_weight.clone(); }
    if overrides.padding.is_some()      { target.padding      = overrides.padding.clone(); }
    if overrides.margin.is_some()       { target.margin       = overrides.margin.clone(); }
    if overrides.gap.is_some()          { target.gap          = overrides.gap.clone(); }
    if overrides.direction.is_some()    { target.direction    = overrides.direction.clone(); }
    if overrides.align.is_some()        { target.align        = overrides.align.clone(); }
    if overrides.justify.is_some()      { target.justify      = overrides.justify.clone(); }
    if overrides.opacity.is_some()      { target.opacity      = overrides.opacity.clone(); }
    if overrides.border_radius.is_some() { target.border_radius = overrides.border_radius.clone(); }
    if !matches!(overrides.overflow, OverflowValue::Visible) {
        target.overflow = overrides.overflow.clone();
    }
    // Merge extra props
    for (k, v) in &overrides.extra {
        target.extra.insert(k.clone(), v.clone());
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine() -> ResponsiveEngine {
        ResponsiveEngine::new(vec![])
    }

    // ── active_breakpoint: monotonic invariant ─────────────────────────────

    #[test]
    fn test_breakpoint_monotonic_320dp() {
        // Below sm threshold → still returns "sm" (first bp)
        assert_eq!(make_engine().active_breakpoint(320.0), "sm");
    }

    #[test]
    fn test_breakpoint_monotonic_360dp() {
        assert_eq!(make_engine().active_breakpoint(360.0), "sm");
    }

    #[test]
    fn test_breakpoint_monotonic_600dp() {
        assert_eq!(make_engine().active_breakpoint(600.0), "md");
    }

    #[test]
    fn test_breakpoint_monotonic_768dp() {
        assert_eq!(make_engine().active_breakpoint(768.0), "md");
    }

    #[test]
    fn test_breakpoint_monotonic_900dp() {
        assert_eq!(make_engine().active_breakpoint(900.0), "lg");
    }

    #[test]
    fn test_breakpoint_monotonic_1200dp() {
        assert_eq!(make_engine().active_breakpoint(1200.0), "xl");
    }

    #[test]
    fn test_breakpoint_monotonic_largest_wins() {
        // Invariant: largest threshold ≤ width always wins
        let engine = make_engine();
        for width in [361.0f32, 599.0, 601.0, 899.0, 901.0, 1199.0, 1201.0] {
            let bp = engine.active_breakpoint(width);
            // Verify no breakpoint with higher threshold is active
            let active_threshold = engine.breakpoints.iter()
                .find(|b| b.name == bp).map(|b| b.min_width_dp).unwrap_or(0.0);
            assert!(active_threshold <= width,
                "active bp {bp} has threshold {active_threshold} > width {width}");
        }
    }

    // ── resolve: breakpoint cascade ──────────────────────────────────────

    #[test]
    fn test_resolve_applies_md_override_at_600dp() {
        let engine = make_engine();
        let screen = engine.screen_context(600.0, 800.0, "portrait");

        let mut styles = Styles::default();
        styles.width = Some("100%".to_string());

        let mut md_override = Styles::default();
        md_override.width = Some("75%".to_string());
        styles.breakpoint_overrides.insert("md".to_string(), Box::new(md_override));

        let resolved = engine.resolve(&styles, &screen);
        assert_eq!(resolved.width.as_deref(), Some("75%"),
            "md override should apply at 600dp");
    }

    #[test]
    fn test_resolve_does_not_apply_lg_override_at_600dp() {
        let engine = make_engine();
        let screen = engine.screen_context(600.0, 800.0, "portrait");

        let mut styles = Styles::default();
        styles.width = Some("100%".to_string());

        let mut lg_override = Styles::default();
        lg_override.width = Some("50%".to_string());
        styles.breakpoint_overrides.insert("lg".to_string(), Box::new(lg_override));

        let resolved = engine.resolve(&styles, &screen);
        assert_eq!(resolved.width.as_deref(), Some("100%"),
            "lg override should NOT apply at 600dp (threshold 900)");
    }

    #[test]
    fn test_resolve_applies_cascade_at_900dp() {
        // At 900dp both md and lg should apply; lg wins (applied last)
        let engine = make_engine();
        let screen = engine.screen_context(900.0, 1200.0, "landscape");

        let mut styles = Styles::default();
        styles.width = Some("100%".to_string());

        let mut md_override = Styles::default();
        md_override.width = Some("75%".to_string());
        styles.breakpoint_overrides.insert("md".to_string(), Box::new(md_override));

        let mut lg_override = Styles::default();
        lg_override.width = Some("50%".to_string());
        styles.breakpoint_overrides.insert("lg".to_string(), Box::new(lg_override));

        let resolved = engine.resolve(&styles, &screen);
        assert_eq!(resolved.width.as_deref(), Some("50%"),
            "lg override should win at 900dp (cascade)");
    }

    // ── screen_context ────────────────────────────────────────────────────

    #[test]
    fn test_screen_context_is_phone() {
        let engine = make_engine();
        let ctx = engine.screen_context(390.0, 844.0, "portrait");
        assert!(ctx.is_phone);
        assert!(!ctx.is_tablet);
        assert!(!ctx.is_large);
        assert_eq!(ctx.orientation, "portrait");
    }

    #[test]
    fn test_screen_context_is_tablet() {
        let engine = make_engine();
        let ctx = engine.screen_context(768.0, 1024.0, "landscape");
        assert!(!ctx.is_phone);
        assert!(ctx.is_tablet);
        assert!(!ctx.is_large);
        assert_eq!(ctx.orientation, "landscape");
    }

    #[test]
    fn test_screen_context_is_large() {
        let engine = make_engine();
        let ctx = engine.screen_context(1024.0, 768.0, "landscape");
        assert!(!ctx.is_phone);
        assert!(!ctx.is_tablet);
        assert!(ctx.is_large);
    }

    // ── evaluate_show_if ─────────────────────────────────────────────────

    #[test]
    fn test_show_if_true_literal() {
        let engine = make_engine();
        let ctx = engine.screen_context(390.0, 844.0, "portrait");
        assert!(engine.evaluate_show_if("true", &ctx));
    }

    #[test]
    fn test_show_if_false_literal() {
        let engine = make_engine();
        let ctx = engine.screen_context(390.0, 844.0, "portrait");
        assert!(!engine.evaluate_show_if("false", &ctx));
    }

    #[test]
    fn test_show_if_screen_is_phone() {
        let engine = make_engine();
        let ctx = engine.screen_context(390.0, 844.0, "portrait");
        assert!(engine.evaluate_show_if("screen.is_phone", &ctx));
    }

    #[test]
    fn test_show_if_screen_is_phone_false_on_tablet() {
        let engine = make_engine();
        let ctx = engine.screen_context(768.0, 1024.0, "landscape");
        assert!(!engine.evaluate_show_if("screen.is_phone", &ctx));
    }

    #[test]
    fn test_show_if_breakpoint_eq_md() {
        let engine = make_engine();
        let ctx = engine.screen_context(600.0, 800.0, "portrait");
        assert!(engine.evaluate_show_if(r#"screen.breakpoint == "md""#, &ctx));
    }

    // ── expand_responsive_array ──────────────────────────────────────────

    #[test]
    fn test_expand_responsive_array_base_and_overrides() {
        let items = vec![
            "100%".to_string(),
            "@md: 75%".to_string(),
            "@lg: 50%".to_string(),
        ];
        let map = ResponsiveEngine::expand_responsive_array(&items);
        assert_eq!(map.get("base"), Some(&"100%".to_string()));
        assert_eq!(map.get("md"),   Some(&"75%".to_string()));
        assert_eq!(map.get("lg"),   Some(&"50%".to_string()));
    }

    #[test]
    fn test_expand_responsive_array_base_only() {
        let items = vec!["100%".to_string()];
        let map = ResponsiveEngine::expand_responsive_array(&items);
        assert_eq!(map.get("base"), Some(&"100%".to_string()));
        assert_eq!(map.len(), 1);
    }

    // ── Custom breakpoints ────────────────────────────────────────────────

    #[test]
    fn test_custom_breakpoints_used_when_provided() {
        use crate::parser::ast::Breakpoint;
        let engine = ResponsiveEngine::new(vec![
            Breakpoint { name: "mobile".to_string(), min_width_dp: 0.0 },
            Breakpoint { name: "desktop".to_string(), min_width_dp: 1024.0 },
        ]);
        assert_eq!(engine.active_breakpoint(800.0), "mobile");
        assert_eq!(engine.active_breakpoint(1024.0), "desktop");
    }
}
