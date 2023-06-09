use anyhow::Result;
use egui::{ClippedPrimitive, Context};
use egui_wgpu::{winit::Painter, WgpuConfiguration};
use egui_winit::State;
use winit::{event_loop::EventLoopWindowTarget, window::Window};

pub struct EguiWinitWgpuContext {
    context: Context,
    painter: Painter,
    state: State,
}

impl EguiWinitWgpuContext {
    pub fn new<T>(window: &Window, event_loop: &EventLoopWindowTarget<T>) -> Result<Self> {
        let mut painter = Painter::new(WgpuConfiguration::default(), 1, None, true);

        futures::executor::block_on(painter.set_window(Some(&window)))?;

        let mut state = State::new(&event_loop);
        state.set_pixels_per_point(window.scale_factor() as f32);

        Ok(Self {
            context: Context::default(),
            painter,
            state,
        })
    }

    pub fn handle_window_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.state.on_event(&self.context, event).repaint
    }

    pub fn on_resized(&mut self, width: u32, height: u32) {
        self.painter.on_window_resized(width, height);
    }

    pub fn on_scaled(&mut self, scale_factor: f32) {
        self.state.set_pixels_per_point(scale_factor);
    }

    pub fn render(&mut self, window: &Window, run_ui: impl FnOnce(&Context)) {
        let raw_input = self.state.take_egui_input(window);

        let full_output = self.context.run(raw_input, |ctx| {
            run_ui(ctx);
        });

        self.state
            .handle_platform_output(window, &self.context, full_output.platform_output);

        let clipped_primitives: &[ClippedPrimitive] = &self.context.tessellate(full_output.shapes);

        self.painter.paint_and_update_textures(
            window.scale_factor() as f32,
            egui::Color32::BLACK.to_normalized_gamma_f32(),
            clipped_primitives,
            &full_output.textures_delta,
            false,
        );
    }
}
