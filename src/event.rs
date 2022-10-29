use std::path::PathBuf;

#[derive(Debug)]
pub enum UserEvent {
    ChangeTexture(usize),
    NewFile,
    OpenFile,
    OpenTexture,
    RequestRedraw,
    SaveFile,
    SelectFile(PathBuf),
}
