mod uniform;

use crate::viewport::Viewport;
use anyhow::Result;
use egui::{ClippedPrimitive, TexturesDelta};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use std::{borrow::Cow, time::Instant};
use uniform::Uniform;
use wgpu::{util::DeviceExt, SurfaceTexture, TextureView};
use winit::window::Window;

pub const UNIFORM_GROUP_ID: u32 = 0;

async fn init_device(
    window: &Window,
) -> (
    wgpu::Surface,
    wgpu::TextureFormat,
    (wgpu::Device, wgpu::Queue),
) {
    let default_backend = wgpu::Backends::PRIMARY;
    let backend = wgpu::util::backend_bits_from_env().unwrap_or(default_backend);
    let instance = wgpu::Instance::new(backend);
    let surface = unsafe { instance.create_surface(window) };

    let adapter =
        wgpu::util::initialize_adapter_from_env_or_default(&instance, backend, Some(&surface))
            .await
            .expect("No suitable GPU adapters found on the system!");

    let adapter_features = adapter.features();

    let texture_format = surface
        .get_supported_formats(&adapter)
        .first()
        .copied()
        .expect("Get preferred format");
    (
        surface,
        texture_format,
        adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device Descriptor"),
                    features: adapter_features & wgpu::Features::default(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("Request device"),
    )
}

pub struct Runtime {
    device: wgpu::Device,
    format: wgpu::TextureFormat,
    pipeline: wgpu::RenderPipeline,
    queue: wgpu::Queue,
    sampler: wgpu::Sampler,
    shader_vert: String,
    surface: wgpu::Surface,
    texture_bind_groups: Vec<(wgpu::BindGroupLayout, wgpu::BindGroup)>,
    time_instant: Instant,
    ui_render_pass: RenderPass,
    uniform: Uniform,
    uniform_bind_group: wgpu::BindGroup,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
}

impl Runtime {
    pub fn new(
        window: &Window,
        shader_frag: &str,
        shader_vert: &str,
        textures: Vec<(u32, u32, &Vec<u8>)>,
    ) -> Result<Self> {
        let (surface, format, (device, queue)) = futures::executor::block_on(init_device(window));

        device.push_error_scope(wgpu::ErrorFilter::Validation);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        let (uniform, uniform_buffer, uniform_bind_group_layout, uniform_bind_group) =
            setup_uniform(&device);

        let texture_bind_groups = textures
            .iter()
            .map(|(width, height, data)| {
                create_texture(&device, &queue, &sampler, *width, *height, &data)
            })
            .collect::<Vec<(wgpu::BindGroupLayout, wgpu::BindGroup)>>();

        let mut bind_group_layouts = vec![&uniform_bind_group_layout];
        for (layout, _) in &texture_bind_groups {
            bind_group_layouts.push(layout);
        }
        let pipeline = build_pipeline(
            shader_frag,
            shader_vert,
            &bind_group_layouts,
            &device,
            format,
        )?;

        let ui_render_pass = RenderPass::new(&device, format, 1);

        Ok(Self {
            device,
            format,
            pipeline,
            queue,
            sampler,
            shader_vert: shader_vert.to_string(),
            surface,
            texture_bind_groups,
            time_instant: Instant::now(),
            ui_render_pass,
            uniform,
            uniform_bind_group,
            uniform_bind_group_layout,
            uniform_buffer,
        })
    }

    pub fn add_texture(&mut self, width: u32, height: u32, buffer: &[u8]) {
        self.texture_bind_groups.push(create_texture(
            &self.device,
            &self.queue,
            &self.sampler,
            width,
            height,
            buffer,
        ));
    }

    pub fn change_texture(&mut self, index: usize, width: u32, height: u32, buffer: &[u8]) {
        self.texture_bind_groups[index] = create_texture(
            &self.device,
            &self.queue,
            &self.sampler,
            width,
            height,
            buffer,
        );
    }

    pub fn get_surface(&self) -> Result<(SurfaceTexture, TextureView)> {
        let surface_texture = self.surface.get_current_texture()?;

        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        Ok((surface_texture, texture_view))
    }

    pub fn max_texture_count(&self) -> u32 {
        self.device.limits().max_bind_groups
    }

    pub fn pop_error_scope(&mut self) -> Option<wgpu::Error> {
        let error_scope =
            futures::executor::block_on(async { self.device.pop_error_scope().await });

        self.device.push_error_scope(wgpu::ErrorFilter::Validation);

        error_scope
    }

    pub fn remove_texture(&mut self, index: usize) {
        self.texture_bind_groups.remove(index);
    }

    pub fn render(&mut self, view: &TextureView, viewport: &Viewport) -> Result<()> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.uniform.time = self.time_instant.elapsed().as_secs_f32();

        self.queue
            .write_buffer(&self.uniform_buffer, 0, self.uniform.as_bytes());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_viewport(
                viewport.x,
                viewport.y,
                viewport.width,
                viewport.height,
                viewport.min_depth,
                viewport.max_depth,
            );

            render_pass.set_pipeline(&self.pipeline);

            render_pass.set_bind_group(UNIFORM_GROUP_ID, &self.uniform_bind_group, &[]);
            let mut index = 1;
            for (_, bind_group) in &self.texture_bind_groups {
                render_pass.set_bind_group(index, bind_group, &[]);
                index += 1;
            }
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));

        Ok(())
    }

    pub fn render_ui(
        &mut self,
        view: &TextureView,
        clipped_primitives: &Vec<ClippedPrimitive>,
        textures_delta: &TexturesDelta,
        viewport: &Viewport,
        scale_factor: f32,
    ) -> Result<()> {
        self.ui_render_pass
            .add_textures(&self.device, &self.queue, textures_delta)?;

        let screen_descriptor = ScreenDescriptor {
            physical_width: viewport.width as u32,
            physical_height: viewport.height as u32,
            scale_factor,
        };

        self.ui_render_pass.update_buffers(
            &self.device,
            &self.queue,
            &clipped_primitives,
            &screen_descriptor,
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("UI Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_viewport(
                viewport.x,
                viewport.y,
                viewport.width,
                viewport.height,
                viewport.min_depth,
                viewport.max_depth,
            );

            self.ui_render_pass
                .execute_with_renderpass(&mut render_pass, &clipped_primitives, &screen_descriptor)
                .unwrap();
        }

        self.queue.submit(Some(encoder.finish()));

        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.format,
                width,
                height,
                present_mode: wgpu::PresentMode::AutoVsync,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
            },
        );

        self.uniform.resolution = [width as f32 / 2.0, height as f32];
    }

    pub fn update(&mut self, shader_frag: &str, textures: Vec<(u32, u32, &Vec<u8>)>) -> Result<()> {
        self.time_instant = Instant::now();

        let resolution = self.uniform.resolution;
        self.uniform = Uniform::default();
        self.uniform.resolution = resolution;

        let texture_bind_groups = textures
            .iter()
            .map(|(width, height, data)| {
                create_texture(
                    &self.device,
                    &self.queue,
                    &self.sampler,
                    *width,
                    *height,
                    &data,
                )
            })
            .collect();

        self.texture_bind_groups = texture_bind_groups;
        let mut bind_group_layouts = vec![&self.uniform_bind_group_layout];
        for (layout, _) in &self.texture_bind_groups {
            bind_group_layouts.push(layout);
        }
        self.pipeline = build_pipeline(
            shader_frag,
            &self.shader_vert,
            &bind_group_layouts,
            &self.device,
            self.format,
        )?;

        Ok(())
    }

    pub fn update_cursor(&mut self, cursor: [f32; 2]) {
        self.uniform.cursor = [cursor[0], self.uniform.resolution[1] - cursor[1]];
    }

    pub fn update_mouse_press(&mut self) {
        self.uniform.mouse_down = 1;
        self.uniform.mouse_press = self.uniform.cursor;
    }

    pub fn update_mouse_release(&mut self) {
        self.uniform.mouse_down = 0;
        self.uniform.mouse_release = self.uniform.cursor;
    }
}

fn build_pipeline(
    shader_frag: &str,
    shader_vert: &str,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    device: &wgpu::Device,
    texture_format: wgpu::TextureFormat,
) -> Result<wgpu::RenderPipeline> {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts,
        push_constant_ranges: &[],
    });
    let fs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::from(shader_frag)),
    });
    let vs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::from(shader_vert)),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vs_module,
            entry_point: "main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &fs_module,
            entry_point: "main",
            targets: &[Some(wgpu::ColorTargetState {
                format: texture_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });
    Ok(pipeline)
}

fn create_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    sampler: &wgpu::Sampler,
    width: u32,
    height: u32,
    buffer: &[u8],
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    let texture_size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("Diffuse Texture"),
    });

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        buffer,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: std::num::NonZeroU32::new(4 * width),
            rows_per_image: std::num::NonZeroU32::new(height),
        },
        texture_size,
    );

    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
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
            label: Some("Texture Bind Group Layout"),
        });

    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
        label: Some("Bind Group"),
    });

    (texture_bind_group_layout, texture_bind_group)
}

fn get_texture_view(surface_texture: &SurfaceTexture) -> Result<TextureView> {
    let texture_view = surface_texture
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    Ok(texture_view)
}

fn setup_uniform(
    device: &wgpu::Device,
) -> (
    Uniform,
    wgpu::Buffer,
    wgpu::BindGroupLayout,
    wgpu::BindGroup,
) {
    let uniform = Uniform::default();

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: uniform.as_bytes(),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let uniform_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Uniform Bind Group"),
        layout: &uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    (
        uniform,
        uniform_buffer,
        uniform_bind_group_layout,
        uniform_bind_group,
    )
}
