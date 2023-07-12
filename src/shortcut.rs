use egui::{Key, KeyboardShortcut, Modifiers};

pub struct Shortcut {
    pub app_quit: KeyboardShortcut,
    pub file_new: KeyboardShortcut,
    pub file_open: KeyboardShortcut,
    pub file_save: KeyboardShortcut,
    pub file_save_as: KeyboardShortcut,
}

impl Shortcut {
    pub fn new() -> Self {
        Self {
            app_quit: KeyboardShortcut::new(Modifiers::CTRL, Key::Q),
            file_new: KeyboardShortcut::new(Modifiers::CTRL, Key::N),
            file_open: KeyboardShortcut::new(Modifiers::CTRL, Key::O),
            file_save: KeyboardShortcut::new(Modifiers::CTRL, Key::S),
            file_save_as: KeyboardShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, Key::S),
        }
    }
}
