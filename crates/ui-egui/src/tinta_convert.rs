//! Tinta Presenta: converting a clip this build can't play (H.264, HEVC, AAC, WMV…) to WebM with
//! the FFmpeg installed on the computer (`deckcraft_media::transcode`).
//!
//! Inserting such a clip leaves a request from the engine (`tinta_convert::OFFER`); the app then
//! asks whether to convert it ("Always convert without asking" turns the question off), and the
//! link under the media bar of a clip that doesn't play converts it on demand. The conversion runs
//! in the background with a progress window and Cancel; when it ends, `media.replace` puts the
//! WebM in the clip's place as one undo step. Without FFmpeg the app says how to install it for
//! this system and lets the person choose the program. Only desktop builds that set
//! [`crate::Services::convert_media`] offer any of this.
//!
//! The strings are translated here, not in the shared catalogues, so syncing with upstream never
//! conflicts.

use std::collections::VecDeque;

use deckcraft_model::{MediaId, ShapeId, SlideId};
use egui::{RichText, Ui};
use serde_json::json;

use crate::SlideApp;

#[cfg(not(target_arch = "wasm32"))]
use deckcraft_media::transcode::{self, Ffmpeg, Job, Outcome};

#[cfg(target_arch = "wasm32")]
mod transcode {
    //! The web build has no processes: nothing is ever found or started.
    #[derive(Clone, Debug)]
    pub struct Ffmpeg;
    // The same shape as the desktop's, so the code that matches on it builds; nothing makes one.
    #[allow(dead_code)]
    #[derive(Clone, Debug)]
    pub enum Outcome {
        Done(std::path::PathBuf),
        Failed(String),
        Cancelled,
    }
    pub struct Job;
    impl Job {
        pub fn start(_: &Ffmpeg, _: &[u8], _: bool, _: u64) -> Result<Job, String> {
            Err("converting needs the desktop app".into())
        }
        pub fn progress(&self) -> f32 {
            0.0
        }
        pub fn cancel(&self) {}
        pub fn outcome(&self) -> Option<Outcome> {
            Some(Outcome::Cancelled)
        }
    }
    pub fn find(_: Option<&std::path::Path>) -> Result<Ffmpeg, String> {
        Err("converting needs the desktop app".into())
    }
    pub fn check(p: &std::path::Path) -> Result<Ffmpeg, String> {
        find(Some(p))
    }
}
#[cfg(target_arch = "wasm32")]
use transcode::{Ffmpeg, Job, Outcome};

/// The clip being offered or converted: where it is, and which media it had then (a clip whose
/// media changed in the meantime is left alone).
#[derive(Clone, Debug, PartialEq)]
struct Clip {
    doc: u64,
    slide: SlideId,
    shape: ShapeId,
    media: MediaId,
    name: String,
    codec: String,
    video: bool,
    duration_ms: u64,
}

#[derive(Clone, Debug, PartialEq)]
enum Dialog {
    /// "Convert this clip to WebM?"
    Offer(Clip),
    /// No usable FFmpeg; the clip to convert once there is one.
    Missing {
        clip: Clip,
        error: Option<String>,
    },
    Failed {
        name: String,
        error: String,
    },
}

struct Running {
    clip: Clip,
    job: Job,
    /// The converted file, kept while the clip's presentation isn't the active one.
    done: Option<std::path::PathBuf>,
}

/// The conversion state of one app window.
#[derive(Default)]
pub struct Converter {
    queue: VecDeque<Clip>,
    dialog: Option<Dialog>,
    running: Option<Running>,
    /// The offer dialog's "Always convert without asking" box.
    always: bool,
    /// The FFmpeg last found, with the setting it was found for.
    found: Option<(String, Ffmpeg)>,
}

impl std::fmt::Debug for Converter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Converter").field("queue", &self.queue.len()).field("dialog", &self.dialog).field("running", &self.running.is_some()).finish()
    }
}

impl Converter {
    /// Whether a dialog or a conversion is showing.
    pub fn is_busy(&self) -> bool {
        self.dialog.is_some() || self.running.is_some()
    }
    /// How far the running conversion is (0–1), if one runs.
    pub fn progress(&self) -> Option<f32> {
        self.running.as_ref().map(|r| r.job.progress())
    }
}

/// The clip shape `shape` on the current slide of the active presentation.
fn clip_at(app: &SlideApp, shape: ShapeId) -> Option<Clip> {
    let d = app.session.active()?;
    let index = d.selection.slide;
    let slide = d.doc.slides.get(index)?.id;
    let clip = crate::media::clip_on(&d.doc, index, shape)?;
    let item = d.doc.media(clip.media)?;
    let info = deckcraft_media::probe_headers(&item.data).ok();
    let codec = info
        .as_ref()
        .map(|i| {
            let mut c: Vec<&str> = i.video.iter().filter(|v| !v.decodable).map(|v| v.codec.as_str()).collect();
            c.extend(i.audio.iter().filter(|a| !a.decodable).map(|a| a.codec.as_str()));
            c.join(" / ")
        })
        .filter(|c| !c.is_empty())
        .unwrap_or_else(|| item.content_type.clone());
    let duration_ms = info.map(|i| i.duration_ms).filter(|d| *d > 0).unwrap_or(u64::from(clip.duration_ms));
    Some(Clip { doc: d.uid, slide, shape, media: clip.media, name: item.name.clone(), codec, video: clip.video, duration_ms })
}

/// The engine asked to offer converting `shape` (a clip just inserted that doesn't play).
pub(crate) fn offer(app: &mut SlideApp, shape: ShapeId) {
    if !app.services.convert_media {
        return;
    }
    let Some(clip) = clip_at(app, shape) else { return };
    if app.session.prefs.tinta_convert.ask {
        app.tinta_convert.queue.push_back(clip);
    } else {
        start(app, clip);
    }
}

/// The person asked to convert `shape` on the current slide (the link under its media bar).
pub(crate) fn convert_now(app: &mut SlideApp, shape: ShapeId) {
    if let Some(clip) = clip_at(app, shape) {
        start(app, clip);
    }
}

/// Whether the app can convert clips (shows the link under the media bar).
pub(crate) fn available(app: &SlideApp) -> bool {
    app.services.convert_media
}

/// The FFmpeg to use, found once per setting.
fn ffmpeg(app: &mut SlideApp) -> Result<Ffmpeg, String> {
    let setting = app.session.prefs.tinta_convert.ffmpeg_path.clone();
    if let Some((s, f)) = &app.tinta_convert.found
        && *s == setting
    {
        return Ok(f.clone());
    }
    let configured = deckcraft_engine::tinta_convert::configured_ffmpeg(&app.session.prefs.tinta_convert);
    let f = transcode::find(configured.as_deref()).map_err(|e| e.to_string())?;
    app.tinta_convert.found = Some((setting, f.clone()));
    Ok(f)
}

fn start(app: &mut SlideApp, clip: Clip) {
    if app.tinta_convert.running.is_some() {
        if !app.tinta_convert.queue.contains(&clip) {
            app.set_status(tr_name(QUEUED, &clip.name));
            app.tinta_convert.queue.push_back(clip);
        }
        return;
    }
    let ff = match ffmpeg(app) {
        Ok(f) => f,
        Err(e) => {
            let error = (e != "FFmpeg was not found").then_some(e);
            app.tinta_convert.dialog = Some(Dialog::Missing { clip, error });
            return;
        }
    };
    let Some(data) = app.session.documents().iter().find(|d| d.uid == clip.doc).and_then(|d| d.doc.media(clip.media)).map(|m| m.data.clone()) else {
        return;
    };
    match Job::start(&ff, &data, clip.video, clip.duration_ms) {
        Ok(job) => app.tinta_convert.running = Some(Running { clip, job, done: None }),
        Err(error) => app.tinta_convert.dialog = Some(Dialog::Failed { name: clip.name, error }),
    }
}

/// Puts the converted file in place of the clip, when its presentation is the active one.
/// `true` once the result has been used or dropped.
fn apply(app: &mut SlideApp, clip: &Clip, path: &std::path::Path) -> bool {
    let Some(doc_index) = app.session.documents().iter().position(|d| d.uid == clip.doc) else { return true };
    if app.session.active_index() != Some(doc_index) {
        return false;
    }
    let Some(d) = app.session.active() else { return false };
    let Some(slide) = d.doc.slides.iter().position(|s| s.id == clip.slide) else { return true };
    if crate::media::clip_on(&d.doc, slide, clip.shape).map(|c| c.media) != Some(clip.media) {
        // Deleted, or given other media meanwhile: the conversion is no longer wanted.
        return true;
    }
    let name = deckcraft_engine::tinta_convert::webm_name(&clip.name);
    let params = json!({"id": clip.shape, "slide": slide, "name": name, "path": path.to_string_lossy()});
    match app.run("media.replace", params) {
        Ok(_) => app.set_status(tr_name(DONE, &clip.name)),
        Err(error) => app.tinta_convert.dialog = Some(Dialog::Failed { name: clip.name.clone(), error }),
    }
    true
}

/// Advances the conversion and draws its windows (once per frame).
pub fn show(app: &mut SlideApp, ctx: &egui::Context) {
    let lang = crate::i18n::current().code();
    poll(app);
    progress_window(app, ctx, lang);
    // Never on top of another question (the first-start error-report one, a dialog, feedback):
    // the offer waits until that one is answered.
    if another_modal(app) {
        return;
    }
    if app.tinta_convert.dialog.is_none()
        && let Some(clip) = app.tinta_convert.queue.pop_front()
    {
        if app.tinta_convert.running.is_none() && !app.session.prefs.tinta_convert.ask {
            start(app, clip);
        } else if app.tinta_convert.running.is_none() {
            app.tinta_convert.always = false;
            app.tinta_convert.dialog = Some(Dialog::Offer(clip));
        } else {
            app.tinta_convert.queue.push_front(clip);
        }
    }
    if let Some(dialog) = app.tinta_convert.dialog.clone() {
        let modal = egui::Modal::new(egui::Id::unique("tinta.convert")).show(ctx, |ui| {
            ui.set_width(460.0);
            dialog_contents(app, ui, &dialog, lang)
        });
        if modal.inner.is_none() && modal.should_close() {
            app.tinta_convert.dialog = None;
        } else if let Some(action) = modal.inner {
            act(app, dialog, action);
        }
    }
}

/// Whether another modal question is showing.
fn another_modal(app: &SlideApp) -> bool {
    let first_start = crate::tinta_reports_ui::available(app) && app.session.prefs.tinta_reports.consent.is_none();
    first_start || app.dialog.is_some() || app.tinta_feedback.is_open()
}

fn poll(app: &mut SlideApp) {
    let Some(mut r) = app.tinta_convert.running.take() else { return };
    if r.done.is_none() {
        match r.job.outcome() {
            None => {
                app.tinta_convert.running = Some(r);
                return;
            }
            Some(Outcome::Done(path)) => r.done = Some(path),
            Some(Outcome::Cancelled) => {
                app.set_status(tr(crate::i18n::current().code(), CANCELLED));
                return;
            }
            Some(Outcome::Failed(error)) => {
                app.tinta_convert.dialog = Some(Dialog::Failed { name: r.clip.name, error });
                return;
            }
        }
    }
    if let Some(path) = r.done.clone()
        && !apply(app, &r.clip, &path)
    {
        // Its presentation isn't the active one: keep the file until it is.
        app.tinta_convert.running = Some(r);
    }
}

fn progress_window(app: &mut SlideApp, ctx: &egui::Context, lang: &str) {
    let Some(r) = &app.tinta_convert.running else { return };
    if r.done.is_some() {
        return;
    }
    ctx.request_repaint_after(std::time::Duration::from_millis(200));
    let p = r.job.progress();
    let title = tr(lang, CONVERTING).replace("{name}", &r.clip.name);
    let mut cancel = false;
    egui::Window::new(tr(lang, TITLE_PROGRESS))
        .id(egui::Id::unique("tinta.convert.progress"))
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-16.0, -40.0))
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.set_width(320.0);
            ui.label(title);
            ui.add(egui::ProgressBar::new(p).desired_width(320.0).show_percentage());
            ui.add_space(4.0);
            ui.allocate_ui_with_layout(egui::vec2(320.0, 24.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                cancel = ui.button(tr(lang, CANCEL)).clicked();
            });
        });
    if cancel {
        r.job.cancel();
    }
}

/// What a dialog's button asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Convert,
    OpenPlayer,
    Close,
    Choose,
}

fn dialog_contents(app: &mut SlideApp, ui: &mut Ui, dialog: &Dialog, lang: &str) -> Option<Action> {
    let mut action = None;
    match dialog {
        Dialog::Offer(clip) => {
            ui.label(RichText::new(tr(lang, OFFER_TITLE)).strong().size(17.0));
            ui.add_space(8.0);
            ui.label(tr(lang, OFFER_BODY).replace("{name}", &clip.name).replace("{codec}", &clip.codec));
            ui.add_space(8.0);
            ui.checkbox(&mut app.tinta_convert.always, tr(lang, ALWAYS));
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button(RichText::new(tr(lang, CONVERT)).strong()).clicked() {
                    action = Some(Action::Convert);
                }
                if ui.button(tr(lang, OPEN_PLAYER)).clicked() {
                    action = Some(Action::OpenPlayer);
                }
                if ui.button(tr(lang, NOT_NOW)).clicked() {
                    action = Some(Action::Close);
                }
            });
        }
        Dialog::Missing { error, .. } => {
            ui.label(RichText::new(tr(lang, MISSING_TITLE)).strong().size(17.0));
            ui.add_space(8.0);
            ui.label(tr(lang, MISSING_BODY));
            ui.add_space(6.0);
            if std::env::var_os("FLATPAK_ID").is_some() {
                ui.label(tr(lang, FLATPAK));
            } else {
                let (how, commands) = install_hint();
                ui.label(tr(lang, how));
                for c in commands {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(*c).monospace());
                        if ui.small_button(tr(lang, COPY)).clicked() {
                            ui.ctx().copy_text((*c).to_string());
                        }
                    });
                }
            }
            if let Some(e) = error {
                ui.add_space(6.0);
                ui.label(RichText::new(tr(lang, DETAILS).replace("{error}", e)).small().weak());
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button(RichText::new(tr(lang, TRY_AGAIN)).strong()).clicked() {
                    action = Some(Action::Convert);
                }
                if app.services.pick_open.is_some() && ui.button(tr(lang, CHOOSE)).clicked() {
                    action = Some(Action::Choose);
                }
                if ui.button(tr(lang, OPEN_PLAYER)).clicked() {
                    action = Some(Action::OpenPlayer);
                }
                if ui.button(tr(lang, CLOSE)).clicked() {
                    action = Some(Action::Close);
                }
            });
        }
        Dialog::Failed { name, error } => {
            ui.label(RichText::new(tr(lang, FAILED_TITLE)).strong().size(17.0));
            ui.add_space(8.0);
            ui.label(name.as_str());
            ui.label(RichText::new(tr(lang, DETAILS).replace("{error}", error)).small().weak());
            ui.add_space(10.0);
            if ui.button(tr(lang, CLOSE)).clicked() {
                action = Some(Action::Close);
            }
        }
    }
    action
}

/// The install instructions for this system: the sentence and the commands.
fn install_hint() -> (&'static str, &'static [&'static str]) {
    if cfg!(windows) {
        (HOW_WINDOWS, &["winget install Gyan.FFmpeg"])
    } else if cfg!(target_os = "macos") {
        (HOW_MAC, &["brew install ffmpeg"])
    } else {
        (HOW_LINUX, &["sudo apt install ffmpeg", "sudo dnf install ffmpeg"])
    }
}

fn act(app: &mut SlideApp, dialog: Dialog, action: Action) {
    app.tinta_convert.dialog = None;
    let clip = match dialog {
        Dialog::Offer(clip) | Dialog::Missing { clip, .. } => Some(clip),
        Dialog::Failed { .. } => None,
    };
    match (action, clip) {
        (Action::Convert, Some(clip)) => {
            if app.tinta_convert.always {
                let _ = app.run("media.convertOptions", json!({"ask": false}));
                app.tinta_convert.always = false;
            }
            // A new attempt looks for FFmpeg again (it may have been installed meanwhile).
            app.tinta_convert.found = None;
            start(app, clip);
        }
        (Action::OpenPlayer, Some(clip)) => {
            let slide = app.session.active().and_then(|d| d.doc.slides.iter().position(|s| s.id == clip.slide));
            app.open_media_externally(clip.shape, slide);
        }
        (Action::Choose, Some(clip)) => choose_ffmpeg(app, clip),
        _ => {}
    }
}

/// "Choose FFmpeg…": the picked program becomes the setting once it proves to be an FFmpeg that
/// writes WebM; then the clip is converted with it.
fn choose_ffmpeg(app: &mut SlideApp, clip: Clip) {
    let Some(pick) = app.services.pick_open.as_mut() else { return };
    let Some(path) = pick("ffmpeg") else {
        app.tinta_convert.dialog = Some(Dialog::Missing { clip, error: None });
        return;
    };
    match transcode::check(std::path::Path::new(&path)) {
        Ok(f) => {
            app.session.prefs.tinta_convert.ffmpeg_path = path.clone();
            app.tinta_convert.found = Some((path, f));
            start(app, clip);
        }
        Err(e) => app.tinta_convert.dialog = Some(Dialog::Missing { clip, error: Some(e.to_string()) }),
    }
}

fn tr_name(text: &'static str, name: &str) -> String {
    tr(crate::i18n::current().code(), text).replace("{name}", name)
}

pub(crate) const LINK: &str = "Convert to WebM";
const OFFER_TITLE: &str = "Convert this clip to WebM?";
const OFFER_BODY: &str = "Tinta Presenta can't play {name} ({codec}). FFmpeg on this computer can convert it to WebM (VP9 video, Opus sound), which plays here and in web browsers. The converted clip replaces the original in this presentation; Undo brings the original back.";
const ALWAYS: &str = "Always convert without asking";
const CONVERT: &str = "Convert";
const OPEN_PLAYER: &str = "Open in the system player";
const NOT_NOW: &str = "Not now";
const TITLE_PROGRESS: &str = "Converting to WebM";
const CONVERTING: &str = "Converting {name}…";
const CANCEL: &str = "Cancel";
const DONE: &str = "{name} was converted to WebM.";
const CANCELLED: &str = "Conversion cancelled.";
const QUEUED: &str = "{name} will be converted after the current clip.";
const FAILED_TITLE: &str = "The clip couldn't be converted";
const CLOSE: &str = "Close";
const MISSING_TITLE: &str = "FFmpeg is needed to convert clips";
const MISSING_BODY: &str = "Converting uses FFmpeg, a free program that Tinta Office doesn't include. Install it, then press Try again.";
const HOW_WINDOWS: &str = "To install it, run this in PowerShell or Command Prompt:";
const HOW_MAC: &str = "To install it with Homebrew, run this in Terminal:";
const HOW_LINUX: &str = "To install it, run one of these in a terminal (Debian and Ubuntu, Fedora):";
const FLATPAK: &str = "This Flatpak version can't run the FFmpeg installed on your system. To convert clips, install the .deb or .rpm package of Tinta Presenta, or open the clip in the system player.";
const COPY: &str = "Copy";
const CHOOSE: &str = "Choose FFmpeg…";
const TRY_AGAIN: &str = "Try again";
const DETAILS: &str = "Details: {error}";

const STRINGS: [&str; 25] = [
    LINK,
    OFFER_TITLE,
    OFFER_BODY,
    ALWAYS,
    CONVERT,
    OPEN_PLAYER,
    NOT_NOW,
    TITLE_PROGRESS,
    CONVERTING,
    CANCEL,
    DONE,
    CANCELLED,
    QUEUED,
    FAILED_TITLE,
    CLOSE,
    MISSING_TITLE,
    MISSING_BODY,
    HOW_WINDOWS,
    HOW_MAC,
    HOW_LINUX,
    FLATPAK,
    COPY,
    CHOOSE,
    TRY_AGAIN,
    DETAILS,
];

/// `text` (one of the constants above) in the language `lang` (`es`, `ca-valencia`, `pt_BR`…),
/// English when that language has no translation.
pub fn tr(lang: &str, text: &'static str) -> &'static str {
    let lang = lang.split(['-', '_']).next().unwrap_or(lang).to_ascii_lowercase();
    let Some(i) = STRINGS.iter().position(|s| *s == text) else { return text };
    TRANSLATIONS.iter().find(|(code, _)| *code == lang).and_then(|(_, t)| t.get(i).copied()).filter(|t| !t.is_empty()).unwrap_or(text)
}

include!("tinta_convert_i18n.rs");

#[cfg(test)]
#[path = "tinta_convert_tests.rs"]
mod tests;
