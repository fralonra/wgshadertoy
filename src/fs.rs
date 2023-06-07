use rfd::FileDialog;
use std::{fs::write, path::PathBuf};

pub fn create_file(filename: &str) -> Option<PathBuf> {
    FileDialog::new()
        .set_directory("~")
        .add_filter("WebGPU Shader", &[wgs_core::EXTENSION])
        .set_file_name(filename)
        .save_file()
}

pub fn select_file() -> Option<PathBuf> {
    FileDialog::new()
        .set_directory("~")
        .add_filter("WebGPU Shader", &[wgs_core::EXTENSION])
        .pick_file()
}

pub fn select_texture() -> Option<PathBuf> {
    FileDialog::new()
        .set_directory("~")
        .add_filter("Textures", &["png", "jpg"])
        .pick_file()
}

pub fn write_file<C: AsRef<[u8]>>(path: &PathBuf, contents: C) {
    match write(path.as_path(), contents) {
        Ok(_) => {}
        Err(err) => {
            log::warn!("{}", format!("Failed to write file: {}", err));
        }
    }
}
