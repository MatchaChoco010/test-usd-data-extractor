use bytemuck::Zeroable;
use glam::{Vec2, Vec3};
use std::sync::Arc;
use wgpu::{CommandEncoder, TextureView};

use crate::scene::Scene;

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}
impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<Vec3>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vec3>() * 2) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Camera {
    pub view: glam::Mat4,
    pub projection: glam::Mat4,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Model {
    pub model: glam::Mat4,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub intensity: f32,
    pub color: Vec3,
    pub angle: f32,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct DirectionalLights {
    pub lights: [DirectionalLight; 4],
    pub count: u32,
    pub _padding: [u32; 3],
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct PointLight {
    pub position: Vec3,
    pub intensity: f32,
    pub color: Vec3,
    pub _padding: u32,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct PointLights {
    pub lights: [PointLight; 16],
    pub count: u32,
    pub _padding: [u32; 3],
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct SpotLight {
    pub position: Vec3,
    pub intensity: f32,
    pub direction: Vec3,
    pub angle: f32,
    pub color: Vec3,
    pub softness: f32,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct SpotLights {
    pub lights: [SpotLight; 16],
    pub count: u32,
    pub _padding: [u32; 3],
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}
impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.

    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}

pub struct Renderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    directional_lights_buffer: wgpu::Buffer,
    point_lights_buffer: wgpu::Buffer,
    spot_lights_buffer: wgpu::Buffer,
    lights_bind_group: wgpu::BindGroup,
    model_bind_group_layout: wgpu::BindGroupLayout,
    model_bind_groups: Vec<wgpu::BindGroup>,
    depth_texture: Texture,
    render_pipeline: wgpu::RenderPipeline,
}
impl Renderer {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        config: wgpu::SurfaceConfiguration,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<Camera>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let directional_lights_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<DirectionalLights>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let point_lights_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<PointLights>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let spot_lights_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<SpotLights>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let lights_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("lights_bind_group_layout"),
            });
        let lights_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &lights_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: directional_lights_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: point_lights_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: spot_lights_buffer.as_entire_binding(),
                },
            ],
            label: Some("lights_bind_group"),
        });

        let model_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("model_bind_group_layout"),
            });

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &lights_bind_group_layout,
                    &model_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
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
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            device,
            queue,
            camera_buffer,
            camera_bind_group,
            directional_lights_buffer,
            point_lights_buffer,
            spot_lights_buffer,
            lights_bind_group,
            model_bind_group_layout,
            model_bind_groups: vec![],
            depth_texture,
            render_pipeline,
        }
    }

    pub fn resize(&mut self, config: &wgpu::SurfaceConfiguration) {
        self.depth_texture = Texture::create_depth_texture(&self.device, config, "depth_texture");
    }

    pub fn render(
        &mut self,
        view: &TextureView,
        encoder: &mut CommandEncoder,
        size: winit::dpi::PhysicalSize<u32>,
        scene: &Scene,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        {
            let camera = Camera {
                view: scene.camera.view_matrix,
                projection: glam::Mat4::perspective_rh(
                    scene.camera.fovy,
                    size.width as f32 / size.height as f32,
                    0.01,
                    100.0,
                ),
            };
            self.queue
                .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera]));
        }

        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

        {
            let mut directional_lights = Vec::with_capacity(4);
            for light in scene.distant_lights.values() {
                directional_lights.push(DirectionalLight {
                    direction: light.direction,
                    intensity: light.intensity,
                    color: light.color,
                    angle: light.angle,
                });
            }
            let mut lights = [Zeroable::zeroed(); 4];
            for i in 0..4 {
                lights[i] = directional_lights
                    .get(i)
                    .cloned()
                    .unwrap_or(Zeroable::zeroed());
            }
            let directional_lights = DirectionalLights {
                lights,
                count: directional_lights.len().max(4) as u32,
                _padding: [0; 3],
            };
            self.queue.write_buffer(
                &self.directional_lights_buffer,
                0,
                bytemuck::cast_slice(&[directional_lights]),
            );
        }

        {
            let mut point_lights = Vec::with_capacity(16);
            let mut spot_lights = Vec::with_capacity(16);
            for light in scene.sphere_lights.values() {
                if light.is_spot {
                    spot_lights.push(SpotLight {
                        position: light.position,
                        intensity: light.intensity,
                        direction: light.direction.unwrap(),
                        color: light.color,
                        angle: light.cone_angle.unwrap().to_radians(),
                        softness: light.cone_softness.unwrap(),
                    });
                } else {
                    point_lights.push(PointLight {
                        position: light.position,
                        intensity: light.intensity,
                        color: light.color,
                        _padding: 0,
                    });
                }
            }
            {
                let mut lights = [Zeroable::zeroed(); 16];
                for i in 0..16 {
                    lights[i] = point_lights.get(i).cloned().unwrap_or(Zeroable::zeroed());
                }
                let point_lights = PointLights {
                    lights,
                    count: point_lights.len().max(16) as u32,
                    _padding: [0; 3],
                };
                self.queue.write_buffer(
                    &self.point_lights_buffer,
                    0,
                    bytemuck::cast_slice(&[point_lights]),
                );
            }
            {
                let mut lights = [Zeroable::zeroed(); 16];
                for i in 0..16 {
                    lights[i] = spot_lights.get(i).cloned().unwrap_or(Zeroable::zeroed());
                }
                let spot_lights = SpotLights {
                    lights,
                    count: spot_lights.len().max(16) as u32,
                    _padding: [0; 3],
                };
                self.queue.write_buffer(
                    &self.spot_lights_buffer,
                    0,
                    bytemuck::cast_slice(&[spot_lights]),
                );
            }
        }

        render_pass.set_bind_group(1, &self.lights_bind_group, &[]);

        self.model_bind_groups.clear();
        for (_, mesh) in &scene.meshes {
            let model_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.model_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &mesh.model_buffer,
                        offset: 0,
                        size: None,
                    }),
                }],
                label: Some("model_bind_group"),
            });
            self.model_bind_groups.push(model_bind_group);
        }

        render_pass.set_pipeline(&self.render_pipeline);

        for (i, mesh) in scene.meshes.values().enumerate() {
            render_pass.set_bind_group(2, &self.model_bind_groups[i], &[]);
            if let Some(vertex_buffer) = mesh.vertex_buffer.as_ref() {
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw(0..mesh.vertex_count, 0..1);
            }
        }
    }
}
