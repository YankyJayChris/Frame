use wgpu::util::DeviceExt;
use winit::event_loop::EventLoop;
use winit::window::Window;
use rusttype::{Font, Scale};
use image::{DynamicImage, Rgba};
use resvg::usvg;

pub struct Canvas {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    window: Window,
    render_pipeline: wgpu::RenderPipeline,
    buffer: Vec<u32>,
    width: u32,
    height: u32,
    font: Font<'static>,
    vertex_buffer: wgpu::Buffer,
    texture_bind_group: wgpu::BindGroup,
}

impl Canvas {
    pub fn new(event_loop: &EventLoop<()>, width: u32, height: u32) -> Self {
        let window = Window::new(event_loop).unwrap();
        window.set_inner_size(winit::dpi::PhysicalSize::new(width, height));
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = unsafe { instance.create_surface(&window).unwrap() };
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })).unwrap();
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        )).unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // Vertex data for a full-screen quad
        let vertices: &[f32] = &[
            -1.0, -1.0, 0.0, 1.0,  // Bottom-left
             1.0, -1.0, 1.0, 1.0,  // Bottom-right
            -1.0,  1.0, 0.0, 0.0,  // Top-left
             1.0,  1.0, 1.0, 0.0,  // Top-right
        ];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Texture setup
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Canvas Texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&texture_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 4 * 4,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 },
                        wgpu::VertexAttribute { offset: 8, shader_location: 1, format: wgpu::VertexFormat::Float32x2 },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let mut font_data = Vec::new();
        std::fs::File::open("assets/fonts/Roboto-Regular.ttf").unwrap().read_to_end(&mut font_data).unwrap();
        let font = Font::try_from_vec(font_data).unwrap();

        Canvas {
            device,
            queue,
            surface,
            window,
            render_pipeline,
            buffer: vec![0; (width * height) as usize],
            width,
            height,
            font,
            vertex_buffer,
            texture_bind_group,
        }
    }

    pub fn draw_text(&mut self, text: &str, x: i32, y: i32, color: u32) {
        let scale = Scale::uniform(20.0);
        let v_metrics = self.font.v_metrics(scale);
        let offset = rusttype::point(x as f32, y as f32 + v_metrics.ascent);

        for glyph in self.font.layout(text, scale, offset) {
            if let Some(bb) = glyph.pixel_bounding_box() {
                glyph.draw(|gx, gy, v| {
                    let px = gx as i32 + bb.min.x;
                    let py = gy as i32 + bb.min.y;
                    if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                        let idx = py as usize * self.width as usize + px as usize;
                        self.buffer[idx] = blend(self.buffer[idx], color, v);
                    }
                });
            }
        }
    }

    pub fn draw_image(&mut self, src: &str, x: i32, y: i32) {
        let img = image::open(format!("assets/{}", src)).unwrap().into_rgba8();
        for (ix, iy, pixel) in img.enumerate_pixels() {
            let px = x + ix as i32;
            let py = y + iy as i32;
            if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                let idx = py as usize * self.width as usize + px as usize;
                self.buffer[idx] = rgba_to_u32(*pixel);
            }
        }
    }

    pub fn draw_svg(&mut self, src: &str, x: i32, y: i32) {
        let opt = usvg::Options::default();
        let svg_data = std::fs::read(format!("assets/{}", src)).unwrap();
        let tree = usvg::Tree::from_data(&svg_data, &opt).unwrap();
        let pixmap = resvg::render(&tree, resvg::FitTo::Width(100), None).unwrap();
        for (i, pixel) in pixmap.pixels().enumerate() {
            let px = x + (i % pixmap.width() as usize) as i32;
            let py = y + (i / pixmap.width() as usize) as i32;
            if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                let idx = py as usize * self.width as usize + px as usize;
                self.buffer[idx] = rgba_to_u32(pixel.to_rgba());
            }
        }
    }

    pub fn update(&mut self) {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Canvas Texture"),
            size: wgpu::Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            wgpu::ImageCopyTexture { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            bytemuck::cast_slice(&self.buffer),
            wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(self.width * 4), rows_per_image: None },
            wgpu::Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..4, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        self.surface.get_current_texture().unwrap().present();
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
    ((pixel[0] as u32) << 16) | ((pixel[1] as u32) << 8) | (pixel[2] as u32)
}

pub trait CanvasApp {
    fn render(&mut self, canvas: &mut Canvas);
}

pub fn run_app<T: CanvasApp + 'static>(app: T) {
    let event_loop = EventLoop::new().unwrap();
    let mut canvas = Canvas::new(&event_loop, 800, 600);
    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;
        match event {
            winit::event::Event::WindowEvent { event: winit::event::WindowEvent::CloseRequested, .. } => {
                *control_flow = winit::event_loop::ControlFlow::Exit;
            }
            winit::event::Event::MainEventsCleared => {
                canvas.buffer.fill(0x000000);
                app.render(&mut canvas);
                canvas.update();
            }
            _ => {}
        }
    }).unwrap();
}