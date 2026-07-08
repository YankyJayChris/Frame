//! Style types for the Frame framework.
//!
//! Full responsive style resolution: Task 15.

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Styles {
    pub props: HashMap<String, String>,
}

impl Styles {
    pub fn new() -> Self {
        Styles { props: HashMap::new() }
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
