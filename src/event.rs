use winit::event_loop::EventLoopProxy;

#[derive(Clone, Debug)]
pub enum AppStatus {
    Idle,
    Info(String),
    Warning(String),
    Error(String),
}

#[derive(Debug)]
pub enum UserEvent {
    CaptureImage,
    ChangeTexture(usize),
    NewFile,
    OpenFile,
    OpenTexture,
    Pause,
    RequestRedraw,
    Restart,
    Resume,
    SaveFile,
    SaveFileAs,
}

pub trait EventProxy<T> {
    fn send_event(&self, event: T);
}

#[derive(Debug, Default)]
pub struct AppResponse {
    pub request_redraw: bool,
    pub set_title: Option<String>,
}

pub struct EventProxyWinit<T: 'static> {
    inner: EventLoopProxy<T>,
}

impl<T> EventProxy<T> for EventProxyWinit<T> {
    fn send_event(&self, event: T) {
        self.inner.send_event(event);
    }
}

impl<T> EventProxyWinit<T> {
    pub fn from_proxy(inner: EventLoopProxy<T>) -> Self {
        Self { inner }
    }
}
