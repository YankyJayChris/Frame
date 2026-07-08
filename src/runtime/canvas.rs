//! Canvas abstraction for Frame runtime rendering.
//!
//! Full wgpu/winit/softbuffer implementation will be wired up in Task 9+.
//! This stub provides the types that the rest of the codebase depends on
//! so that `cargo build` succeeds at the foundation stage.

pub struct Canvas {
    pub width: u32,
    pub height: u32,
}

impl Canvas {
    pub fn new(_width: u32, _height: u32) -> Self {
        Canvas { width: _width, height: _height }
    }

    pub fn draw_text(&mut self, _text: &str, _x: i32, _y: i32, _color: u32) {}
    pub fn draw_rect(&mut self, _x: i32, _y: i32, _w: i32, _h: i32, _color: u32) {}
    pub fn draw_image(&mut self, _src: &str, _x: i32, _y: i32) {}
    pub fn draw_svg(&mut self, _src: &str, _x: i32, _y: i32) {}
    pub fn present(&mut self) {}
    pub fn handle_touch(&self, _x: f64, _y: f64) -> Option<(i32, i32)> {
        None
    }
}

pub trait CanvasApp {
    fn render(&mut self, canvas: &mut Canvas);
    fn on_touch(&mut self, x: i32, y: i32);
}

/// Run a canvas application (no-op stub — real implementation in Task 9+).
pub fn run_app<T: CanvasApp + 'static>(_app: T) {
    // Full event loop with winit will be implemented in Task 9.
}
