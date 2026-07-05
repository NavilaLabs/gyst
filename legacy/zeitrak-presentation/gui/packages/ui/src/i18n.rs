use dioxus_i18n::prelude::I18nConfig;
use unic_langid::langid;

/// Build the application-wide i18n configuration with all supported locales.
/// Call this once from the root `App` component via `use_init_i18n(i18n_config)`.
pub fn i18n_config() -> I18nConfig {
    I18nConfig::new(langid!("en-US"))
        .with_locale((langid!("en-US"), include_str!("./locales/en-US.ftl")))
        .with_locale((langid!("de-DE"), include_str!("./locales/de-DE.ftl")))
}
