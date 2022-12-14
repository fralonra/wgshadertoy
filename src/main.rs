mod app;
mod context;
mod error;
mod event;
#[cfg(feature = "fps")]
mod fps_counter;
mod fs;
mod runtime;
mod ui;
mod viewport;
mod wgs;

fn main() {
    env_logger::init();

    let app = app::App::new().unwrap();

    app.run();
}
