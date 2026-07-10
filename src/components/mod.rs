//! Built-in Frame component implementations.
//!
//! Full set of 20 built-in components with rendering: Tasks 9–14.
//! This stub defines the Component trait and skeleton types so the
//! codebase compiles at the foundation stage.

use crate::runtime::Canvas;
use crate::styles::Styles;
use std::rc::Rc;
use std::cell::RefCell;
use crate::runtime::Animation;

pub trait Component {
    fn render(&self, canvas: &mut Canvas, styles: &Styles, animations: &mut Vec<Animation>);
    fn mount(&self) {}
    fn update(&self) {}
    fn unmount(&self) {}
    fn children(&self) -> Option<&Vec<Rc<RefCell<Box<dyn Component>>>>> { None }
    fn on_click(&self) -> Option<&String> { None }
    fn on_touch_start(&self) -> Option<&String> { None }
    fn on_touch_move(&self) -> Option<&String> { None }
    fn on_touch_end(&self) -> Option<&String> { None }
    fn on_touch_cancel(&self) -> Option<&String> { None }
    fn styles(&self) -> Styles;
}

/// Minimal AppBar stub.
pub struct AppBar {
    pub title: Option<String>,
    pub styles: Styles,
}

impl Component for AppBar {
    fn render(&self, _canvas: &mut Canvas, _styles: &Styles, _animations: &mut Vec<Animation>) {}
    fn styles(&self) -> Styles { self.styles.clone() }
}

/// Minimal Text stub.
pub struct Text {
    pub content: String,
    pub styles: Styles,
}

impl Component for Text {
    fn render(&self, _canvas: &mut Canvas, _styles: &Styles, _animations: &mut Vec<Animation>) {}
    fn styles(&self) -> Styles { self.styles.clone() }
}
