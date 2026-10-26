//! Interface languages. Strings in the code stay English and are the lookup keys; a catalog maps
//! them to display text when the UI is drawn. Command ids, document text, file names, the control
//! channel, the CLI and MCP never see translated text, so scripts and agents are unaffected. A
//! string without a translation shows in English, so coverage can grow one entry at a time, and a
//! missing translation never fails a build, a test or a release.
//!
//! Catalogs are `i18n/<code>.tsv` in the craft-i18n catalog format (`context <TAB> English <TAB>
//! translation`, `\n` `\t` `\\` escapes, `#` comments), so they can move to storytold/craft-i18n
//! as `locales/<code>/deckcraft.tsv` unchanged. Only rows with an empty context are read here.
//!
//! The `language` setting (`ui.json`) is `auto` by default: the UI follows the operating system's
//! language when there is a catalog for it, and English otherwise. Preferences › Interface
//! language, or the `app.language` command, picks one by hand.
//!
//! # Adding a language
//! Add `i18n/<code>.tsv`, a [`Language`] variant and its rows in [`Language::ALL`],
//! [`Language::code`], [`Language::name`] and [`Language::source`].

use std::cell::Cell;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU8, Ordering};

/// The `language` setting that follows the system.
pub const AUTO: &str = "auto";

/// An interface language.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    En,
    Uk,
    Es,
    Ca,
    Gl,
    Eu,
    It,
    Fr,
    De,
    Pl,
}

impl Language {
    /// Every language, English first: it is the source and the fallback.
    pub const ALL: [Language; 10] =
        [Language::En, Language::Uk, Language::Es, Language::Ca, Language::Gl, Language::Eu, Language::It, Language::Fr, Language::De, Language::Pl];

    /// BCP 47 code, lowercase; also the `language` setting's value.
    pub const fn code(self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Uk => "uk",
            Language::Es => "es",
            Language::Ca => "ca",
            Language::Gl => "gl",
            Language::Eu => "eu",
            Language::It => "it",
            Language::Fr => "fr",
            Language::De => "de",
            Language::Pl => "pl",
        }
    }

    /// The language's name in itself, for the picker.
    pub const fn name(self) -> &'static str {
        match self {
            Language::En => "English",
            Language::Uk => "Українська",
            Language::Es => "Español",
            Language::Ca => "Català",
            Language::Gl => "Galego",
            Language::Eu => "Euskara",
            Language::It => "Italiano",
            Language::Fr => "Français",
            Language::De => "Deutsch",
            Language::Pl => "Polski",
        }
    }

    /// The catalog source; `None` for English, which needs none.
    const fn source(self) -> Option<&'static str> {
        match self {
            Language::En => None,
            Language::Uk => Some(include_str!("i18n/uk.tsv")),
            Language::Es => Some(include_str!("i18n/es.tsv")),
            Language::Ca => Some(include_str!("i18n/ca.tsv")),
            Language::Gl => Some(include_str!("i18n/gl.tsv")),
            Language::Eu => Some(include_str!("i18n/eu.tsv")),
            Language::It => Some(include_str!("i18n/it.tsv")),
            Language::Fr => Some(include_str!("i18n/fr.tsv")),
            Language::De => Some(include_str!("i18n/de.tsv")),
            Language::Pl => Some(include_str!("i18n/pl.tsv")),
        }
    }

    /// The language a BCP 47 or POSIX locale tag (`uk`, `uk-UA`, `uk_UA.UTF-8`, `uk_UA@euro`)
    /// asks for, when there is one.
    pub fn from_tag(tag: &str) -> Option<Language> {
        let primary = tag.split(['-', '_', '.', '@']).next()?.trim().to_ascii_lowercase();
        Language::ALL.into_iter().find(|l| l.code() == primary)
    }

    fn catalog(self) -> Option<&'static Catalog> {
        static UK: LazyLock<Catalog> = LazyLock::new(|| Catalog::parse(Language::Uk.source().unwrap_or_default()));
        static ES: LazyLock<Catalog> = LazyLock::new(|| Catalog::parse(Language::Es.source().unwrap_or_default()));
        static CA: LazyLock<Catalog> = LazyLock::new(|| Catalog::parse(Language::Ca.source().unwrap_or_default()));
        static GL: LazyLock<Catalog> = LazyLock::new(|| Catalog::parse(Language::Gl.source().unwrap_or_default()));
        static EU: LazyLock<Catalog> = LazyLock::new(|| Catalog::parse(Language::Eu.source().unwrap_or_default()));
        static IT: LazyLock<Catalog> = LazyLock::new(|| Catalog::parse(Language::It.source().unwrap_or_default()));
        static FR: LazyLock<Catalog> = LazyLock::new(|| Catalog::parse(Language::Fr.source().unwrap_or_default()));
        static DE: LazyLock<Catalog> = LazyLock::new(|| Catalog::parse(Language::De.source().unwrap_or_default()));
        static PL: LazyLock<Catalog> = LazyLock::new(|| Catalog::parse(Language::Pl.source().unwrap_or_default()));
        match self {
            Language::En => None,
            Language::Uk => Some(&UK),
            Language::Es => Some(&ES),
            Language::Ca => Some(&CA),
            Language::Gl => Some(&GL),
            Language::Eu => Some(&EU),
            Language::It => Some(&IT),
            Language::Fr => Some(&FR),
            Language::De => Some(&DE),
            Language::Pl => Some(&PL),
        }
    }
}

/// The language a `language` setting resolves to: `auto` (or anything unknown) follows the system.
pub fn resolve(pref: &str) -> Language {
    if pref == AUTO { system() } else { Language::from_tag(pref).unwrap_or_else(system) }
}

/// The operating system's interface language, when there is a catalog for it; English otherwise.
/// Tests always get English, whatever the machine's locale.
pub fn system() -> Language {
    #[cfg(any(test, target_arch = "wasm32"))]
    {
        Language::En
    }
    #[cfg(not(any(test, target_arch = "wasm32")))]
    {
        static SYSTEM: LazyLock<Language> =
            LazyLock::new(|| sys_locale::get_locales().take(16).find_map(|tag| Language::from_tag(&tag)).unwrap_or(Language::En));
        *SYSTEM
    }
}

/// A normalised `language` setting: `auto`, or the code of a language there is a catalog for.
pub fn normalize_pref(pref: &str) -> Option<&'static str> {
    if pref.trim().eq_ignore_ascii_case(AUTO) {
        return Some(AUTO);
    }
    Language::from_tag(pref).map(Language::code)
}

thread_local! {
    /// The language the UI thread draws in; the app sets it from its setting every frame.
    static CURRENT: Cell<Language> = const { Cell::new(Language::En) };
}

/// The app's language for text drawn off the UI thread (placeholder prompts in slide renders).
/// Tests leave it English, so a test that switches language cannot change another's renders.
static APP_LANGUAGE: AtomicU8 = AtomicU8::new(0);

pub fn set_current(language: Language) {
    CURRENT.with(|c| c.set(language));
    #[cfg(not(test))]
    APP_LANGUAGE.store(language as u8, Ordering::Relaxed);
}

/// The renderer's placeholder prompt translator ([`deckcraft_render::set_prompt_text`]): the app's
/// language, whichever thread renders.
pub fn prompt_text(english: &'static str) -> &'static str {
    let stored = APP_LANGUAGE.load(Ordering::Relaxed);
    let language = Language::ALL.into_iter().find(|l| *l as u8 == stored).unwrap_or(Language::En);
    tr(language, english)
}

pub fn current() -> Language {
    CURRENT.with(Cell::get)
}

/// `english` in the current language.
pub fn tl(english: &str) -> &str {
    tr(current(), english)
}

/// `english` in `language`: its translation (with the product name rebranded), or `english`
/// itself when there is none.
pub fn tr(language: Language, english: &str) -> &str {
    match language.catalog().and_then(|c| c.rows.get(english)) {
        Some(text) => deckcraft_engine::brand::rebrand_static(text.as_str()),
        None => english,
    }
}

/// Whether `language` has a translation of `english`.
pub fn has(language: Language, english: &str) -> bool {
    language.catalog().is_some_and(|c| c.rows.contains_key(english))
}

/// `template` translated, with each `{name}` replaced by its value. Translators may reorder the
/// placeholders.
pub fn tf(template: &str, args: &[(&str, &str)]) -> String {
    let mut out = tl(template).to_string();
    for (name, value) in args {
        out = out.replace(&format!("{{{name}}}"), value);
    }
    out
}

/// One parsed catalog: plain rows only, keyed by their English.
struct Catalog {
    rows: HashMap<String, String>,
}

impl Catalog {
    /// Lenient: a malformed row (fewer than three columns, an empty translation) is skipped and
    /// that string shows in English; the first of two rows for the same English wins.
    fn parse(source: &str) -> Catalog {
        let mut rows = HashMap::new();
        for line in source.lines() {
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            let mut cols = line.splitn(3, '\t');
            let (Some(context), Some(english), Some(text)) = (cols.next(), cols.next(), cols.next()) else {
                log::warn!("interface catalog: skipped a row without three columns");
                continue;
            };
            let text = unescape(text.trim_end_matches('\r'));
            if !context.is_empty() || english.is_empty() || text.is_empty() {
                continue;
            }
            rows.entry(unescape(english)).or_insert(text);
        }
        Catalog { rows }
    }
}

/// The catalog escapes: `\n`, `\t` and `\\`. Any other backslash stays as it is.
fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
    use super::*;

    #[test]
    fn locale_tags_reduce_to_a_language() {
        for tag in ["uk", "uk-UA", "uk_UA.UTF-8", "UK-ua", "uk_UA@euro"] {
            assert_eq!(Language::from_tag(tag), Some(Language::Uk), "{tag}");
        }
        assert_eq!(Language::from_tag("en-GB"), Some(Language::En));
        assert_eq!(Language::from_tag("es_ES.UTF-8"), Some(Language::Es));
        assert_eq!(Language::from_tag("ca-ES-valencia"), Some(Language::Ca));
        assert_eq!(Language::from_tag("gl_ES.UTF-8"), Some(Language::Gl));
        assert_eq!(Language::from_tag("eu-ES"), Some(Language::Eu));
        assert_eq!(Language::from_tag("it_IT.UTF-8"), Some(Language::It));
        assert_eq!(Language::from_tag("fr-CA"), Some(Language::Fr));
        assert_eq!(Language::from_tag("de_AT"), Some(Language::De));
        assert_eq!(Language::from_tag("pl-PL"), Some(Language::Pl));
        assert_eq!(Language::from_tag("ru-RU"), None);
        assert_eq!(Language::from_tag(""), None);
        assert_eq!(resolve("uk"), Language::Uk);
        assert_eq!(resolve(AUTO), Language::En, "tests see an English system");
        assert_eq!(resolve("xx"), Language::En);
        assert_eq!(normalize_pref(" AUTO "), Some(AUTO));
        assert_eq!(normalize_pref("uk_UA"), Some("uk"));
        assert_eq!(normalize_pref("klingon"), None);
    }

    #[test]
    fn lookups_translate_and_fall_back_to_english() {
        assert_eq!(tr(Language::Uk, "Insert"), "Вставлення");
        assert_eq!(tr(Language::En, "Insert"), "Insert");
        assert_eq!(tr(Language::Es, "Insert"), "Insertar");
        assert_eq!(tr(Language::Ca, "Insert"), "Insereix");
        assert_eq!(tr(Language::Gl, "Insert"), "Inserir");
        assert_eq!(tr(Language::Eu, "Insert"), "Txertatu");
        assert_eq!(tr(Language::Eu, "Slide {n} of {total}"), "{n}/{total} diapositiba");
        assert_eq!(tr(Language::It, "Insert"), "Inserisci");
        assert_eq!(tr(Language::Fr, "Insert"), "Insérer");
        assert_eq!(tr(Language::De, "Slide {n} of {total}"), "Folie {n} von {total}");
        assert_eq!(tr(Language::Pl, "Slide {n} of {total}"), "Slajd {n} z {total}");
        assert_eq!(tr(Language::Ca, "Slide {n} of {total}"), "Diapositiva {n} de {total}");
        assert_eq!(tr(Language::Es, "Slide {n} of {total}"), "Diapositiva {n} de {total}");
        assert_eq!(tr(Language::Uk, "no such string anywhere"), "no such string anywhere");
        set_current(Language::Uk);
        assert_eq!(tl("Home"), "Основне");
        assert_eq!(tf("Slide {n} of {total}", &[("n", "2"), ("total", "9")]), "Слайд 2 з 9");
        set_current(Language::En);
        assert_eq!(tl("Home"), "Home");
        assert_eq!(tf("Slide {n} of {total}", &[("n", "2"), ("total", "9")]), "Slide 2 of 9");
    }

    #[test]
    fn hostile_catalog_rows_are_skipped() {
        let c = Catalog::parse("# comment\n\nonly two\tcolumns\n\tempty\t\nctx\tHome\tX\n\tA\\nB\tА\\nБ\n\tA\\nB\tsecond\n\ttrail\\\tx\\q\\\n");
        assert_eq!(c.rows.get("A\nB").map(String::as_str), Some("А\nБ"));
        assert!(!c.rows.contains_key("Home"), "a context row is not a plain row");
        assert!(!c.rows.contains_key("empty"));
        assert_eq!(c.rows.len(), 2);
    }

    /// Every row of every catalog: three columns, no context, an English key once, a non-empty
    /// translation, the same `{placeholders}` and the same trailing `…` as its English.
    #[test]
    fn catalogs_are_well_formed() {
        for language in Language::ALL {
            let Some(source) = language.source() else { continue };
            let mut seen = std::collections::HashSet::new();
            for (n, line) in source.lines().enumerate() {
                if line.trim().is_empty() || line.starts_with('#') {
                    continue;
                }
                let cols: Vec<&str> = line.split('\t').collect();
                assert_eq!(cols.len(), 3, "{}.tsv:{}: three columns", language.code(), n + 1);
                let (english, text) = (cols[1], cols[2]);
                assert!(cols[0].is_empty(), "{}.tsv:{}: no context", language.code(), n + 1);
                assert!(!text.trim().is_empty(), "{}.tsv:{}: empty translation", language.code(), n + 1);
                assert!(seen.insert(english), "{}.tsv:{}: {english:?} twice", language.code(), n + 1);
                assert_eq!(placeholders(english), placeholders(text), "{}.tsv:{}: placeholders of {english:?}", language.code(), n + 1);
                assert_eq!(english.ends_with('…'), text.ends_with('…'), "{}.tsv:{}: trailing … of {english:?}", language.code(), n + 1);
            }
        }
    }

    fn placeholders(s: &str) -> Vec<&str> {
        let mut v: Vec<&str> = s.split('{').skip(1).filter_map(|p| p.split_once('}').map(|(name, _)| name)).collect();
        v.sort_unstable();
        v
    }
}
