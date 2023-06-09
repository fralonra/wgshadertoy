#![windows_subsystem = "windows"]

mod about;
mod app;
mod core;
mod egui_winit_wgpu_context;
mod event;
#[cfg(feature = "fps")]
mod fps_counter;
mod fs;
mod ui;
mod window;

fn main() {
    env_logger::init();

    let app = app::App::new().unwrap();

    app.run();
}
