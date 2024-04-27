use std::sync::Arc;
use winit::window::Window;

pub struct EguiRenderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    egui_context: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
}
impl EguiRenderer {
    pub fn new(
        window: &Window,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        output_color_format: wgpu::TextureFormat,
    ) -> Self {
        let egui_context = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_context.clone(),
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(&device, output_color_format, None, 1);

        Self {
            device,
            queue,
            egui_context,
            egui_state,
            egui_renderer,
        }
    }

    pub fn change_scale_factor(&mut self, new_scale_factor: f32) {
        self.egui_context.set_zoom_factor(new_scale_factor);
    }

    pub fn handle_event(&mut self, window: &Window, event: &winit::event::WindowEvent) {
        let _ = self.egui_state.on_window_event(window, event);
    }

    pub fn render(
        &mut self,
        window: &Window,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        usd_filename: &mut String,
        usd_time_code: &mut i64,
        usd_time_code_range: std::ops::RangeInclusive<i64>,
        load_button_clicked: &mut bool,
    ) {
        let raw_input = self.egui_state.take_egui_input(window);

        let full_output = self.egui_context.run(raw_input, |ui| {
            egui::Window::new("Control Panel").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("USD File: ");
                    ui.text_edit_singleline(usd_filename);
                    if ui.button("Load .usd file").clicked() {
                        *load_button_clicked = true;
                    }
                });
                ui.scope(|ui| {
                    ui.style_mut().spacing.slider_width = 400.0;
                    ui.horizontal(|ui| {
                        ui.label("Time Code: ");
                        ui.add(egui::Slider::new(usd_time_code, usd_time_code_range));
                    });
                });
            });
        });

        self.egui_state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self
            .egui_context
            .tessellate(full_output.shapes, window.scale_factor() as f32);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(&self.device, &self.queue, *id, &image_delta);
        }

        let size = window.inner_size();
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [size.width, size.height],
            pixels_per_point: window.scale_factor() as f32,
        };
        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            encoder,
            &tris,
            &screen_descriptor,
        );
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            label: Some("egui main render pass"),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        self.egui_renderer
            .render(&mut render_pass, &tris, &screen_descriptor);
        drop(render_pass);
        for x in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(x)
        }
    }
}
