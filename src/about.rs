use crate::{
    egui_winit_wgpu_context::EguiWinitWgpuContext, event::UserEvent, window::WindowExt,
    window_icon::window_icon,
};
use anyhow::Result;
use egui::{pos2, vec2, CentralPanel, Rect};
use raw_window_handle::RawWindowHandle;
use std::env;
use winit::{
    dpi::LogicalSize,
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder, WindowId},
};

const RECOMMAND_HEIGHT: f64 = 200.0;
const RECOMMAND_WIDTH: f64 = 360.0;

pub struct AboutWindow {
    context: EguiWinitWgpuContext,
    window: Window,
}

impl WindowExt<UserEvent> for AboutWindow {
    fn new(
        event_loop: &EventLoopWindowTarget<UserEvent>,
        parent: Option<RawWindowHandle>,
    ) -> Result<Self> {
        let mut builder = WindowBuilder::new()
            .with_title("About")
            .with_inner_size(LogicalSize::new(RECOMMAND_WIDTH, RECOMMAND_HEIGHT))
            .with_resizable(false)
            .with_window_icon(window_icon());

        builder = unsafe { builder.with_parent_window(parent) };

        let window = builder.build(event_loop)?;

        let context = EguiWinitWgpuContext::new(&window, event_loop)?;

        Ok(Self { context, window })
    }

    fn handle_window_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.context.handle_window_event(event)
    }

    fn on_resized(&mut self, width: u32, height: u32) {
        self.context.on_resized(width, height);
    }

    fn on_scaled(&mut self, scale_factor: f32) {
        self.context.on_scaled(scale_factor);
    }

    fn render(&mut self) {
        self.context.render(&self.window, |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                let widget_size = ui.available_size();
                let content_size =
                    widget_size.min(vec2(RECOMMAND_WIDTH as f32, RECOMMAND_HEIGHT as f32));

                let padding = 20.0;

                let rect = Rect::from_center_size(
                    pos2(widget_size.x / 2.0, widget_size.y / 2.0),
                    vec2(
                        content_size.x - padding * 2.0,
                        content_size.y - padding * 2.0,
                    ),
                );

                ui.allocate_ui_at_rect(rect, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("WgShadertoy");
                        ui.strong(env!("CARGO_PKG_VERSION"));
                        ui.strong(
                            option_env!("GIT_COMMIT_HASH").unwrap_or("Git commit hash not found"),
                        );
                    });

                    ui.centered_and_justified(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Homepage: ");
                            ui.hyperlink(env!("CARGO_PKG_HOMEPAGE"));
                        })
                    });
                });
            });
        });
    }

    fn request_redraw(&self) {
        self.window.request_redraw();
    }

    fn window_id(&self) -> WindowId {
        self.window.id()
    }
}
