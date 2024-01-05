#[derive(Clone, Copy, Default, PartialEq)]
pub enum Theme {
    Light,
    #[default]
    Dark,
}

#[derive(Default)]
pub struct Preferences {
    pub record_fps: bool,
    pub theme: Theme,
}
