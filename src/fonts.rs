use egui::{FontData, FontDefinitions, FontFamily};
use font_kit::{
    family_name::FamilyName, handle::Handle, properties::Properties, source::SystemSource,
};
use std::fs::read;

// Try to load suitable fonts for different scripts.
//
// We have pre-listed some popular fonts, instead of
// programmatically searching for best matches in the system,
// because the latter is more complex and time-consuming.
// Fonts that match more scripts are listed as far forward as possible,
// allowing them to load first to reduce the total number of fonts loaded.
//
// If you find that a certain language cannot be displayed
// correctly on your device, please submit feedback: https://github.com/fralonra/wgshadertoy/issues/7
const FONTS_MAP: [(&str, &[&str]); 4] = [
    (
        "Arabic",
        &[
            "Noto Sans Arabic",
            "Noto Kufi Arabic",
            "Noto Naskh Arabic",
            "Myriad Arabic",
            "Adobe Arabic",
            "Adobe Naskh",
            "Calibri",
            "Tahoma",
            "Cairo",
            "Tajawal",
            "Almarai",
        ],
    ),
    (
        "Han",
        &[
            "Noto Sans CJK SC",
            "Noto Sans CJK TC",
            "Noto Sans CJK HK",
            "Noto Sans CJK JP",
            "Noto Sans CJK KR",
            "Noto Serif CJK SC",
            "Noto Serif CJK TC",
            "Noto Serif CJK HK",
            "Noto Serif CJK JP",
            "Noto Serif CJK KR",
            "WenQuanYi Zen Hei",
            "Source Han Sans CN",
            "Microsoft YaHei",
            "Heiti SC",
            "Heiti TC",
            "PingFang SC",
            "PingFang TC",
            "PingFang HK",
            "FZNewKai",
        ],
    ),
    (
        "Japanese",
        &[
            "Noto Sans CJK SC",
            "Noto Sans CJK TC",
            "Noto Sans CJK HK",
            "Noto Sans CJK JP",
            "Noto Sans CJK KR",
            "Noto Serif CJK SC",
            "Noto Serif CJK TC",
            "Noto Serif CJK HK",
            "Noto Serif CJK JP",
            "Noto Serif CJK KR",
            "WenQuanYi Zen Hei",
            "Noto Sans JP",
            "Noto Serif JP",
            "Source Han Sans CN",
            "Source Han Sans TW",
            "Source Han Sans HK",
            "Source Han Sans JP",
            "Source Han Sans KR",
            "Microsoft YaHei",
            "Heiti SC",
            "Heiti TC",
            "PingFang SC",
            "PingFang TC",
            "PingFang HK",
            "MS Gothic",
            "MS Mincho",
            "Meiryo",
            "Kochi Mincho",
            "Hiragino Sans",
            "Hiragino Mincho Pro",
            "IPAmjMincho",
            "Sazanami Gothic",
            "Sazanami Mincho",
        ],
    ),
    (
        "Korean",
        &[
            "Noto Sans CJK SC",
            "Noto Sans CJK TC",
            "Noto Sans CJK HK",
            "Noto Sans CJK JP",
            "Noto Sans CJK KR",
            "Noto Serif CJK SC",
            "Noto Serif CJK TC",
            "Noto Serif CJK HK",
            "Noto Serif CJK JP",
            "Noto Serif CJK KR",
            "WenQuanYi Zen Hei",
            "Noto Sans KR",
            "Noto Serif KR",
            "Source Han Sans KR",
            "Malgun Gothic",
            "Batang",
            "Gungsuh",
            "Dotum",
            "Gulim",
            "AppleGothic",
            "AppleMyungjo",
            "Nanum Gothic",
            "Nanum Myeongjo",
            "UnBatang",
            "UnGungsuh",
            "Baekmuk Batang",
            "Baekmuk Dotum",
            "Baekmuk Gulim",
        ],
    ),
];

pub fn load_font(fonts: &mut FontDefinitions, font_name: &str, font_data: FontData) {
    let font_name = font_name.to_owned();

    if fonts.font_data.contains_key(&font_name) {
        return;
    }

    if let Some(vec) = fonts.families.get_mut(&FontFamily::Proportional) {
        vec.push(font_name.clone());
    }

    if let Some(vec) = fonts.families.get_mut(&FontFamily::Monospace) {
        vec.push(font_name.clone());
    }

    log::info!("Font loaded: {}", font_name);

    fonts.font_data.insert(font_name, font_data);
}

pub fn load_system_font(font_def: &mut FontDefinitions) {
    let source = SystemSource::new();

    for (script, fonts) in FONTS_MAP {
        let mut script_font_loaded = false;

        if fonts.is_empty() {
            continue;
        }

        for font_name in fonts {
            if font_def.font_data.contains_key(&font_name.to_string()) {
                script_font_loaded = true;

                break;
            }

            if let Ok(handle) = source.select_best_match(
                &[FamilyName::Title(font_name.to_string())],
                &Properties::new(),
            ) {
                let buf = match handle {
                    Handle::Memory { bytes, .. } => Some(bytes.to_vec()),
                    Handle::Path { path, .. } => read(path).ok(),
                };

                if let Some(buf) = buf {
                    load_font(font_def, font_name, FontData::from_owned(buf.to_vec()));

                    script_font_loaded = true;

                    break;
                }
            }
        }

        if !script_font_loaded {
            log::warn!("Failed to load suitable fonts for {} script", script);
        }
    }
}
