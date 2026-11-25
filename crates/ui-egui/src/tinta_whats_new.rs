//! Tinta Office: the "What's new in Preview 4" window. The desktop app turns it on with its
//! settings folder ([`enable`]); it then opens by itself once, the first time this release starts
//! (a small file in that folder remembers it, [`SEEN_FILE`]), and again whenever the What's new
//! command asks ([`request`]). Close, Esc or a click outside closes it. Without [`enable`]
//! (upstream's builds, tests, the web, the command line) it never opens, and the command says so.
//!
//! The same file lives in all five Tinta apps, each with its own list, written from that app's
//! release notes (`docs/tinta-releases/preview-4.md`). Its strings are translated here, not in the
//! shared catalogues, so syncing with upstream never conflicts: the list in Spanish and Ukrainian,
//! as the notes are (Catalan, Galician and Basque read the Spanish list), the title and buttons in
//! every interface language Tinta ships; anything else reads English.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use egui::{Id, RichText};

/// The release this list describes. A later release's list opens by itself once again.
pub const RELEASE: &str = "preview-4";
/// In the settings folder: the release whose list already opened by itself.
pub const SEEN_FILE: &str = "tinta-whats-new.txt";
/// The app the list is about.
const APP: &str = "Tinta Presenta";
const MAX_WIDTH: f32 = 520.0;
/// The list scrolls past this share of the window's height.
const MAX_HEIGHT_SHARE: f32 = 0.6;

/// Set by [`enable`]: the settings folder, or `None` when this run keeps no settings (the window
/// still opens on request, but never by itself).
static ENABLED: OnceLock<Option<PathBuf>> = OnceLock::new();
/// Asked for (by the command, or due at start) without a `Context` at hand: opens on the next frame.
static REQUESTED: AtomicBool = AtomicBool::new(false);

/// The desktop app: turn the window on. It opens on the first frame when this release's list has
/// not opened by itself before in `settings`.
pub fn enable(settings: Option<PathBuf>) {
    let due = settings.as_deref().is_some_and(|dir| !seen(dir));
    if ENABLED.set(settings).is_ok() && due {
        REQUESTED.store(true, Ordering::Relaxed);
    }
}

/// Whether this build shows the window (the desktop app turned it on).
pub fn available() -> bool {
    ENABLED.get().is_some()
}

/// The What's new command: open the window on the next frame. `false` when this build has no
/// such window (nothing opens).
pub fn request() -> bool {
    let on = available();
    if on {
        REQUESTED.store(true, Ordering::Relaxed);
    }
    on
}

/// Whether this release's list already opened by itself with these settings.
pub fn seen(settings: &Path) -> bool {
    std::fs::read_to_string(settings.join(SEEN_FILE)).is_ok_and(|s| s.trim() == RELEASE)
}

/// Remember that this release's list opened; `false` when the file could not be written (the
/// list then opens again next start).
pub fn remember(settings: &Path) -> bool {
    std::fs::create_dir_all(settings).and_then(|()| std::fs::write(settings.join(SEEN_FILE), RELEASE)).is_ok()
}

/// Each frame, after the window's own dialogs: opens what was asked for and draws the window.
pub fn show(ctx: &egui::Context, lang: &str) {
    if REQUESTED.swap(false, Ordering::Relaxed) {
        open(ctx);
        if let Some(Some(dir)) = ENABLED.get() {
            let _ = remember(dir);
        }
    }
    draw(ctx, lang);
}

fn id() -> Id {
    Id::unique("tinta.whatsNew")
}

/// Open the window in `ctx` (its Close button takes the keyboard focus).
pub fn open(ctx: &egui::Context) {
    ctx.data_mut(|d| d.insert_temp(id(), true));
}

pub fn is_open(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(id())).is_some()
}

/// Draw the window when it is open: the list, and Close.
pub fn draw(ctx: &egui::Context, lang: &str) {
    // `true` until Close has taken the focus once.
    let Some(focus) = ctx.data(|d| d.get_temp::<bool>(id())) else {
        return;
    };
    let list = list(lang);
    let max_height = (ctx.content_rect().height() * MAX_HEIGHT_SHARE).max(160.0);
    let modal = egui::Modal::new(id().with("modal")).show(ctx, |ui| {
        ui.set_max_width(MAX_WIDTH);
        ui.label(RichText::new(APP).weak());
        ui.heading(tr(lang, TITLE));
        ui.add_space(4.0);
        ui.label(list.intro);
        ui.add_space(8.0);
        egui::ScrollArea::vertical().max_height(max_height).auto_shrink([false, true]).show(ui, |ui| {
            for (title, text) in list.items {
                ui.label(RichText::new(*title).strong());
                ui.label(*text);
                ui.add_space(6.0);
            }
        });
        ui.add_space(8.0);
        let close = ui.button(tr(lang, CLOSE));
        if focus {
            close.request_focus();
        }
        close.clicked()
    });
    if modal.inner || modal.should_close() {
        ctx.data_mut(|d| d.remove::<bool>(id()));
    } else if focus {
        ctx.data_mut(|d| d.insert_temp(id(), false));
    }
}

/// What's new: one release's highlights in one language.
struct List {
    /// What kind of version this is.
    intro: &'static str,
    /// Each change: a short title and a sentence or two.
    items: &'static [(&'static str, &'static str)],
}

/// The list in `lang`: Spanish also for Catalan, Galician and Basque, English when there is none.
fn list(lang: &str) -> &'static List {
    match base(lang).as_str() {
        "es" | "ca" | "gl" | "eu" => &ES,
        "uk" => &UK,
        _ => &EN,
    }
}

/// `es` for `es-ES`, `ca` for `ca-valencia`, `pt` for `pt_BR`.
fn base(lang: &str) -> String {
    lang.split(['-', '_']).next().unwrap_or(lang).to_ascii_lowercase()
}

/// English: the source.
const EN: List = List {
    intro: "A test version: work on copies of your presentations.",
    items: &[
        (
            "Tinta templates",
            "Four starter decks with azulejo tiles (talk, report, client proposal and workshop) on Tinta's own themes, in File ▸ New.",
        ),
        ("SVG pictures", "SVG icons and drawings show and are kept when you save, and you can insert .svg files."),
        ("Video and audio", "Playback options (volume, loop, full screen, start automatically…) are kept in .pptx files."),
        ("Videos this version can't play", "Convert H.264 or HEVC videos to WebM with your own FFmpeg, without leaving the presentation."),
        (
            "Closer to the original",
            "Diagram text and chart legends are kept; copying, duplicating and ungrouping shapes inside groups works as expected.",
        ),
        (
            "Tinta Office look",
            "Tinta Correo's colours and control shapes, light and dark; a button on the View tab switches back to the classic look.",
        ),
    ],
};

/// Spanish (also read in Catalan, Galician and Basque).
const ES: List = List {
    intro: "Versión de prueba: úsela con copias de sus presentaciones.",
    items: &[
        (
            "Plantillas de Tinta",
            "Cuatro presentaciones de inicio con azulejos (charla, informe, propuesta para un cliente y taller), con temas propios de Tinta, en Archivo ▸ Nuevo.",
        ),
        ("Imágenes SVG", "Los iconos y dibujos SVG se ven y se conservan al guardar, y puede insertar archivos .svg."),
        (
            "Vídeo y audio",
            "Las opciones de reproducción (volumen, repetir, pantalla completa, inicio automático…) se conservan en los archivos .pptx.",
        ),
        ("Vídeos que esta versión no reproduce", "Convierta los vídeos H.264 o HEVC a WebM con su propio FFmpeg, sin salir de la presentación."),
        (
            "Más fiel al original",
            "El texto de los diagramas y las leyendas de los gráficos se conservan; copiar, duplicar y desagrupar formas dentro de grupos funciona como se espera.",
        ),
        (
            "Aspecto de Tinta Office",
            "Los colores y la forma de los controles de Tinta Correo, en claro y oscuro; un botón en la pestaña Vista vuelve al aspecto clásico.",
        ),
    ],
};

/// Ukrainian.
const UK: List = List {
    intro: "Тестова версія: працюйте з копіями презентацій.",
    items: &[
        (
            "Шаблони Tinta",
            "Чотири готові презентації з азулежу (виступ, звіт, пропозиція клієнтові й майстер-клас) з власними темами Tinta, у «Файл ▸ Створити».",
        ),
        ("Зображення SVG", "SVG-піктограми й малюнки показуються і зберігаються, а файли .svg можна вставляти."),
        ("Відео й аудіо", "Параметри відтворення (гучність, повтор, на весь екран, автозапуск…) зберігаються у файлах .pptx."),
        ("Відео, яке ця версія не відтворює", "Перетворіть відео H.264 чи HEVC на WebM власним FFmpeg, не виходячи з презентації."),
        ("Ближче до оригіналу", "Текст схем і легенди діаграм зберігаються; копіювання, дублювання й розгрупування фігур у групах працюють як слід."),
        (
            "Вигляд Tinta Office",
            "Кольори й форма елементів як у Tinta Correo, у світлій і темній темі; кнопка на вкладці «Вигляд» повертає класичний вигляд.",
        ),
    ],
};

pub const TITLE: &str = "What's new in Preview 4";
pub const CLOSE: &str = "Close";
/// The command's name in menus and on buttons.
pub const MENU: &str = "What's new";

/// The strings above, in the order of each row of [`TRANSLATIONS`].
const STRINGS: [&str; 3] = [TITLE, CLOSE, MENU];

/// `text` (one of the constants above) in the language `lang` (`es`, `ca-valencia`, `pt_BR`…),
/// English when that language has no translation.
pub fn tr(lang: &str, text: &'static str) -> &'static str {
    let lang = base(lang);
    let Some(i) = STRINGS.iter().position(|s| *s == text) else {
        return text;
    };
    TRANSLATIONS.iter().find(|(code, _)| *code == lang).and_then(|(_, t)| t.get(i).copied()).unwrap_or(text)
}

/// Each language's translations, in the order of [`STRINGS`].
const TRANSLATIONS: &[(&str, [&str; 3])] = &[
    ("es", ["Novedades de la vista previa 4", "Cerrar", "Novedades"]),
    ("ca", ["Novetats de la vista prèvia 4", "Tanca", "Novetats"]),
    ("gl", ["Novidades da vista previa 4", "Pechar", "Novidades"]),
    ("eu", ["4. aurrebistako berrikuntzak", "Itxi", "Berrikuntzak"]),
    ("uk", ["Що нового в попередній версії 4", "Закрити", "Що нового"]),
    ("it", ["Novità dell'anteprima 4", "Chiudi", "Novità"]),
    ("fr", ["Nouveautés de la préversion 4", "Fermer", "Nouveautés"]),
    ("de", ["Neu in Vorschau 4", "Schließen", "Neuigkeiten"]),
    ("pl", ["Co nowego w wersji zapoznawczej 4", "Zamknij", "Co nowego"]),
    ("ro", ["Noutăți în versiunea de previzualizare 4", "Închide", "Noutăți"]),
    ("cs", ["Novinky v náhledové verzi 4", "Zavřít", "Novinky"]),
    ("lt", ["Kas naujo 4-ojoje peržiūros versijoje", "Uždaryti", "Kas naujo"]),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(ctx: &egui::Context, events: Vec<egui::Event>, lang: &str) {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1024.0, 768.0))),
            events,
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| draw(ui.ctx(), lang));
        out.textures_delta.clear();
    }

    fn escape() -> egui::Event {
        egui::Event::Key { key: egui::Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE }
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tinta-whats-new-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn the_list_opens_by_itself_once_per_release() {
        let dir = temp_dir("seen");
        assert!(!seen(&dir));
        assert!(remember(&dir));
        assert!(seen(&dir));
        // An older release's mark: the new list is due again.
        assert!(std::fs::write(dir.join(SEEN_FILE), "preview-3").is_ok());
        assert!(!seen(&dir));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn escape_closes_the_window() {
        let ctx = egui::Context::default();
        frame(&ctx, vec![], "en");
        assert!(!is_open(&ctx));
        open(&ctx);
        frame(&ctx, vec![], "en");
        frame(&ctx, vec![], "en");
        assert!(is_open(&ctx));
        frame(&ctx, vec![escape()], "en");
        assert!(!is_open(&ctx));
    }

    #[test]
    fn the_window_draws_in_every_language() {
        for lang in ["en", "es", "ca-valencia", "gl", "eu", "uk", "it", "fr", "de", "pl", "ro", "cs", "lt", "pt_BR"] {
            let ctx = egui::Context::default();
            open(&ctx);
            frame(&ctx, vec![], lang);
            frame(&ctx, vec![], lang);
            assert!(is_open(&ctx), "{lang}");
        }
    }

    #[test]
    fn every_list_has_every_item() {
        for list in [&ES, &UK] {
            assert_eq!(list.items.len(), EN.items.len());
            assert!(!list.intro.is_empty());
            assert!(list.items.iter().all(|(title, text)| !title.is_empty() && !text.is_empty()));
        }
        assert!(!std::ptr::eq(list("ca-valencia"), list("en")));
        assert!(std::ptr::eq(list("ca-valencia"), list("es")));
        assert!(std::ptr::eq(list("pt-BR"), list("en")));
    }

    #[test]
    fn every_string_is_translated() {
        for (lang, t) in TRANSLATIONS {
            for (en, tr) in STRINGS.iter().zip(t) {
                assert!(!tr.is_empty(), "{lang}: {en}");
            }
        }
        assert_ne!(tr("uk", CLOSE), CLOSE);
        assert_eq!(tr("pt-BR", CLOSE), CLOSE);
    }
}
