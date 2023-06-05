use super::TokenType;
use egui::{Color32, Context, FontId, Id, Style, TextFormat};

#[derive(Clone, Hash, PartialEq)]
pub struct CodeTheme {
    dark_mode: bool,
    formats: [TextFormat; TokenType::Total as usize],
}

impl Default for CodeTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl CodeTheme {
    pub fn from_style(style: &Style) -> Self {
        if style.visuals.dark_mode {
            Self::dark()
        } else {
            Self::light()
        }
    }

    pub fn from_memory(ctx: &Context) -> Self {
        if ctx.style().visuals.dark_mode {
            ctx.data_mut(|d| {
                d.get_persisted(egui::Id::new("dark"))
                    .unwrap_or_else(CodeTheme::dark)
            })
        } else {
            ctx.data_mut(|d| {
                d.get_persisted(egui::Id::new("light"))
                    .unwrap_or_else(CodeTheme::light)
            })
        }
    }

    pub fn dark() -> Self {
        let font_id = FontId::monospace(16.0);
        let formats = [
            // Comment: #8b949e
            TextFormat::simple(font_id.clone(), Color32::from_rgb(139, 148, 158)),
            // Function: #d2a8ff
            TextFormat::simple(font_id.clone(), Color32::from_rgb(210, 168, 255)),
            // KeywordOther: #ff7b72
            TextFormat::simple(font_id.clone(), Color32::from_rgb(255, 123, 114)),
            // KeywordType: #ff7b72
            TextFormat::simple(font_id.clone(), Color32::from_rgb(255, 123, 114)),
            // Literal: #c9d1d9
            TextFormat::simple(font_id.clone(), Color32::from_rgb(201, 209, 217)),
            // Numeric: #79c0ff
            TextFormat::simple(font_id.clone(), Color32::from_rgb(121, 192, 255)),
            // Whitespace,
            TextFormat::simple(font_id.clone(), Color32::TRANSPARENT),
        ];

        Self {
            dark_mode: true,
            formats,
        }
    }

    pub fn light() -> Self {
        let font_id = FontId::monospace(16.0);
        let formats = [
            // Comment: #6a737d
            TextFormat::simple(font_id.clone(), Color32::from_rgb(106, 115, 125)),
            // Function #6f42c1
            TextFormat::simple(font_id.clone(), Color32::from_rgb(111, 66, 193)),
            // KeywordOther: #d73a49
            TextFormat::simple(font_id.clone(), Color32::from_rgb(215, 58, 73)),
            // KeywordType: #d73a49
            TextFormat::simple(font_id.clone(), Color32::from_rgb(215, 58, 73)),
            // Literal: #24292e
            TextFormat::simple(font_id.clone(), Color32::from_rgb(36, 41, 46)),
            // Numeric: #005cc5
            TextFormat::simple(font_id.clone(), Color32::from_rgb(0, 92, 197)),
            // Whitespace,
            TextFormat::simple(font_id.clone(), Color32::TRANSPARENT),
        ];

        Self {
            dark_mode: false,
            formats,
        }
    }

    pub fn format(&self, token_type: TokenType) -> TextFormat {
        self.formats[token_type as usize].clone()
    }

    pub fn store_in_memory(self, ctx: &Context) {
        if self.dark_mode {
            ctx.data_mut(|d| d.insert_persisted(egui::Id::new("dark"), self));
        } else {
            ctx.data_mut(|d| d.insert_persisted(egui::Id::new("light"), self));
        }
    }
}
