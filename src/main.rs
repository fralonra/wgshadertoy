#![windows_subsystem = "windows"]

mod about;
mod app;
mod core;
mod egui_winit_wgpu_context;
mod event;
mod example;
#[cfg(feature = "fps")]
mod fps_counter;
mod fs;
mod shortcut;
mod ui;
mod window;
mod window_icon;

fn main() {
    env_logger::init();

    match app::App::new() {
        Ok(app) => app.run(),
        Err(err) => {
            log::error!("Failed to initialize WgShadertoy: {}", err);
        }
    }
}
