#![windows_subsystem = "windows"]

#[macro_use]
mod macros;

mod about;
mod app;
mod core;
mod egui_winit_wgpu_context;
mod event;
mod example;
mod fonts;
#[cfg(feature = "fps")]
mod fps_counter;
mod fs;
mod i18n;
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
