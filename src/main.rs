#![windows_subsystem = "windows"]

mod core;
mod event;
#[cfg(feature = "fps")]
mod fps_counter;
mod fs;
mod runtime;
mod ui;
mod viewport;
mod wgs;

mod app {
    use crate::{core::Core, event::UserEvent};
    use anyhow::Result;
    use winit::event_loop::{EventLoop, EventLoopBuilder};

    pub struct App {
        core: Core,
        event_loop: EventLoop<UserEvent>,
    }

    impl App {
        pub fn new() -> Result<Self> {
            let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

            let core = Core::new(&event_loop)?;

            Ok(Self { core, event_loop })
        }

        pub fn run(self) {
            self.core.run(self.event_loop);
        }
    }
}

fn main() {
    env_logger::init();

    let app = app::App::new().unwrap();

    app.run();
}
