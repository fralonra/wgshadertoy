#[cfg(feature = "fps")]
use crate::fps_counter::FpsCounter;
use crate::{
    event::{AppStatus, UserEvent},
    fs::{create_file, select_file, select_texture, write_file},
    runtime::Runtime,
    ui::{EditContext, Ui},
    viewport::Viewport,
    wgs::{self, WgsData},
};
use anyhow::Result;
use egui_winit::State;
use image::ImageResult;
use std::{
    fs::read,
    io::{self, Cursor},
    path::PathBuf,
    time::Instant,
};
use wgpu::SurfaceError;
use winit::{
    dpi::{PhysicalSize, Size},
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::{Icon, Window, WindowBuilder},
};

const DEFAULT_FRAGMENT: &'static str = include_str!("assets/frag.default.wgsl");
const DEFAULT_VERTEX: &'static str = include_str!("assets/vert.wgsl");

pub struct Core {
    cursor: [f32; 2],
    event_proxy: EventLoopProxy<UserEvent>,
    #[cfg(feature = "fps")]
    fps_counter: FpsCounter,
    has_validation_error: bool,
    status_clock: Instant,
    runtime: Runtime,
    wgs_data: WgsData,
    wgs_path: Option<PathBuf>,
    window: Window,
    ui: Ui,
    ui_edit_context: EditContext,
}

impl Core {
    pub fn new(event_loop: &EventLoop<UserEvent>) -> Result<Self> {
        let wgs_data = default_wgs();

        let event_proxy = event_loop.create_proxy();

        let window = WindowBuilder::new()
            .with_min_inner_size(Size::Physical(PhysicalSize::new(720, 360)))
            .with_title(format_title(&None))
            .with_transparent(true)
            .with_window_icon(window_icon())
            .build(&event_loop)?;

        let textures = wgs_data
            .textures_ref()
            .iter()
            .map(|texture| (texture.width, texture.height, &texture.data))
            .collect();

        let runtime = Runtime::new(
            &window,
            &concat_shader_frag(&wgs_data.frag(), wgs_data.textures_ref().len()),
            DEFAULT_VERTEX,
            textures,
        )?;

        let mut state = State::new(&event_loop);
        state.set_pixels_per_point(window.scale_factor() as f32);

        let mut ui = Ui::new(state);

        for texture in wgs_data.textures_ref() {
            ui.add_texture(texture.width, texture.height, &texture.data);
        }

        let ui_edit_context = EditContext {
            frag: wgs_data.frag(),
            name: wgs_data.name(),
        };

        event_proxy
            .send_event(UserEvent::ChangeStatus(Some((
                AppStatus::Info,
                "Shader compiled successfully!".to_owned(),
            ))))
            .unwrap();

        Ok(Self {
            cursor: Default::default(),
            event_proxy,
            #[cfg(feature = "fps")]
            fps_counter: FpsCounter::new(),
            has_validation_error: false,
            status_clock: Instant::now(),
            runtime,
            wgs_data,
            wgs_path: None,
            window,
            ui,
            ui_edit_context,
        })
    }

    pub fn run(mut self, event_loop: EventLoop<UserEvent>) {
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::MainEventsCleared => self.window.request_redraw(),
                Event::RedrawRequested(_) => {
                    if self.status_clock.elapsed().as_secs() > 5 {
                        self.ui.change_status(None);
                    }

                    if self.runtime.pop_error_scope().is_some() {
                        self.has_validation_error = true;
                        self.event_proxy
                            .send_event(UserEvent::ChangeStatus(Some((
                                AppStatus::Error,
                                "Shader validation error".to_string(),
                            ))))
                            .unwrap();
                        self.status_clock = Instant::now();
                    }

                    if let Err(error) = self.render() {
                        match error.downcast_ref::<SurfaceError>() {
                            Some(SurfaceError::OutOfMemory) => {
                                panic!("Swapchain error: {}. Rendering cannot continue.", error)
                            }
                            Some(_) | None => {
                                log::warn!("Failed to render: {}", error);
                                self.window.request_redraw();
                            }
                        }
                    }

                    #[cfg(feature = "fps")]
                    log::info!("FPS: {}", self.fps_counter.tick());
                }
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.window.id() => {
                    self.ui.handle_event(event);

                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::CursorMoved { position, .. } => {
                            self.cursor = [position.x as f32, position.y as f32];
                            let half_width = self.window.inner_size().width as f32 / 2.0;
                            if position.x as f32 > half_width {
                                self.runtime.update_cursor([
                                    position.x as f32 - half_width,
                                    position.y as f32,
                                ]);
                            }
                        }
                        WindowEvent::MouseInput { button, state, .. } => match button {
                            MouseButton::Left => {
                                let half_width = self.window.inner_size().width as f32 / 2.0;
                                if self.cursor[0] > half_width {
                                    match state {
                                        ElementState::Pressed => self.runtime.update_mouse_press(),
                                        ElementState::Released => {
                                            self.runtime.update_mouse_release()
                                        }
                                    }
                                }
                            }
                            _ => {}
                        },
                        WindowEvent::Resized(physical_size) => {
                            self.runtime
                                .resize(physical_size.width, physical_size.height);
                            self.window.request_redraw();
                        }
                        _ => {}
                    }
                }
                Event::UserEvent(event) => {
                    let mut need_update = false;

                    match event {
                        UserEvent::ChangeStatus(status) => {
                            self.ui.change_status(status);
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
                                    Err(err) => {
                                        log::error!("Failed to open texture: {}", err);
                                    }
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
                            self.window.set_title(&format_title(&self.wgs_path));

                            self.ui.reset_textures();
                            self.ui_edit_context.frag = self.wgs_data.frag();
                            self.ui_edit_context.name = self.wgs_data.name();

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
                                        self.window.set_title(&format_title(&self.wgs_path));

                                        self.ui.reset_textures();
                                        for texture in self.wgs_data.textures_ref() {
                                            self.runtime.add_texture(
                                                texture.width,
                                                texture.height,
                                                &texture.data,
                                            );
                                            self.ui.add_texture(
                                                texture.width,
                                                texture.height,
                                                &texture.data,
                                            );
                                        }

                                        self.ui_edit_context.frag = self.wgs_data.frag();
                                        self.ui_edit_context.name = self.wgs_data.name();

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
                        UserEvent::RequestRedraw => {
                            self.wgs_data.set_frag(&self.ui_edit_context.frag);
                            need_update = true;
                        }
                        UserEvent::SaveFile => {
                            self.wgs_data.set_frag(&self.ui_edit_context.frag);
                            self.wgs_data.set_name(&self.ui_edit_context.name);

                            if self.wgs_path.is_none() {
                                self.wgs_path = create_file(&format!(
                                    "{}.{}",
                                    self.wgs_data.name().to_ascii_lowercase().replace(" ", "_"),
                                    wgs::EXTENSION
                                ));
                                self.window.set_title(&format_title(&self.wgs_path));
                            };
                            if self.wgs_path.is_some() {
                                self.wgs_data.set_frag(&self.ui_edit_context.frag);
                                save_wgs(&self.wgs_path.as_ref().unwrap(), &self.wgs_data);

                                self.event_proxy
                                    .send_event(UserEvent::ChangeStatus(Some((
                                        AppStatus::Info,
                                        "Shader saved successfully!".to_owned(),
                                    ))))
                                    .unwrap();
                                self.status_clock = Instant::now();
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
                            &concat_shader_frag(
                                &self.wgs_data.frag(),
                                self.wgs_data.textures_ref().len(),
                            ),
                            textures,
                        ) {
                            Ok(()) => {
                                self.event_proxy
                                    .send_event(UserEvent::ChangeStatus(Some((
                                        AppStatus::Info,
                                        "Shader compiled successfully!".to_owned(),
                                    ))))
                                    .unwrap();
                                self.status_clock = Instant::now();

                                self.window.request_redraw();
                            }
                            Err(err) => {
                                self.event_proxy
                                    .send_event(UserEvent::ChangeStatus(Some((
                                        AppStatus::Error,
                                        err.to_string(),
                                    ))))
                                    .unwrap();
                                self.status_clock = Instant::now();
                            }
                        }
                    }
                }
                _ => {}
            }
        });
    }

    fn render(&mut self) -> Result<()> {
        let size = self.window.inner_size();
        let half_width = size.width as f32 / 2.0;

        let (surface_texture, texture_view) = self.runtime.get_surface()?;

        if !self.has_validation_error {
            let viewport = Viewport {
                x: half_width,
                width: half_width,
                height: size.height as f32,
                ..Default::default()
            };
            self.runtime.render(&texture_view, &viewport)?;
        }

        {
            let texture_addable =
                self.wgs_data.textures_ref().len() + 1 < self.runtime.max_texture_count() as usize;

            let (clipped_primitives, textures_delta) = self.ui.prepare(
                &self.window,
                &mut self.ui_edit_context,
                texture_addable,
                &self.event_proxy,
            );

            let viewport = Viewport {
                width: half_width,
                height: size.height as f32,
                ..Default::default()
            };
            self.runtime.render_ui(
                &texture_view,
                &clipped_primitives,
                &textures_delta,
                &viewport,
                self.window.scale_factor() as f32,
            )?;
        }

        surface_texture.present();

        Ok(())
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

#[cfg(target_os = "macos")]
fn window_icon() -> Option<Icon> {
    None
}

#[cfg(not(target_os = "macos"))]
fn window_icon() -> Option<Icon> {
    match window_icon_from_memory(include_bytes!("../extra/windows/wgshadertoy.ico")) {
        Ok(icon) => Some(icon),
        Err(err) => {
            log::warn!("Failed to load window icon: {}", err);
            None
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn window_icon_from_memory(raw: &[u8]) -> Result<Icon> {
    let image = image::load_from_memory(raw)?;

    let image = image.into_rgba8();

    let (width, height) = image.dimensions();

    let icon = Icon::from_rgba(image.into_raw(), width, height)?;

    Ok(icon)
}
