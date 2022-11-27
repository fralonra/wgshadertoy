use crate::error;
use winit::window::Window;

pub struct Context {
    device: wgpu::Device,
    format: wgpu::TextureFormat,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
}

impl Context {
    pub fn new(window: &Window, width: u32, height: u32) -> Self {
        let default_backend = wgpu::Backends::PRIMARY;
        let backend = wgpu::util::backend_bits_from_env().unwrap_or(default_backend);
        let instance = wgpu::Instance::new(backend);
        let surface = unsafe { instance.create_surface(window) };

        let (format, (device, queue)) = futures::executor::block_on(async {
            let adapter = wgpu::util::initialize_adapter_from_env_or_default(
                &instance,
                backend,
                Some(&surface),
            )
            .await
            .expect("No suitable GPU adapters found on the system!");

            let adapter_features = adapter.features();
            (
                surface
                    .get_supported_formats(&adapter)
                    .first()
                    .copied()
                    .expect("Get preferred format"),
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
        });

        device.push_error_scope(wgpu::ErrorFilter::Validation);

        let mut context = Self {
            device,
            format,
            queue,
            surface,
        };

        context.resize(width, height);

        context
    }

    pub fn device_ref(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn finish(&self, encoder: wgpu::CommandEncoder, frame: wgpu::SurfaceTexture) {
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    pub fn get_render_stuff(
        &self,
    ) -> Result<
        (
            wgpu::CommandEncoder,
            wgpu::SurfaceTexture,
            wgpu::TextureView,
        ),
        error::SurfaceError,
    > {
        match self.surface.get_current_texture() {
            Ok(frame) => {
                let encoder = self
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    });

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                Ok((encoder, frame, view))
            }
            Err(error) => match error {
                error::SurfaceError::OutOfMemory => {
                    panic!("Swapchain error: {}. Rendering cannot continue.", error)
                }
                _ => Err(error),
            },
        }
    }

    pub fn queue_ref(&self) -> &wgpu::Queue {
        &self.queue
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
    }
}
