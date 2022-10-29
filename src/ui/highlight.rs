mod code_theme;
mod token_type;

pub use code_theme::CodeTheme;
use egui::{text::LayoutJob, util::cache};
use token_type::TokenType;

#[derive(Default)]
pub struct Highlighter {}

impl cache::ComputerMut<(&CodeTheme, &str), LayoutJob> for Highlighter {
    fn compute(&mut self, (theme, code): (&CodeTheme, &str)) -> LayoutJob {
        self.highlight(theme, code)
    }
}

impl Highlighter {
    pub fn highlight(&self, theme: &CodeTheme, mut text: &str) -> LayoutJob {
        let mut job = LayoutJob::default();
        let mut is_function_declaration = false;
        let mut is_inside_angle_bracket = false;

        while !text.is_empty() {
            if text.starts_with("//") {
                let end = text.find("\n").unwrap_or(text.len());
                job.append(&text[..end], 0.0, theme.format(TokenType::Comment));
                text = &text[end..];
            } else if text.starts_with("/*") {
                if text.contains("*/") {
                    let end = text.find("*/").unwrap_or(text.len() - 2);
                    let block = &text[..end + 2];
                    job.append(block, 0.0, theme.format(TokenType::Comment));
                    text = &text[end + 2..];
                } else {
                    job.append(&text[..text.len()], 0.0, theme.format(TokenType::Comment));
                    text = &text[text.len()..];
                }
            } else if text.starts_with(|c: char| c.is_ascii_digit()) {
                let end = text[1..]
                    .find(|c: char| !c.is_ascii_digit())
                    .map_or_else(|| text.len(), |i| i + 1);
                let numeric = &text[..end];

                job.append(numeric, 0.0, theme.format(TokenType::Numeric));
                text = &text[end..];
            } else if text.starts_with(|c: char| c.is_ascii_whitespace()) {
                let end = text[1..]
                    .find(|c: char| !c.is_ascii_whitespace())
                    .map_or_else(|| text.len(), |i| i + 1);
                job.append(&text[..end], 0.0, theme.format(TokenType::Whitespace));
                text = &text[end..];
            } else if text.starts_with("@") {
                job.append("@", 0.0, theme.format(TokenType::Literal));
                text = &text[1..];
                let end = text[1..]
                    .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .map_or_else(|| text.len(), |i| i + 1);
                let word = &text[..end];
                job.append(word, 0.0, theme.format(TokenType::KeywordType));
                text = &text[end..];
            } else if text.starts_with("<") {
                job.append("<", 0.0, theme.format(TokenType::Literal));
                text = &text[1..];
                is_inside_angle_bracket = true;
            } else if text.starts_with(|c: char| c.is_ascii_alphabetic()) {
                let end = text[1..]
                    .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .map_or_else(|| text.len(), |i| i + 1);
                let word = &text[..end];
                let token_type = if is_function_declaration {
                    is_function_declaration = false;
                    TokenType::FunctionDefinition
                } else if is_inside_angle_bracket {
                    is_inside_angle_bracket = false;
                    TokenType::KeywordType
                } else if is_keyword_other(word) {
                    if word == "fn" {
                        is_function_declaration = true;
                    }
                    TokenType::KeywordOther
                } else if is_keyword_type(word) {
                    TokenType::KeywordType
                } else {
                    TokenType::Literal
                };
                job.append(word, 0.0, theme.format(token_type));
                text = &text[end..];
            } else {
                let mut it = text.char_indices();
                it.next();
                let end = it.next().map_or(text.len(), |(idx, _chr)| idx);
                job.append(&text[..end], 0.0, theme.format(TokenType::Literal));
                text = &text[end..];
            }
        }

        job
    }
}

fn is_keyword_other(word: &str) -> bool {
    matches!(
        word,
        // https://www.w3.org/TR/WGSL/#keyword-summary
        // Other Keywords
        |"bitcast"| "break"
            | "case"
            | "const"
            | "continue"
            | "continuing"
            | "default"
            | "discard"
            | "else"
            | "enable"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "let"
            | "loop"
            | "override"
            | "return"
            | "static_assert"
            | "struct"
            | "switch"
            | "true"
            | "type"
            | "var"
            | "while"
    )
}

fn is_keyword_type(word: &str) -> bool {
    matches!(
        word,
        // https://www.w3.org/TR/WGSL/#keyword-summary
        // Type-defining Keywords
        "array"
            | "atomic"
            | "bool"
            | "f32"
            | "f16"
            | "i32"
            | "mat2x2"
            | "mat2x3"
            | "mat2x4"
            | "mat3x2"
            | "mat3x3"
            | "mat3x4"
            | "mat4x2"
            | "mat4x3"
            | "mat4x4"
            | "ptr"
            | "sampler"
            | "sampler_comparison"
            | "texture_1d"
            | "texture_2d"
            | "texture_2d_array"
            | "texture_3d"
            | "texture_cube"
            | "texture_cube_array"
            | "texture_multisampled_2d"
            | "texture_storage_1d"
            | "texture_storage_2d"
            | "texture_storage_2d_array"
            | "texture_storage_3d"
            | "texture_depth_2d"
            | "texture_depth_2d_array"
            | "texture_depth_cube"
            | "texture_depth_cube_array"
            | "texture_depth_multisampled_2d"
            | "u32"
            | "vec2"
            | "vec3"
            | "vec4"
    )
}
