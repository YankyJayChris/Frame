mod canvas;
mod plugins;
mod state;
mod navigation;
mod animation;

pub use canvas::{Canvas, CanvasApp, run_app};
pub use plugins::*;
pub use state::{State, I18n, Reactive};
pub use navigation::Navigation;
pub use animation::Animation;

use std::collections::HashMap;
use rusqlite::Connection;
use tokio::sync::mpsc;

pub trait Component {
    fn render(&self, canvas: &mut Canvas, styles: &Styles, animations: &mut Vec<Animation>);
    fn mount(&self) {}
    fn update(&self) {}
    fn unmount(&self) {}
    fn children(&self) -> Option<&Vec<Rc<RefCell<Box<dyn Component>>>>> { None }
    fn on_click(&self) -> Option<&String> { None }
    fn styles(&self) -> Styles;
}

pub trait StyledComponent {
    fn styles(&self) -> Styles;
}

#[derive(Debug, Clone, Default)]
pub struct Styles {
    map: HashMap<String, String>,
}

impl Styles {
    pub fn new() -> Self { Styles { map: HashMap::new() } }
    pub fn insert(&mut self, key: &str, value: &str) { self.map.insert(key.to_string(), value.to_string()); }
    pub fn get(&self, key: &str) -> Option<&String> { self.map.get(key) }
    pub fn get_px(&self, key: &str, default: i32) -> i32 {
        self.get(key).and_then(|v| v.trim_end_matches("dp").parse().ok()).unwrap_or(default)
    }
    pub fn get_hex(&self, key: &str, default: u32) -> u32 {
        self.get(key).and_then(|v| u32::from_str_radix(v.trim_start_matches('#'), 16).ok()).unwrap_or(default)
    }
}

pub struct Animation {
    pub kind: String,
    pub duration: u32, // in milliseconds
    pub from: Styles,  // Restored
    pub to: Styles,    // Restored
}

impl Animation {
    pub fn new(kind: &str, duration: u32, from: Styles, to: Styles) -> Self {
        Animation { kind: kind.to_string(), duration, from, to }
    }
}

pub fn render_to_string(_components: &[Box<dyn Component>]) -> String {
    "<html>SSR Placeholder</html>".to_string()
}