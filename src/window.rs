use anyhow::Result;
use raw_window_handle::RawWindowHandle;
use winit::{event::WindowEvent, event_loop::EventLoopWindowTarget, window::WindowId};

pub trait WindowExt<E> {
    fn new(event_loop: &EventLoopWindowTarget<E>, parent: Option<RawWindowHandle>) -> Result<Self>
    where
        Self: Sized;

    fn handle_window_event(&mut self, event: &WindowEvent) -> bool;

    fn on_resized(&mut self, width: u32, height: u32);

    fn on_scaled(&mut self, scale_factor: f32);

    fn render(&mut self);

    fn request_redraw(&self);

    fn window_id(&self) -> WindowId;
}
