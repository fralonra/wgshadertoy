use rfd::FileDialog;
use std::{
    fs::write,
    path::{Path, PathBuf},
};

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

pub fn write_file<P, C>(path: P, contents: C)
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    match write(path, contents) {
        Ok(_) => {}
        Err(err) => {
            log::warn!("{}", format!("Failed to write file: {}", err));
        }
    }
}
