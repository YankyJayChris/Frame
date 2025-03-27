use winit::event::{Event, WindowEvent, TouchPhase};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::WindowBuilder;
use softbuffer::GraphicsContext;
use image::{DynamicImage, Rgba};
use fontdue::{Font, TextStyle};
use std::path::Path;

pub struct Canvas {
    graphics: GraphicsContext,
    width: u32,
    height: u32,
    buffer: Vec<u32>,
    font: Font,
    touch_positions: Vec<(f64, f64)>,
}

impl Canvas {
    pub fn new(event_loop: &EventLoop<()>, width: u32, height: u32) -> Self {
        let window = WindowBuilder::new()
            .with_title("Frame App")
            .with_inner_size(winit::dpi::PhysicalSize::new(width, height))
            .build(event_loop)
            .unwrap();
        let graphics = unsafe { GraphicsContext::new(window).unwrap() };
        let buffer = vec![0xFF000000; (width * height) as usize]; // Black background

        // Load system font (e.g., default sans-serif)
        let font = fontdue::Font::from_system("Sans".to_string(), fontdue::FontSettings::default())
            .unwrap_or_else(|_| {
                // Fallback to a minimal built-in font if system font fails
                Font::from_bytes(include_bytes!("../../assets/fonts/DejaVuSans.ttf").as_ref(), fontdue::FontSettings::default()).unwrap()
            });

        Canvas {
            graphics,
            width,
            height,
            buffer,
            font,
            touch_positions: Vec::new(),
        }
    }

    pub fn draw_text(&mut self, text: &str, x: i32, y: i32, color: u32) {
        let style = TextStyle::new(text, 20.0, 0);
        let glyphs = self.font.rasterize_text(&style);
        for glyph in glyphs {
            for (dx, dy, coverage) in glyph.pixels {
                let px = x + dx as i32;
                let py = y + dy as i32;
                if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                    let idx = (py * self.width as i32 + px) as usize;
                    if idx < self.buffer.len() {
                        self.buffer[idx] = blend(self.buffer[idx], color, coverage);
                    }
                }
            }
        }
    }

    pub fn draw_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u32) {
        for py in y..y + h {
            for px in x..x + w {
                if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                    let idx = (py * self.width as i32 + px) as usize;
                    if idx < self.buffer.len() {
                        self.buffer[idx] = color;
                    }
                }
            }
        }
    }

    pub fn draw_image(&mut self, src: &str, x: i32, y: i32) {
        let img_path = format!("assets/{}", src);
        if Path::new(&img_path).exists() {
            let img = image::open(&img_path).unwrap().into_rgba8();
            for (ix, iy, pixel) in img.enumerate_pixels() {
                let px = x + ix as i32;
                let py = y + iy as i32;
                if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                    let idx = (py * self.width as i32 + px) as usize;
                    if idx < self.buffer.len() {
                        self.buffer[idx] = rgba_to_u32(*pixel);
                    }
                }
            }
        } else {
            println!("Image not found: {}", src);
        }
    }

    pub fn present(&mut self) {
        self.graphics.set_buffer(&self.buffer, self.width as u16, self.height as u16);
    }

    pub fn handle_touch(&mut self, x: f64, y: f64) -> Option<(i32, i32)> {
        let px = x as i32;
        let py = y as i32;
        if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
            Some((px, py))
        } else {
            None
        }
    }
}

fn blend(bg: u32, fg: u32, alpha: f32) -> u32 {
    let bg_r = (bg >> 16) & 0xFF;
    let bg_g = (bg >> 8) & 0xFF;
    let bg_b = bg & 0xFF;
    let fg_r = (fg >> 16) & 0xFF;
    let fg_g = (fg >> 8) & 0xFF;
    let fg_b = fg & 0xFF;

    let r = (fg_r as f32 * alpha + bg_r as f32 * (1.0 - alpha)) as u32;
    let g = (fg_g as f32 * alpha + bg_g as f32 * (1.0 - alpha)) as u32;
    let b = (fg_b as f32 * alpha + bg_b as f32 * (1.0 - alpha)) as u32;

    (r << 16) | (g << 8) | b
}

fn rgba_to_u32(pixel: Rgba<u8>) -> u32 {
    ((pixel[0] as u32) << 16) | ((pixel[1] as u32) << 8) | (pixel[2] as u32) | ((pixel[3] as u32) << 24)
}

pub trait CanvasApp {
    fn render(&mut self, canvas: &mut Canvas);
    fn on_touch(&mut self, x: i32, y: i32);
}

pub fn run_app<T: CanvasApp + 'static>(mut app: T) {
    let event_loop = EventLoop::new().unwrap();
    let mut canvas = Canvas::new(&event_loop, 360, 640);
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                canvas.buffer.fill(0xFF000000);
                app.render(&mut canvas);
                canvas.present();
            }
            Event::WindowEvent { event: WindowEvent::Touch(touch), .. } => {
                match touch.phase {
                    TouchPhase::Started | TouchPhase::Moved => {
                        if let Some((px, py)) = canvas.handle_touch(touch.location.x, touch.location.y) {
                            canvas.touch_positions.push((touch.location.x, touch.location.y));
                            app.on_touch(px, py);
                        }
                    }
                    TouchPhase::Ended => {
                        canvas.touch_positions.clear();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }).unwrap();
}