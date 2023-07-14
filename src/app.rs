use crate::{
    about::AboutWindow,
    core::{format_title, Core},
    event::UserEvent,
    window::WindowExt,
    window_icon::window_icon,
};
use anyhow::Result;
use std::collections::HashMap;
use winit::{
    dpi::{LogicalSize, Size},
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    window::{Window, WindowBuilder, WindowId},
};

const RECOMMAND_HEIGHT: f64 = 720.0;
const RECOMMAND_WIDTH: f64 = 1280.0;

const RECOMMAND_SIZE: Size = Size::Logical(LogicalSize::new(RECOMMAND_WIDTH, RECOMMAND_HEIGHT));

pub struct App {
    core: Core,
    event_loop: EventLoop<UserEvent>,
    sub_window_map: HashMap<WindowId, Box<dyn WindowExt<UserEvent>>>,
    window: Window,
}

impl App {
    pub fn new() -> Result<Self> {
        let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

        let window = WindowBuilder::new()
            .with_min_inner_size(Size::Logical(LogicalSize::new(720.0, 360.0)))
            .with_inner_size(RECOMMAND_SIZE)
            .with_title(format_title(&None))
            .with_window_icon(window_icon())
            .build(&event_loop)?;

        try_resize_window(&window);

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
            sub_window_map: HashMap::new(),
            window,
        })
    }

    pub fn run(mut self) {
        self.event_loop.run(move |event, event_loop, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::MainEventsCleared => self.window.request_redraw(),
                Event::RedrawRequested(window_id) => {
                    if window_id == self.window.id() {
                        self.core.redraw(&self.window);
                    } else {
                        if let Some(window) = self.sub_window_map.get_mut(&window_id) {
                            window.render();
                        }
                    }
                }
                Event::WindowEvent {
                    ref event,
                    window_id,
                } => {
                    if window_id == self.window.id() {
                        self.core.handle_window_event(event);
                    } else {
                        if let Some(window) = self.sub_window_map.get_mut(&window_id) {
                            if window.handle_window_event(event) {
                                window.request_redraw();
                            }
                        }
                    }

                    match event {
                        WindowEvent::CloseRequested => {
                            if window_id == self.window.id() {
                                self.sub_window_map.clear();

                                *control_flow = ControlFlow::Exit;
                            } else {
                                self.sub_window_map.remove(&window_id);
                            }
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            if window_id == self.window.id() {
                                self.core
                                    .update_cursor(position.x as f32, position.y as f32)
                            }
                        }
                        WindowEvent::MouseInput { button, state, .. } => {
                            if window_id == self.window.id() {
                                match button {
                                    MouseButton::Left => self
                                        .core
                                        .handle_mouse_input(*state == ElementState::Pressed),
                                    _ => {}
                                }
                            }
                        }
                        WindowEvent::Resized(physical_size) => {
                            if window_id == self.window.id() {
                                self.core.resize(
                                    physical_size.width as f32,
                                    physical_size.height as f32,
                                    self.window.scale_factor() as f32,
                                );
                            } else {
                                if let Some(window) = self.sub_window_map.get_mut(&window_id) {
                                    window.on_resized(physical_size.width, physical_size.height);
                                }
                            }
                        }
                        WindowEvent::ScaleFactorChanged {
                            scale_factor,
                            new_inner_size,
                        } => {
                            if window_id == self.window.id() {
                                try_resize_window(&self.window);

                                self.core.resize(
                                    new_inner_size.width as f32,
                                    new_inner_size.height as f32,
                                    *scale_factor as f32,
                                );
                            } else {
                                if let Some(window) = self.sub_window_map.get_mut(&window_id) {
                                    window.on_scaled(*scale_factor as f32);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Event::UserEvent(event) => {
                    let response = self.core.handle_user_event(event);

                    if response.request_quit {
                        self.sub_window_map.clear();

                        *control_flow = ControlFlow::Exit;

                        return;
                    }

                    if response.request_open_about {
                        if let Ok(sub_window) = AboutWindow::new(event_loop, None) {
                            self.sub_window_map
                                .insert(sub_window.window_id(), Box::new(sub_window));
                        }
                    }

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

fn try_resize_window(window: &Window) {
    if let Some(monitor) = window.current_monitor() {
        let monitor_size = monitor.size();
        let outer_size = window.outer_size();

        if monitor_size.width <= outer_size.width || monitor_size.height <= outer_size.height {
            window.set_maximized(true);
        }
    }
}
