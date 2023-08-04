#[derive(Debug)]
pub enum Example {
    Default,
    MouseInput,
    Texture,
    TwoTexture,
}

impl Example {
    pub fn data(self) -> &'static [u8] {
        match self {
            Self::Default => include_bytes!("../examples/default.wgs"),
            Self::MouseInput => include_bytes!("../examples/mouse_input.wgs"),
            Self::Texture => include_bytes!("../examples/texture.wgs"),
            Self::TwoTexture => include_bytes!("../examples/two_textures.wgs"),
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::MouseInput => "Mouse Input",
            Self::Texture => "Texture",
            Self::TwoTexture => "Two Textures",
        }
    }
}
