//! Frame runtime navigation engine.
//!
//! Supports:
//! - Push / pop navigation stack
//! - `navigate_replace` — replace current entry
//! - `navigate_back_to(route)` — pop to a named route
//! - Modal overlay slot (navigate_modal / navigate_dismiss)
//! - Tab navigation with independent per-tab stacks
//! - Route params (path segments and query string)
//! - Transition hints per navigation call

use std::collections::{HashMap, VecDeque};
use crate::parser::Page;

// ─── NavEntry ────────────────────────────────────────────────────────────────

/// A single entry in a navigation stack.
#[derive(Debug, Clone)]
pub struct NavEntry {
    pub route: String,
    pub params: HashMap<String, String>,
    pub transition: Option<String>,
}

impl NavEntry {
    fn new(route: impl Into<String>) -> Self {
        NavEntry { route: route.into(), params: HashMap::new(), transition: None }
    }

    fn with_params(mut self, params: HashMap<String, String>) -> Self {
        self.params = params;
        self
    }

    fn with_transition(mut self, t: Option<String>) -> Self {
        self.transition = t;
        self
    }
}

// ─── NavStack ────────────────────────────────────────────────────────────────

/// An independent navigation stack (used per-tab and for the root stack).
#[derive(Debug, Clone, Default)]
pub struct NavStack {
    entries: VecDeque<NavEntry>,
}

impl NavStack {
    pub fn new(root: impl Into<String>) -> Self {
        let mut s = NavStack::default();
        s.entries.push_back(NavEntry::new(root));
        s
    }

    /// Push a new route, optionally with params and transition.
    pub fn push(&mut self, route: impl Into<String>, params: HashMap<String, String>, transition: Option<String>) {
        let entry = NavEntry::new(route).with_params(params).with_transition(transition);
        self.entries.push_back(entry);
    }

    /// Replace the current top entry.
    pub fn replace(&mut self, route: impl Into<String>, params: HashMap<String, String>) {
        self.entries.pop_back();
        self.entries.push_back(NavEntry::new(route).with_params(params));
    }

    /// Pop one entry. Returns the popped route, or None if at root.
    pub fn pop(&mut self) -> Option<String> {
        if self.entries.len() > 1 {
            self.entries.pop_back().map(|e| e.route)
        } else {
            None
        }
    }

    /// Pop all entries back to (and including or excluding) a named route.
    /// Returns true if the target was found.
    pub fn pop_to(&mut self, target_route: &str, inclusive: bool) -> bool {
        let pos = self.entries.iter().rposition(|e| e.route == target_route);
        if let Some(idx) = pos {
            let keep = if inclusive { idx } else { idx + 1 };
            self.entries.truncate(keep);
            true
        } else {
            false
        }
    }

    /// Clear to root (keep only first entry).
    pub fn clear_to_root(&mut self) {
        self.entries.truncate(1);
    }

    /// Current top entry.
    pub fn current(&self) -> Option<&NavEntry> {
        self.entries.back()
    }

    /// Current route string.
    pub fn current_route(&self) -> &str {
        self.entries.back().map(|e| e.route.as_str()).unwrap_or("/")
    }

    /// Depth of the stack.
    pub fn depth(&self) -> usize { self.entries.len() }

    /// Can pop (not at root).
    pub fn can_pop(&self) -> bool { self.entries.len() > 1 }
}

// ─── Navigation ──────────────────────────────────────────────────────────────

pub struct Navigation {
    /// Root stack (used when no tabs are active).
    root_stack: NavStack,
    /// Per-tab stacks. Key = tab name/index string.
    tab_stacks: HashMap<String, NavStack>,
    /// Currently active tab, or None if not in tab mode.
    active_tab: Option<String>,
    /// Modal route (presented over the current stack).
    modal: Option<NavEntry>,
    /// All registered pages (for `current_page()` lookup).
    pages: Vec<Page>,
}

impl Navigation {
    /// Create a Navigation instance starting at `home_route`.
    pub fn new(home_route: impl Into<String>) -> Self {
        Navigation {
            root_stack: NavStack::new(home_route),
            tab_stacks: HashMap::new(),
            active_tab: None,
            modal: None,
            pages: Vec::new(),
        }
    }

    /// Convenience: start at `/home`.
    pub fn default_home() -> Self {
        Self::new("/home")
    }

    pub fn set_pages(&mut self, pages: Vec<Page>) {
        self.pages = pages;
    }

    // ── Tab management ────────────────────────────────────────────────────────

    /// Register a tab with its root route.
    pub fn register_tab(&mut self, tab: impl Into<String>, root_route: impl Into<String>) {
        let tab = tab.into();
        self.tab_stacks.entry(tab).or_insert_with(|| NavStack::new(root_route));
    }

    /// Switch to a named tab. Creates its stack with `root_route` if not registered.
    pub fn switch_tab(&mut self, tab: impl Into<String>, root_route: impl Into<String>) {
        let tab = tab.into();
        self.tab_stacks.entry(tab.clone()).or_insert_with(|| NavStack::new(root_route));
        self.active_tab = Some(tab);
    }

    /// Active tab name, if in tab mode.
    pub fn active_tab(&self) -> Option<&str> {
        self.active_tab.as_deref()
    }

    // ── Core navigation ───────────────────────────────────────────────────────

    fn active_stack(&self) -> &NavStack {
        if let Some(tab) = &self.active_tab {
            self.tab_stacks.get(tab.as_str()).unwrap_or(&self.root_stack)
        } else {
            &self.root_stack
        }
    }

    fn active_stack_mut(&mut self) -> &mut NavStack {
        if let Some(tab) = self.active_tab.clone() {
            self.tab_stacks.entry(tab).or_insert_with(|| NavStack::new("/"))
        } else {
            &mut self.root_stack
        }
    }

    /// Push a route. Params come from route query string (`?k=v`) or passed explicitly.
    pub fn navigate(&mut self, route: &str, transition: Option<&str>) {
        let (base, params) = parse_route(route);
        self.active_stack_mut().push(base, params, transition.map(str::to_string));
    }

    /// Replace current entry (no new back-stack entry).
    pub fn navigate_replace(&mut self, route: &str) {
        let (base, params) = parse_route(route);
        self.active_stack_mut().replace(base, params);
    }

    /// Pop one entry. Returns the popped route.
    pub fn navigate_back(&mut self) -> Option<String> {
        self.active_stack_mut().pop()
    }

    /// Pop to a specific route, keeping it as the top.
    pub fn navigate_back_to(&mut self, route: &str) -> bool {
        self.active_stack_mut().pop_to(route, false)
    }

    /// Pop everything and navigate to root.
    pub fn navigate_clear_stack(&mut self, route: &str) {
        let stack = self.active_stack_mut();
        stack.clear_to_root();
        let (base, params) = parse_route(route);
        stack.push(base, params, None);
    }

    // ── Modal ─────────────────────────────────────────────────────────────────

    /// Present a route modally (over the current stack).
    pub fn navigate_modal(&mut self, route: &str) {
        let (base, params) = parse_route(route);
        self.modal = Some(NavEntry::new(base).with_params(params));
    }

    /// Dismiss the current modal.
    pub fn navigate_dismiss(&mut self) {
        self.modal = None;
    }

    /// Whether a modal is currently presented.
    pub fn has_modal(&self) -> bool {
        self.modal.is_some()
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    /// Current visible route (modal takes precedence over stack).
    pub fn current_route(&self) -> &str {
        self.modal.as_ref()
            .map(|m| m.route.as_str())
            .unwrap_or_else(|| self.active_stack().current_route())
    }

    /// Current active `Page` definition, if registered.
    pub fn current_page(&self) -> Option<&Page> {
        let route = self.current_route();
        self.pages.iter().find(|p| p.route == route)
    }

    /// Get a route param from the current entry.
    pub fn get_param(&self, key: &str) -> Option<&str> {
        let entry = self.modal.as_ref()
            .or_else(|| self.active_stack().current());
        entry.and_then(|e| e.params.get(key)).map(|s| s.as_str())
    }

    /// Whether the current stack can go back.
    pub fn can_go_back(&self) -> bool {
        self.modal.is_some() || self.active_stack().can_pop()
    }

    /// Depth of the active stack.
    pub fn stack_depth(&self) -> usize {
        self.active_stack().depth()
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Split `/profile/42?tab=posts` into (`/profile/42`, {tab: "posts"}).
fn parse_route(route: &str) -> (String, HashMap<String, String>) {
    let mut params = HashMap::new();
    if let Some(q) = route.find('?') {
        let base  = route[..q].to_string();
        let query = &route[q + 1..];
        for pair in query.split('&') {
            if let Some(eq) = pair.find('=') {
                params.insert(pair[..eq].to_string(), pair[eq + 1..].to_string());
            }
        }
        (base, params)
    } else {
        (route.to_string(), params)
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn nav() -> Navigation { Navigation::default_home() }

    #[test]
    fn test_push_and_current_route() {
        let mut n = nav();
        n.navigate("/profile", None);
        assert_eq!(n.current_route(), "/profile");
    }

    #[test]
    fn test_pop_returns_previous() {
        let mut n = nav();
        n.navigate("/profile", None);
        n.navigate_back();
        assert_eq!(n.current_route(), "/home");
    }

    #[test]
    fn test_pop_at_root_returns_none() {
        let mut n = nav();
        assert!(n.navigate_back().is_none());
    }

    #[test]
    fn test_navigate_replace_no_back() {
        let mut n = nav();
        n.navigate_replace("/login");
        assert_eq!(n.current_route(), "/login");
        assert!(!n.can_go_back(), "replace should not grow the stack");
    }

    #[test]
    fn test_navigate_back_to() {
        let mut n = nav();
        n.navigate("/a", None);
        n.navigate("/b", None);
        n.navigate("/c", None);
        assert!(n.navigate_back_to("/a"));
        assert_eq!(n.current_route(), "/a");
    }

    #[test]
    fn test_navigate_back_to_missing_returns_false() {
        let mut n = nav();
        n.navigate("/a", None);
        assert!(!n.navigate_back_to("/nonexistent"));
    }

    #[test]
    fn test_modal_over_stack() {
        let mut n = nav();
        n.navigate_modal("/sheet");
        assert_eq!(n.current_route(), "/sheet");
        assert!(n.has_modal());
        n.navigate_dismiss();
        assert_eq!(n.current_route(), "/home");
        assert!(!n.has_modal());
    }

    #[test]
    fn test_query_params_parsed() {
        let mut n = nav();
        n.navigate("/profile?userId=42", None);
        assert_eq!(n.get_param("userId"), Some("42"));
    }

    #[test]
    fn test_tab_stacks_independent() {
        let mut n = nav();
        n.switch_tab("home", "/home");
        n.navigate("/home/detail", None);
        n.switch_tab("search", "/search");
        // search tab should be at its root, not affected by home tab
        assert_eq!(n.current_route(), "/search");
        n.switch_tab("home", "/home");
        assert_eq!(n.current_route(), "/home/detail");
    }

    #[test]
    fn test_clear_stack() {
        let mut n = nav();
        n.navigate("/a", None);
        n.navigate("/b", None);
        n.navigate("/c", None);
        n.navigate_clear_stack("/root");
        assert_eq!(n.current_route(), "/root");
        assert_eq!(n.stack_depth(), 2); // root entry + new route
    }

    #[test]
    fn test_stack_depth() {
        let mut n = nav();
        assert_eq!(n.stack_depth(), 1);
        n.navigate("/a", None);
        n.navigate("/b", None);
        assert_eq!(n.stack_depth(), 3);
    }

    #[test]
    fn test_can_go_back_with_modal() {
        let mut n = nav();
        assert!(!n.can_go_back());
        n.navigate_modal("/sheet");
        assert!(n.can_go_back());
    }
}
