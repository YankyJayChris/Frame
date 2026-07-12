//! Style types for the Frame framework.
//!
//! Re-exports the canonical `Styles` type from `parser::ast` so all code
//! uses one consistent, fully-typed definition.  The old `HashMap`-based
//! wrapper is retained as `LegacyStyles` for any callers that still need it,
//! but all new code should use `Styles` (= `parser::ast::Styles`).

// Re-export the canonical AST Styles type.
pub use crate::parser::ast::Styles;

use std::collections::HashMap;

/// Legacy `HashMap<String, String>` style map — kept for backward compatibility.
/// Prefer the canonical `Styles` struct for all new code.
#[derive(Debug, Clone, Default)]
pub struct LegacyStyles {
    pub props: HashMap<String, String>,
}

impl LegacyStyles {
    pub fn new() -> Self {
        LegacyStyles { props: HashMap::new() }
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        self.props.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.props.get(key)
    }

    pub fn get_px(&self, key: &str, default: i32) -> i32 {
        self.props
            .get(key)
            .and_then(|v| {
                v.trim_end_matches(|c: char| !c.is_ascii_digit())
                    .parse::<i32>()
                    .ok()
            })
            .unwrap_or(default)
    }

    pub fn with_defaults() -> Self {
        let mut s = Self::new();
        s.insert("padding", "10px");
        s.insert("margin", "5px");
        s.insert("font-size", "16px");
        s.insert("color", "#FFFFFF");
        s
    }
}
