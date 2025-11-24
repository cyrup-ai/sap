use chrono::Locale;
use once_cell::sync::OnceCell;
use sys_locale::get_locale;

/// Finds current locale
pub fn current_locale() -> Locale {
    const DEFAULT: Locale = Locale::en_US;
    static CACHE: OnceCell<Locale> = OnceCell::new();

    *CACHE.get_or_init(|| {
        get_locale()
            .as_deref()
            .and_then(|s| {
                let normalized = s.replace('-', "_");
                Locale::try_from(normalized.as_str()).ok()
            })
            .unwrap_or(DEFAULT)
    })
}
