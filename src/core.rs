#[cfg(feature = "fps")]
use crate::fps_counter::FpsCounter;
use crate::{
    event::{AppResponse, AppStatus, EventProxyWinit, UserEvent},
    fs::{create_file, select_file, select_texture, write_file},
    runtime::Runtime,
    ui::{EditContext, Ui, UiState},
    viewport::Viewport,
    wgs::{self, WgsData},
};
use anyhow::Result;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit::State;
use image::{ColorType, ImageResult};
use std::{
    fs::read,
    io::{self, Cursor},
    path::PathBuf,
    time::Instant,
};
use wgpu::SurfaceError;
use winit::{event::WindowEvent, event_loop::EventLoop, window::Window};

const DEFAULT_FRAGMENT: &'static str = include_str!("assets/frag.default.wgsl");
const DEFAULT_VERTEX: &'static str = include_str!("assets/vert.wgsl");

pub struct Core {
    cursor: [f32; 2],
    event_proxy: EventProxyWinit<UserEvent>,
    #[cfg(feature = "fps")]
    fps_counter: FpsCounter,
    has_validation_error: bool,
    runtime: Runtime,
    size: (f32, f32),
    state: State,
    status: AppStatus,
    status_clock: Instant,
    wgs_data: WgsData,
    wgs_path: Option<PathBuf>,
    ui: Ui,
    ui_edit_context: EditContext,
    ui_render_pass: RenderPass,
}

impl Core {
    pub fn new<W>(
        event_loop: &EventLoop<UserEvent>,
        w: &W,
        width: f32,
        height: f32,
        scale_factor: f32,
    ) -> Result<Self>
    where
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    {
        let wgs_data = default_wgs();

        let textures = wgs_data
            .textures_ref()
            .iter()
            .map(|texture| (texture.width, texture.height, &texture.data))
            .collect();

        let runtime = Runtime::new(
            w,
            &concat_shader_frag(&wgs_data.frag(), wgs_data.textures_ref().len()),
            DEFAULT_VERTEX,
            textures,
        )?;

        let mut state = State::new(&event_loop);
        state.set_pixels_per_point(scale_factor);

        let mut ui = Ui::new();

        for texture in wgs_data.textures_ref() {
            ui.add_texture(texture.width, texture.height, &texture.data);
        }

        let ui_edit_context = EditContext {
            frag: wgs_data.frag(),
            name: wgs_data.name(),
        };

        let ui_render_pass = RenderPass::new(runtime.device_ref(), runtime.format(), 1);

        let event_proxy = event_loop.create_proxy();
        let event_proxy = EventProxyWinit::from_proxy(event_proxy);

        let initial_status = AppStatus::Info("Shader compiled successfully!".to_owned());

        Ok(Self {
            cursor: Default::default(),
            event_proxy,
            #[cfg(feature = "fps")]
            fps_counter: FpsCounter::new(),
            has_validation_error: false,
            runtime,
            size: (width, height),
            state,
            status: initial_status,
            status_clock: Instant::now(),
            wgs_data,
            wgs_path: None,
            ui,
            ui_edit_context,
            ui_render_pass,
        })
    }

    pub fn handle_mouse_input(&mut self, press: bool) {
        if self.cursor[0] > self.size.0 / 2.0 {
            if press {
                self.runtime.update_mouse_press();
            } else {
                self.runtime.update_mouse_release();
            }
        }
    }

    pub fn handle_user_event(&mut self, event: UserEvent) -> AppResponse {
        let mut response = AppResponse::default();

        let mut need_update = false;

        match event {
            UserEvent::CaptureImage => {
                let half_width = self.size.0 / 2.0;

                let viewport = Viewport {
                    x: half_width,
                    width: half_width,
                    height: self.size.1,
                    ..Default::default()
                };

                let filename = format!(
                    "Capture_{}.{}",
                    self.wgs_data.name().to_ascii_lowercase().replace(" ", "_"),
                    "png"
                );
                self.runtime
                    .request_capture_image(&viewport, move |width, height, buffer| {
                        on_image_captured(width, height, buffer, &filename);
                    });
            }
            UserEvent::ChangeTexture(index) => {
                let path = select_texture();
                if path.is_some() {
                    let path = path.unwrap();

                    match open_image(path) {
                        Ok((width, height, data)) => {
                            self.runtime.change_texture(index, width, height, &data);
                            self.ui.change_texture(index, width, height, &data);
                            self.wgs_data.change_texture(index, width, height, data);
                        }
                        Err(err) => log::error!("Failed to open texture: {}", err),
                    }
                } else {
                    self.runtime.remove_texture(index);
                    self.ui.remove_texture(index);
                    self.wgs_data.remove_texture(index);
                }
            }
            UserEvent::NewFile => {
                self.wgs_data = default_wgs();
                self.wgs_path = None;
                self.ui.reset_textures();
                self.ui_edit_context.frag = self.wgs_data.frag();
                self.ui_edit_context.name = self.wgs_data.name();

                response.set_title = Some(format_title(&self.wgs_path));

                need_update = true;
            }
            UserEvent::OpenFile => {
                let path = select_file();
                if path.is_some() {
                    let path = path.unwrap();
                    match load_wgs(path.clone()) {
                        Ok(wgs_data) => {
                            self.wgs_data = wgs_data;
                            self.wgs_path = Some(path);

                            self.ui.reset_textures();
                            for texture in self.wgs_data.textures_ref() {
                                self.runtime.add_texture(
                                    texture.width,
                                    texture.height,
                                    &texture.data,
                                );
                                self.ui
                                    .add_texture(texture.width, texture.height, &texture.data);
                            }

                            self.ui_edit_context.frag = self.wgs_data.frag();
                            self.ui_edit_context.name = self.wgs_data.name();

                            response.set_title = Some(format_title(&self.wgs_path));

                            need_update = true;
                        }
                        Err(err) => {
                            log::error!("Failed to open file: {}", err);
                        }
                    }
                }
            }
            UserEvent::OpenTexture => {
                let path = select_texture();
                if path.is_some() {
                    let path = path.unwrap();

                    match open_image(path) {
                        Ok((width, height, data)) => {
                            self.runtime.add_texture(width, height, &data);
                            self.ui.add_texture(width, height, &data);
                            self.wgs_data.add_texture(width, height, data);
                        }
                        Err(err) => {
                            log::error!("Failed to open texture: {}", err);
                        }
                    }
                }
            }
            UserEvent::Pause => {
                self.runtime.pause();
            }
            UserEvent::RequestRedraw => {
                self.wgs_data.set_frag(&self.ui_edit_context.frag);
                need_update = true;
            }
            UserEvent::Restart => {
                self.runtime.restart();
            }
            UserEvent::Resume => {
                self.runtime.resume();
            }
            UserEvent::SaveFile => {
                if let Some(title) = self.save_file() {
                    response.set_title = Some(title);
                }
            }
            UserEvent::SaveFileAs => {
                if let Some(title) = self.save_file_as() {
                    response.set_title = Some(title);
                }
            }
        }

        if need_update {
            self.has_validation_error = false;

            let textures = self
                .wgs_data
                .textures_ref()
                .iter()
                .map(|texture| (texture.width, texture.height, &texture.data))
                .collect();

            match self.runtime.update(
                &concat_shader_frag(&self.wgs_data.frag(), self.wgs_data.textures_ref().len()),
                textures,
            ) {
                Ok(()) => {
                    self.change_status(AppStatus::Info("Shader compiled successfully!".to_owned()));

                    response.request_redraw = true;
                }
                Err(err) => {
                    self.change_status(AppStatus::Error(err.to_string()));
                }
            }
        }

        response
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        self.state.on_event(self.ui.context(), event);
    }

    pub fn redraw(&mut self, window: &Window) -> bool {
        let mut request_redraw = false;

        if self.status_clock.elapsed().as_secs() > 5 {
            self.status = AppStatus::Idle;
        }

        if let Some(error_scope) = self.runtime.pop_error_scope() {
            self.has_validation_error = true;

            if let wgpu::Error::Validation { description, .. } = error_scope {
                log::error!("Validation error: {:?}", description);
            }

            self.change_status(AppStatus::Error("Shader validation error".to_string()));
        }

        if let Err(error) = self.render(window) {
            match error.downcast_ref::<SurfaceError>() {
                Some(SurfaceError::OutOfMemory) => {
                    panic!("Swapchain error: {}. Rendering cannot continue.", error)
                }
                Some(_) | None => {
                    log::warn!("Failed to render: {}", error);
                    request_redraw = true;
                }
            }
        }

        #[cfg(feature = "fps")]
        log::info!("FPS: {}", self.fps_counter.tick());

        request_redraw
    }

    pub fn resize(&mut self, width: f32, height: f32, scale_factor: f32) {
        self.size = (width, height);

        self.runtime.resize(width, height);
        self.state.set_pixels_per_point(scale_factor);
    }

    pub fn update_cursor(&mut self, x: f32, y: f32) {
        self.cursor = [x, y];

        let half_width = self.size.0 / 2.0;
        if x > half_width {
            self.runtime.update_cursor([x - half_width, y]);
        }
    }

    fn change_status(&mut self, status: AppStatus) {
        self.status = status;

        self.status_clock = Instant::now();
    }

    fn render(&mut self, window: &Window) -> Result<()> {
        self.runtime.frame_start()?;

        let half_width = self.size.0 / 2.0;

        if !self.has_validation_error {
            let viewport = Viewport {
                x: half_width,
                width: half_width,
                height: self.size.1,
                ..Default::default()
            };
            self.runtime.render(&viewport)?;
        }

        {
            let ui_state = UiState {
                file_saved: self.wgs_path.is_some(),
                is_paused: self.runtime.is_paused(),
                status: self.status.clone(),
                texture_addable: self.wgs_data.textures_ref().len() + 1
                    < self.runtime.max_texture_count() as usize,
            };

            let raw_input = self.state.take_egui_input(window);

            let full_output = self.ui.prepare(
                raw_input,
                &mut self.ui_edit_context,
                &self.event_proxy,
                ui_state,
            );

            self.state.handle_platform_output(
                window,
                self.ui.context(),
                full_output.platform_output,
            );

            let clipped_primitives = self.ui.context().tessellate(full_output.shapes);

            let viewport = Viewport {
                width: half_width,
                height: self.size.1,
                ..Default::default()
            };

            self.runtime.render_with(|device, queue, view| {
                self.ui_render_pass
                    .add_textures(device, queue, &full_output.textures_delta)?;

                let screen_descriptor = ScreenDescriptor {
                    physical_width: viewport.width as u32,
                    physical_height: viewport.height as u32,
                    scale_factor: window.scale_factor() as f32,
                };

                self.ui_render_pass.update_buffers(
                    device,
                    queue,
                    &clipped_primitives,
                    &screen_descriptor,
                );

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("UI Encoder"),
                });

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Egui Main Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view,
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
                        .execute_with_renderpass(
                            &mut render_pass,
                            &clipped_primitives,
                            &screen_descriptor,
                        )
                        .unwrap();
                }

                queue.submit(Some(encoder.finish()));

                Ok(())
            })?;
        }

        self.runtime.frame_finish()?;

        Ok(())
    }

    fn save_file(&mut self) -> Option<String> {
        self.save_file_impl(false)
    }

    fn save_file_as(&mut self) -> Option<String> {
        self.save_file_impl(true)
    }

    fn save_file_impl(&mut self, save_as: bool) -> Option<String> {
        self.wgs_data.set_frag(&self.ui_edit_context.frag);
        self.wgs_data.set_name(&self.ui_edit_context.name);

        if save_as {
            // Save as.
            if let Some(path) = create_file(&format!(
                "{}.{}",
                self.wgs_data.name().to_ascii_lowercase().replace(" ", "_"),
                wgs::EXTENSION
            )) {
                self.wgs_path = Some(path);
            // Early return when cancelled.
            } else {
                return None;
            }
        // Never been saved before.
        } else if self.wgs_path.is_none() {
            self.wgs_path = create_file(&format!(
                "{}.{}",
                self.wgs_data.name().to_ascii_lowercase().replace(" ", "_"),
                wgs::EXTENSION
            ));
        }

        if self.wgs_path.is_some() {
            self.wgs_data.set_frag(&self.ui_edit_context.frag);
            save_wgs(&self.wgs_path.as_ref().unwrap(), &self.wgs_data);

            self.change_status(AppStatus::Info("Shader saved successfully!".to_owned()));

            Some(format_title(&self.wgs_path))
        } else {
            None
        }
    }
}

fn concat_shader_frag(main_image: &str, texture_count: usize) -> String {
    let prefix = include_str!("assets/frag.prefix.wgsl");

    let mut texture2ds = String::new();
    for index in 0..texture_count {
        texture2ds.push_str(&format!("@group({}) @binding(0)\n", index + 1,));
        texture2ds.push_str(&format!("var texture{}: texture_2d<f32>;\n", index));
        texture2ds.push_str(&format!("@group({}) @binding(1)\n", index + 1,));
        texture2ds.push_str(&format!("var sampler{}: sampler;\n", index));
    }

    let suffix = include_str!("assets/frag.suffix.wgsl");
    format!("{}\n{}\n{}\n{}", prefix, texture2ds, main_image, suffix)
}

fn default_wgs() -> WgsData {
    WgsData::new(wgs::DEFAULT_NAME, DEFAULT_FRAGMENT)
}

fn format_title(file_path: &Option<PathBuf>) -> String {
    format!(
        "WgShadertoy - {}",
        match file_path {
            Some(file_path) => file_path.display().to_string(),
            None => "Untitled".to_owned(),
        }
    )
}

fn load_wgs(path: PathBuf) -> io::Result<WgsData> {
    let buffer = read(path.clone())?;
    let mut reader = Cursor::new(&buffer);

    log::info!("Opened wgs file: {:?}", path);

    Ok(WgsData::load(&mut reader).unwrap())
}

fn on_image_captured(width: u32, height: u32, buffer: Vec<u8>, filename: &str) {
    if let Some(path) = create_file(filename) {
        match image::save_buffer(path.clone(), &buffer, width, height, ColorType::Rgba8) {
            Ok(()) => log::info!("Saving image file: {:?}", path),
            Err(err) => log::error!("Failed to save image: {}", err),
        }
    }
}

fn open_image(path: PathBuf) -> ImageResult<(u32, u32, Vec<u8>)> {
    let image = image::open(path)?;

    let image = image.into_rgba8();

    let width = image.width();
    let height = image.height();
    let data = image.into_vec();

    Ok((width, height, data))
}

fn save_wgs(path: &PathBuf, wgs: &WgsData) {
    let mut writer = Cursor::new(vec![]);
    wgs.save(&mut writer).unwrap();

    write_file(&path, writer.into_inner());

    log::info!("Saving wgs file: {:?}", path);
}
