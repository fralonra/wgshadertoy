use crate::wgs;
use native_dialog::FileDialog;
use std::{fs::write, path::PathBuf};

pub fn create_file(filename: &str) -> Option<PathBuf> {
    FileDialog::new()
        .set_location("~")
        .add_filter("WebGPU Shader", &[wgs::EXTENSION])
        .set_filename(filename)
        .show_save_single_file()
        .unwrap()
}

pub fn select_file() -> Option<PathBuf> {
    FileDialog::new()
        .set_location("~")
        .add_filter("WebGPU Shader", &[wgs::EXTENSION])
        .show_open_single_file()
        .unwrap()
}

pub fn select_texture() -> Option<PathBuf> {
    FileDialog::new()
        .set_location("~")
        .add_filter("Textures", &["png", "jpg"])
        .show_open_single_file()
        .unwrap()
}

pub fn write_file<C: AsRef<[u8]>>(path: &PathBuf, contents: C) {
    match write(path.as_path(), contents) {
        Ok(_) => {}
        Err(err) => {
            log::warn!("{}", format!("Failed to write file: {}", err));
        }
    }
}
