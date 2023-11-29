use anyhow::Result;
use egui::{FontData, FontDefinitions, FontFamily};
use font_kit::{
    family_name::FamilyName, handle::Handle, properties::Properties, source::SystemSource,
};
use std::fs::read;

pub fn load_font(fonts: &mut FontDefinitions, font_name: &str, font_data: FontData) {
    fonts.font_data.insert(font_name.to_owned(), font_data);

    if let Some(vec) = fonts.families.get_mut(&FontFamily::Proportional) {
        vec.push(font_name.to_owned());
    }

    if let Some(vec) = fonts.families.get_mut(&FontFamily::Monospace) {
        vec.push(font_name.to_owned());
    }
}

pub fn load_system_font(fonts: &mut FontDefinitions) {
    if let Some(font) = match system_font() {
        Ok(font) => Some(font),
        Err(err) => {
            log::warn!("Failed to load system fonts: {}", err);

            None
        }
    } {
        load_font(fonts, "System", font);
    }
}

fn system_font() -> Result<FontData> {
    let handle =
        SystemSource::new().select_best_match(&[FamilyName::SansSerif], &Properties::new())?;

    let buf: Vec<u8> = match handle {
        Handle::Memory { bytes, .. } => bytes.to_vec(),
        Handle::Path { path, .. } => read(path)?,
    };

    Ok(FontData::from_owned(buf))
}
