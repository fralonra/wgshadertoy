use anyhow::Result;
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    unic_langid::LanguageIdentifier,
    DesktopLanguageRequester, LanguageLoader,
};
use lazy_static::lazy_static;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

lazy_static! {
    pub static ref LANGUAGE_LOADER: FluentLanguageLoader = {
        let loader: FluentLanguageLoader = fluent_language_loader!();

        loader.load_fallback_language(&Localizations).unwrap();

        loader
    };
}

pub struct Language {
    pub id: &'static str,
    pub label: &'static str,
}

pub const LANGUAGES: [Language; 12] = [
    Language {
        id: "ar",
        label: "العربية (Arabic)",
    },
    Language {
        id: "de",
        label: "Deutsch (German)",
    },
    Language {
        id: "en",
        label: "English (English)",
    },
    Language {
        id: "es",
        label: "Español (Spanish)",
    },
    Language {
        id: "fr",
        label: "Français (French)",
    },
    Language {
        id: "it",
        label: "Italiano (Italian)",
    },
    Language {
        id: "ja",
        label: "日本語 (Japanese)",
    },
    Language {
        id: "ko",
        label: "한국어 (Korean)",
    },
    Language {
        id: "pt",
        label: "Português (Portuguese)",
    },
    Language {
        id: "ru",
        label: "Русский (Russian)",
    },
    Language {
        id: "zh_CN",
        label: "简体中文 (Simplified Chinese)",
    },
    Language {
        id: "zh_TW",
        label: "繁體中文 (Traditional Chinese)",
    },
];

pub fn select_locales(request_languages: &[&'static str]) -> Result<()> {
    let requested_languages: Vec<LanguageIdentifier> = request_languages
        .iter()
        .filter_map(|raw| raw.parse().ok())
        .collect();

    i18n_embed::select(&*LANGUAGE_LOADER, &Localizations, &requested_languages)?;

    Ok(())
}

pub fn select_system_locales() -> Result<()> {
    let requested_languages = DesktopLanguageRequester::requested_languages();

    i18n_embed::select(&*LANGUAGE_LOADER, &Localizations, &requested_languages)?;

    Ok(())
}
