use std::sync::Arc;
use winit::window::Window;

use crate::egui_renderer::EguiRenderer;
use crate::renderer::Renderer;
use crate::scene_loader::SceneLoader;

pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    renderer: Renderer,
    egui_renderer: EguiRenderer,
    scene_loader: SceneLoader,

    usd_filename: String,
    usd_time_code: i64,
    usd_time_code_range: std::ops::RangeInclusive<i64>,
}

impl<'a> State<'a> {
    pub async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };
        surface.configure(&device, &config);

        let renderer = Renderer::new(Arc::clone(&device), Arc::clone(&queue), config.clone());

        let egui_renderer = EguiRenderer::new(
            window,
            Arc::clone(&device),
            Arc::clone(&queue),
            surface_caps.formats[0],
        );

        let scene_loader = SceneLoader::new(Arc::clone(&device), Arc::clone(&queue));

        State {
            surface,
            device,
            queue,
            config,
            size,

            renderer,
            egui_renderer,
            scene_loader,

            usd_filename: String::new(),
            usd_time_code: 1,
            usd_time_code_range: 1..=1,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.renderer.resize(&self.config);
        }
    }

    pub fn change_scale_factor(&mut self, new_scale_factor: f32) {
        self.egui_renderer.change_scale_factor(new_scale_factor);
    }

    pub fn handle_event(&mut self, window: &Window, event: &winit::event::WindowEvent) {
        let _ = self.egui_renderer.handle_event(window, event);
    }

    pub fn draw(&mut self, window: &Window) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {}

        // main rendering
        let size = window.inner_size();
        self.scene_loader.read_scene(|scene| {
            self.renderer.render(&view, &mut encoder, size, scene);
            if let Some(range) = &scene.range {
                self.usd_time_code_range = range.start..=range.end;
            }
        });

        // egui rendering
        let prev_time_code = self.usd_time_code;
        let mut load_button_clicked = false;
        self.egui_renderer.render(
            window,
            &view,
            &mut encoder,
            &mut self.usd_filename,
            &mut self.usd_time_code,
            self.usd_time_code_range.clone(),
            &mut load_button_clicked,
        );
        if load_button_clicked {
            self.usd_time_code = 1;
            self.scene_loader.load_usd(&self.usd_filename);
            self.scene_loader.set_time_code(self.usd_time_code);
        }
        if prev_time_code != self.usd_time_code {
            self.scene_loader.set_time_code(self.usd_time_code);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
