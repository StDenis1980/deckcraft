//! What this build calls itself and where it sends people. The release is DeckCraft by the ArtCraft
//! team; a redistributed build of the code (MIT OR Apache-2.0) must not use the ArtCraft marks
//! (`docs/brand/LICENSE-brand.txt`), so it sets its own name and addresses at compile time:
//!
//! ```sh
//! DECKCRAFT_BRAND_NAME="Acme Slides" DECKCRAFT_BRAND_APP_ID=com.example.acmeslides \
//!   DECKCRAFT_BRAND_WEBSITE=https://example.com DECKCRAFT_BRAND_GITHUB=https://github.com/acme/slides \
//!   cargo build --release -p deckcraft
//! ```
//!
//! or once, for every build of a fork, in `.cargo/config.toml`:
//!
//! ```toml
//! [env]
//! DECKCRAFT_BRAND_NAME = "Acme Slides"
//! ```
//!
//! Unset, every value is the release's and nothing changes. Setting a name makes the build
//! [`REBRANDED`]: the ArtCraft logo, Discord and website links are left out, and the name replaces
//! "DeckCraft" in the interface text ([`rebrand`]). Identifiers that files and other programs rely
//! on (command ids, the `deckcraft` MCP server name, `DECKCRAFT_*` variables, `deckcraft-cli`) stay as they are, so
//! documents and scripts work across builds.
//!
//! Standalone: every layer that writes the name (file metadata, PDF export, the shell) reads it from
//! here.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// The release's name, as written in interface text and documents.
pub const UPSTREAM_NAME: &str = "DeckCraft";

/// The release's application id (Wayland app id, Linux desktop entry, macOS bundle id).
pub const UPSTREAM_APP_ID: &str = "ai.storyteller.deckcraft";

/// `option_env!` with a default, usable in a `const`. A blank value counts as unset.
macro_rules! env_or {
    ($var:literal, $default:expr) => {
        match option_env!($var) {
            Some(v) if !v.is_empty() => v,
            _ => $default,
        }
    };
}

/// The product name shown to people and written into documents (`DECKCRAFT_BRAND_NAME`).
pub const NAME: &str = env_or!("DECKCRAFT_BRAND_NAME", UPSTREAM_NAME);

/// The lowercase, space-free name for folders and sockets on Linux and BSD (XDG), such as
/// `~/.config/deckcraft` (`DECKCRAFT_BRAND_SLUG`). Folders elsewhere use [`NAME`]. A rebranded build
/// keeps its settings, recovery files and running instance apart from the release's.
pub const SLUG: &str = env_or!("DECKCRAFT_BRAND_SLUG", "deckcraft");

/// Reverse-DNS application id (`DECKCRAFT_BRAND_APP_ID`).
pub const APP_ID: &str = env_or!("DECKCRAFT_BRAND_APP_ID", UPSTREAM_APP_ID);

/// Who publishes this build, when that is someone other than the DeckCraft contributors, for the
/// About box and the Windows file properties (`DECKCRAFT_BRAND_PUBLISHER`).
pub const PUBLISHER: Option<&str> = match option_env!("DECKCRAFT_BRAND_PUBLISHER") {
    Some(v) if !v.is_empty() => Some(v),
    _ => None,
};

/// The product's web page (`DECKCRAFT_BRAND_WEBSITE`).
pub const APP_PAGE: &str = env_or!("DECKCRAFT_BRAND_WEBSITE", "https://getartcraft.com/apps/deckcraft");

/// The source repository; its GitHub releases are what the update check reads
/// (`DECKCRAFT_BRAND_GITHUB`).
pub const GITHUB: &str = env_or!("DECKCRAFT_BRAND_GITHUB", "https://github.com/storytold/deckcraft");

/// Where people get help (`DECKCRAFT_BRAND_SUPPORT`). The release sends them to the ArtCraft
/// Discord; a rebranded build without its own support page sends them to its web page.
pub const SUPPORT: &str = env_or!("DECKCRAFT_BRAND_SUPPORT", if REBRANDED { APP_PAGE } else { "https://discord.gg/artcraft" });

/// Whether this build carries a name of its own. The ArtCraft marks and community links are only
/// for the release (`docs/brand/LICENSE-brand.txt`).
pub const REBRANDED: bool = match option_env!("DECKCRAFT_BRAND_NAME") {
    Some(v) => !v.is_empty(),
    None => false,
};

/// Where this build's releases are listed: [`GITHUB`]`/releases`.
pub fn releases_page() -> &'static str {
    static PAGE: OnceLock<String> = OnceLock::new();
    PAGE.get_or_init(|| format!("{}/releases", GITHUB.trim_end_matches('/')))
}

/// The GitHub API address of this build's latest release, or `None` when [`GITHUB`] is not a
/// `https://github.com/owner/repo` address (the update check then only opens the releases page).
pub fn latest_release_api() -> Option<String> {
    let repo = GITHUB.strip_prefix("https://github.com/")?.trim_end_matches('/');
    let mut parts = repo.split('/');
    let (Some(owner), Some(name), None) = (parts.next(), parts.next(), parts.next()) else { return None };
    let plain = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    (plain(owner) && plain(name)).then(|| format!("https://api.github.com/repos/{owner}/{name}/releases/latest"))
}

/// `text` with the release's name replaced by this build's. Borrowed, and unchanged, in the
/// release or when the text doesn't name it.
pub fn rebrand(text: &str) -> Cow<'_, str> {
    replace_name(text, NAME)
}

/// [`rebrand`] for text that has to live as long as the program, such as menu labels and dialog
/// titles. Its inputs are the program's own string literals, a fixed set, so each rebranded text is
/// allocated once and kept.
pub fn rebrand_static(text: &'static str) -> &'static str {
    if !REBRANDED || !text.contains(UPSTREAM_NAME) {
        return text;
    }
    static CACHE: OnceLock<Mutex<HashMap<&'static str, &'static str>>> = OnceLock::new();
    let mut cache = CACHE.get_or_init(Default::default).lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    cache.entry(text).or_insert_with(|| Box::leak(rebrand(text).into_owned().into_boxed_str()))
}

fn replace_name<'a>(text: &'a str, name: &str) -> Cow<'a, str> {
    if name == UPSTREAM_NAME || !text.contains(UPSTREAM_NAME) { Cow::Borrowed(text) } else { Cow::Owned(text.replace(UPSTREAM_NAME, name)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_release_keeps_its_name_and_text() {
        assert!(matches!(replace_name("About DeckCraft", UPSTREAM_NAME), Cow::Borrowed("About DeckCraft")));
    }

    #[test]
    fn a_rebranded_build_replaces_every_mention_and_only_those() {
        assert_eq!(replace_name("About DeckCraft", "Acme Slides"), "About Acme Slides");
        assert_eq!(replace_name("DeckCraft {v}: DeckCraft", "Acme"), "Acme {v}: Acme");
        assert!(matches!(replace_name("Open a file", "Acme"), Cow::Borrowed("Open a file")));
        // Other names that contain "Craft" are not this product's.
        assert_eq!(replace_name("ArtCraft and PhotoCraft", "Acme"), "ArtCraft and PhotoCraft");
    }

    #[test]
    fn static_text_is_rebranded_once() {
        let a = rebrand_static("About DeckCraft");
        assert_eq!(a, rebrand("About DeckCraft"));
        assert!(std::ptr::eq(a, rebrand_static("About DeckCraft")), "the same allocation is reused");
        let open = "Open";
        assert!(std::ptr::eq(rebrand_static(open), open), "text without the name is returned as is");
    }

    #[test]
    fn the_constants_agree() {
        assert_eq!(REBRANDED, option_env!("DECKCRAFT_BRAND_NAME").is_some_and(|v| !v.is_empty()));
        assert!(!NAME.is_empty() && !APP_ID.is_empty());
        assert!(APP_PAGE.starts_with("https://") && GITHUB.starts_with("https://") && SUPPORT.starts_with("https://"));
        assert_eq!(releases_page(), format!("{GITHUB}/releases"));
        if !REBRANDED {
            assert_eq!(latest_release_api().as_deref(), Some("https://api.github.com/repos/storytold/deckcraft/releases/latest"));
        }
    }
}
