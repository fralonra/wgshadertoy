use crate::{
    event::{AppResponse, AppStatus, EventProxyWinit, UserEvent},
    fps_counter::FpsCounter,
    fs::{create_file, select_file, select_texture, write_file},
    preferences::Preferences,
    ui::{EditContext, Ui, UiState},
};
use anyhow::{bail, Result};
use egui::ClippedPrimitive;
use egui_wgpu::{renderer::ScreenDescriptor, Renderer};
use egui_winit::State;
use image::ColorType;
use std::{
    fs::read,
    io::{self, Cursor},
    path::{Path, PathBuf},
    time::Instant,
};
use wgs_core::WgsData;
use wgs_runtime_wgpu::{wgpu, Runtime, RuntimeExt, Viewport};
use winit::{event::WindowEvent, event_loop::EventLoop, window::Window};

pub struct Core {
    cursor: [f32; 2],
    event_proxy: EventProxyWinit<UserEvent>,
    fps: Option<usize>,
    fps_counter: FpsCounter,
    has_validation_error: bool,
    preferences: Preferences,
    runtime: Runtime,
    size: (f32, f32),
    state: State,
    status: AppStatus,
    status_clock: Instant,
    ui: Ui,
    ui_edit_context: EditContext,
    ui_renderer: Renderer,
    wgs_path: Option<PathBuf>,
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
        let wgs = WgsData::default();

        let viewport = Viewport {
            x: width,
            width: width / 2.0,
            height,
            ..Default::default()
        };

        let mut state = State::new(&event_loop);
        state.set_pixels_per_point(scale_factor);

        let mut ui = Ui::new();

        for texture in wgs.textures_ref() {
            ui.add_texture(texture.width, texture.height, &texture.data);
        }

        let ui_edit_context = EditContext {
            frag: wgs.frag(),
            name: wgs.name(),
        };

        let mut runtime = futures::executor::block_on(Runtime::new(w, wgs, Some(viewport)))?;
        runtime.resize(width, height);

        let ui_renderer = Renderer::new(runtime.device_ref(), runtime.format(), None, 1);

        let event_proxy = event_loop.create_proxy();
        let event_proxy = EventProxyWinit::from_proxy(event_proxy);

        let initial_status = AppStatus::Info("Shader compiled successfully!".to_owned());

        Ok(Self {
            cursor: [0.0, 0.0],
            event_proxy,
            fps: None,
            fps_counter: FpsCounter::new(),
            has_validation_error: false,
            runtime,
            preferences: Preferences::default(),
            size: (width, height),
            state,
            status: initial_status,
            status_clock: Instant::now(),
            ui,
            ui_edit_context,
            ui_renderer,
            wgs_path: None,
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

        let mut update_result = None;

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
                    self.runtime
                        .wgs()
                        .name()
                        .to_ascii_lowercase()
                        .replace(' ', "_"),
                    "png"
                );
                self.runtime.request_capture_image(
                    &viewport,
                    move |runtime, width, height, buffer| {
                        runtime.pause();

                        on_image_captured(width, height, buffer, &filename);

                        runtime.resume();
                    },
                );
            }
            UserEvent::ChangeTexture(index) => {
                if let Some(path) = select_texture() {
                    match open_image(path) {
                        Ok((width, height, data)) => {
                            self.ui.change_texture(index, width, height, &data);
                            self.runtime.change_texture(index, width, height, data);
                        }
                        Err(err) => {
                            log::error!("{}", format!("Failed to open texture: {}", err));

                            self.change_status(AppStatus::Error(format!(
                                "{}: {}",
                                fl!("status_err_open_texture"),
                                err
                            )));
                        }
                    }
                }
            }
            UserEvent::NewFile => {
                let wgs = WgsData::default();
                self.wgs_path = None;

                self.ui.reset_textures();
                self.ui_edit_context.frag = wgs.frag();
                self.ui_edit_context.name = wgs.name();

                update_result = Some(self.runtime.load(wgs));

                response.set_title = Some(self.format_title());
            }
            UserEvent::OpenAbout => {
                response.request_open_about = true;
            }
            UserEvent::OpenExample(example) => {
                let bytes = example.data();

                match load_wgs_from_buffer(bytes) {
                    Ok(wgs) => {
                        self.wgs_path = None;

                        self.load_wgs(&wgs);

                        update_result = Some(self.runtime.load(wgs));

                        response.set_title = Some(self.format_title());
                    }
                    Err(err) => {
                        log::error!("{}", format!("Failed to open example: {}", err));

                        self.change_status(AppStatus::Error(format!(
                            "{}: {}",
                            fl!("status_err_open_example"),
                            err
                        )));
                    }
                }
            }
            UserEvent::OpenFile => {
                if let Some(path) = select_file() {
                    match load_wgs_from_file(&path) {
                        Ok(wgs) => {
                            self.wgs_path = Some(path);

                            self.load_wgs(&wgs);

                            update_result = Some(self.runtime.load(wgs));

                            response.set_title = Some(self.format_title());
                        }
                        Err(err) => {
                            log::error!("{}", format!("Failed to open file: {}", err));

                            self.change_status(AppStatus::Error(format!(
                                "{}: {}",
                                fl!("status_err_open_file"),
                                err
                            )));
                        }
                    }
                }
            }
            UserEvent::OpenTexture => {
                if let Some(path) = select_texture() {
                    match open_image(path) {
                        Ok((width, height, data)) => {
                            self.ui.add_texture(width, height, &data);
                            self.runtime.add_texture(width, height, data);
                        }
                        Err(err) => {
                            log::error!("{}", format!("Failed to open texture: {}", err));

                            self.change_status(AppStatus::Error(format!(
                                "{}: {}",
                                fl!("status_err_open_texture"),
                                err
                            )));
                        }
                    }
                }
            }
            UserEvent::Pause => {
                self.runtime.pause();
            }
            UserEvent::Quit => {
                response.request_quit = true;
            }
            UserEvent::RemoveTexture(index) => {
                self.runtime.remove_texture(index);
                self.ui.remove_texture(index);
            }
            UserEvent::RequestRedraw => {
                self.runtime.set_wgs_frag(&self.ui_edit_context.frag);

                update_result = Some(self.runtime.compile());
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

        if let Some(result) = update_result {
            self.has_validation_error = false;

            match result {
                Ok(()) => {
                    self.change_status(AppStatus::Info(fl!("status_compile_ok")));

                    response.request_redraw = true;
                }
                Err(err) => {
                    self.change_status(AppStatus::Error(err.to_string()));
                }
            }
        }

        response
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        self.state.on_event(self.ui.context(), event).repaint
    }

    pub fn preferences(&self) -> &Preferences {
        &self.preferences
    }

    pub fn redraw(&mut self, window: &Window) {
        if self.status_clock.elapsed().as_secs() > 5 {
            self.status = AppStatus::Idle;
        }

        if let Some(error_scope) = self.runtime.pop_error_scope() {
            self.has_validation_error = true;

            if let wgpu::Error::Validation { description, .. } = error_scope {
                log::error!("Validation error: {:?}", description);
            }

            self.change_status(AppStatus::Error(fl!("status_err_valid")));
        }

        if let Err(error) = self.render(window) {
            match error.downcast_ref::<wgpu::SurfaceError>() {
                Some(wgpu::SurfaceError::OutOfMemory) => {
                    panic!("Swapchain error: {}. Rendering cannot continue.", error)
                }
                Some(_) | None => {
                    log::warn!("Failed to render: {}", error);
                }
            }
        }

        if self.preferences.record_fps {
            let fps = self.fps_counter.tick();

            log::info!("FPS: {}", fps);

            self.fps = Some(fps);
        } else if self.fps.is_some() {
            self.fps = None;
        }
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

    pub fn window_title(&self) -> String {
        self.format_title()
    }

    fn change_status(&mut self, status: AppStatus) {
        self.status = status;

        self.status_clock = Instant::now();
    }

    fn format_title(&self) -> String {
        format!("[{}] - WgShadertoy", self.runtime.wgs().name())
    }

    fn load_wgs(&mut self, wgs: &WgsData) {
        self.ui.reset_textures();

        for texture in wgs.textures_ref() {
            self.ui
                .add_texture(texture.width, texture.height, &texture.data);
        }

        self.ui_edit_context.frag = wgs.frag();
        self.ui_edit_context.name = wgs.name();
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
            self.runtime.set_viewport(Some(viewport));

            self.runtime.render()?;
        }

        {
            let ui_state = UiState {
                can_capture: self.runtime.is_capture_supported(),
                file_saved: self.wgs_path.is_some(),
                fps: self.fps,
                is_paused: self.runtime.is_paused(),
                status: self.status.clone(),
                texture_addable: self.runtime.wgs().textures_ref().len() + 1
                    < self.runtime.max_texture_count() as usize,
            };

            let raw_input = self.state.take_egui_input(window);

            let full_output = self.ui.prepare(
                raw_input,
                &mut self.preferences,
                &mut self.ui_edit_context,
                &self.event_proxy,
                ui_state,
            );

            self.state.handle_platform_output(
                window,
                self.ui.context(),
                full_output.platform_output,
            );

            let clipped_primitives: &[ClippedPrimitive] =
                &self.ui.context().tessellate(full_output.shapes);

            let viewport = Viewport {
                width: half_width,
                height: self.size.1,
                ..Default::default()
            };

            let screen_descriptor = ScreenDescriptor {
                size_in_pixels: [viewport.width as u32, viewport.height as u32],
                pixels_per_point: window.scale_factor() as f32,
            };

            self.runtime.render_with(|device, queue, view| {
                for (id, delta) in &full_output.textures_delta.set {
                    self.ui_renderer.update_texture(device, queue, *id, delta);
                }

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("UI Encoder"),
                });

                self.ui_renderer.update_buffers(
                    device,
                    queue,
                    &mut encoder,
                    clipped_primitives,
                    &screen_descriptor,
                );

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

                    self.ui_renderer.render(
                        &mut render_pass,
                        clipped_primitives,
                        &screen_descriptor,
                    );
                }

                queue.submit(Some(encoder.finish()));

                for id in &full_output.textures_delta.free {
                    self.ui_renderer.free_texture(id);
                }

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
        self.runtime.set_wgs_frag(&self.ui_edit_context.frag);
        self.runtime.set_wgs_name(&self.ui_edit_context.name);

        let wgs = self.runtime.wgs();

        if save_as {
            // Save as.
            if let Some(path) = create_file(&format!(
                "{}.{}",
                wgs.name().to_ascii_lowercase().replace(' ', "_"),
                wgs_core::EXTENSION
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
                self.runtime
                    .wgs()
                    .name()
                    .to_ascii_lowercase()
                    .replace(' ', "_"),
                wgs_core::EXTENSION
            ));
        }

        if self.wgs_path.is_some() {
            save_wgs(self.wgs_path.as_ref().unwrap(), wgs);

            self.change_status(AppStatus::Info(fl!("status_save_ok")));

            Some(self.format_title())
        } else {
            None
        }
    }
}

fn load_wgs_from_buffer(buffer: &[u8]) -> io::Result<WgsData> {
    let mut reader = Cursor::new(&buffer);

    Ok(WgsData::load(&mut reader).unwrap())
}

fn load_wgs_from_file<P>(path: P) -> io::Result<WgsData>
where
    P: AsRef<Path>,
{
    let buffer = read(&path)?;

    load_wgs_from_buffer(&buffer)
}

fn on_image_captured(width: u32, height: u32, buffer: Vec<u8>, filename: &str) {
    if let Some(path) = create_file(filename) {
        match image::save_buffer(&path, &buffer, width, height, ColorType::Rgba8) {
            Ok(()) => log::info!("Saving image file: {:?}", path),
            Err(err) => log::error!("Failed to save image: {}", err),
        }
    }
}

fn open_image<P>(path: P) -> Result<(u32, u32, Vec<u8>)>
where
    P: AsRef<Path>,
{
    let image = image::open(path)?;

    let image = image.into_rgba8();

    let width = image.width();
    if width > 2048 {
        bail!("Width larger than 2048");
    }

    let height = image.height();
    if height > 2048 {
        bail!("Height larger than 2048");
    }

    let data = image.into_vec();

    Ok((width, height, data))
}

fn save_wgs<P>(path: P, wgs: &WgsData)
where
    P: AsRef<Path>,
{
    let mut writer = Cursor::new(vec![]);
    wgs.save(&mut writer).unwrap();

    write_file(&path, writer.into_inner());

    log::info!("Saving wgs file: {:?}", path.as_ref());
}
