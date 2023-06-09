#[cfg(not(target_os = "macos"))]
use anyhow::Result;
use winit::window::Icon;

#[cfg(target_os = "macos")]
pub fn window_icon() -> Option<Icon> {
    None
}

#[cfg(not(target_os = "macos"))]
pub fn window_icon() -> Option<Icon> {
    match window_icon_from_memory(include_bytes!("../extra/windows/wgshadertoy.ico")) {
        Ok(icon) => Some(icon),
        Err(err) => {
            log::warn!("Failed to load window icon: {}", err);
            None
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn window_icon_from_memory(raw: &[u8]) -> Result<Icon> {
    let image = image::load_from_memory(raw)?;

    let image = image.into_rgba8();

    let (width, height) = image.dimensions();

    let icon = Icon::from_rgba(image.into_raw(), width, height)?;

    Ok(icon)
}
