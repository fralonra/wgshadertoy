#[derive(Debug)]
pub enum AppStatus {
    Info,
    Warning,
    Error,
}

#[derive(Debug)]
pub enum UserEvent {
    ChangeTexture(usize),
    ChangeStatus(Option<(AppStatus, String)>),
    NewFile,
    OpenFile,
    OpenTexture,
    RequestRedraw,
    SaveFile,
}
