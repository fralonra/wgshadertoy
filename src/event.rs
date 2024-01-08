use crate::example::Example;
use std::fmt::Debug;
use winit::event_loop::EventLoopProxy;

#[derive(Clone, Debug)]
#[allow(dead_code)]
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
    OpenAbout,
    OpenExample(Example),
    OpenFile,
    OpenTexture,
    Pause,
    Quit,
    RemoveTexture(usize),
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
    pub request_open_about: bool,
    pub request_quit: bool,
    pub request_redraw: bool,
    pub set_title: Option<String>,
}

pub struct EventProxyWinit<T: 'static> {
    inner: EventLoopProxy<T>,
}

impl<T> EventProxy<T> for EventProxyWinit<T>
where
    T: Debug,
{
    fn send_event(&self, event: T) {
        // Not sure if unwrap is the best idea here, but I don't know enough
        // about the app to say other wise
        //
        // Feel free to replace
        self.inner.send_event(event).unwrap();
    }
}

impl<T> EventProxyWinit<T> {
    pub fn from_proxy(inner: EventLoopProxy<T>) -> Self {
        Self { inner }
    }
}
