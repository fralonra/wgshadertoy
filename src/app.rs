use crate::{core::Core, event::UserEvent};
use anyhow::Result;
use std::path::PathBuf;
use winit::{
    dpi::{LogicalSize, Size},
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    window::{Icon, Window, WindowBuilder},
};

pub struct App {
    core: Core,
    event_loop: EventLoop<UserEvent>,
    window: Window,
}

impl App {
    pub fn new() -> Result<Self> {
        let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

        let window = WindowBuilder::new()
            .with_min_inner_size(Size::Logical(LogicalSize::new(720.0, 360.0)))
            .with_title(format_title(&None))
            .with_window_icon(window_icon())
            .build(&event_loop)?;

        let inner_size = window.inner_size();

        let core = Core::new(
            &event_loop,
            &window,
            inner_size.width as f32,
            inner_size.height as f32,
            window.scale_factor() as f32,
        )?;

        Ok(Self {
            core,
            event_loop,
            window,
        })
    }

    pub fn run(mut self) {
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::MainEventsCleared => self.window.request_redraw(),
                Event::RedrawRequested(_) => {
                    if self.core.redraw(&self.window) {
                        self.window.request_redraw();
                    }
                }
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.window.id() => {
                    self.core.handle_window_event(event);

                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::CursorMoved { position, .. } => self
                            .core
                            .update_cursor(position.x as f32, position.y as f32),
                        WindowEvent::MouseInput { button, state, .. } => match button {
                            MouseButton::Left => self
                                .core
                                .handle_mouse_input(*state == ElementState::Pressed),
                            _ => {}
                        },
                        WindowEvent::Resized(physical_size) => {
                            self.core
                                .resize(physical_size.width as f32, physical_size.height as f32);
                            self.window.request_redraw();
                        }
                        _ => {}
                    }
                }
                Event::UserEvent(event) => {
                    let response = self.core.handle_user_event(event);

                    if let Some(title) = response.set_title {
                        self.window.set_title(&title);
                    }

                    if response.request_redraw {
                        self.window.request_redraw();
                    }
                }
                _ => {}
            }
        });
    }
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
