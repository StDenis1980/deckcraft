//! One background check at start-up (and on demand from Help) for a newer release, and the modal
//! prompt that offers to update and restart. The flow itself is `craft_update::updater::Updater`.

use craft_update::updater::{Action, Status, Updater};

use crate::SlideApp;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The app the update check works on: the release, or a rebranded build pointed at its own
/// repository (`crates/brand`), so it never offers or installs the release's downloads in place of
/// its own.
pub fn update_app() -> &'static craft_update::App {
    use deckcraft_engine::brand;
    static APP: std::sync::OnceLock<craft_update::App> = std::sync::OnceLock::new();
    APP.get_or_init(|| {
        let release = craft_update::suite::DECKCRAFT;
        if !brand::REBRANDED {
            return release;
        }
        // A rebranded build without a GitHub repository has no releases to check: the empty repo
        // makes every check fail, so nothing is ever downloaded from the release's.
        craft_update::App { name: brand::NAME, repo: brand::github_repo().unwrap_or(""), former_ids: &[], former_names: &[], ..release }
    })
}

pub(crate) struct Runtime {
    updater: Updater,
    /// The desktop host turns the check at start on; tests and the web build leave it off.
    enabled: bool,
    check_requested: bool,
}

impl Default for Runtime {
    fn default() -> Self {
        Self { updater: Updater::new(update_app(), VERSION), enabled: false, check_requested: false }
    }
}

pub(crate) fn enable(app: &mut SlideApp) {
    app.updates_rt.enabled = true;
}

/// Help > Check for Updates…: runs a check and reports the outcome either way.
pub(crate) fn request_check(app: &mut SlideApp) {
    if !matches!(app.updates_rt.updater.status(), Status::Checking) {
        app.updates_rt.check_requested = true;
        app.notify("Checking for updates…");
    }
}

pub(crate) fn tick(app: &mut SlideApp, ctx: &egui::Context) {
    let rt = &mut app.updates_rt;
    if std::mem::take(&mut rt.check_requested) {
        rt.updater.check_now(ctx);
    } else if rt.enabled {
        rt.updater.check_at_start(ctx);
    }
    rt.updater.tick(ctx);
    let outcome = match rt.updater.status() {
        Status::UpToDate => Some(format!("{} {VERSION} is up to date.", deckcraft_engine::brand::NAME)),
        Status::Failed(e) => Some(format!("Could not check for updates: {e}")),
        Status::Idle | Status::Checking => None,
    };
    if let Some(message) = outcome {
        rt.updater.dismiss_status();
        app.notify(message);
    }
}

pub(crate) fn show(app: &mut SlideApp, ctx: &egui::Context) {
    // The restart helper is scheduled before the app closes, so no update starts over unsaved work.
    let clean = !app.session.documents().iter().any(|d| d.is_dirty());
    match app.updates_rt.updater.show_if(ctx, clean) {
        Action::OpenUrl(url) => ctx.open_url(egui::OpenUrl::new_tab(url)),
        // The app's close path asks about unsaved work; the restart helper relaunches once the process exits.
        Action::Close => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
        Action::None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_offered_release_opens_the_prompt() {
        let mut app = SlideApp::new(deckcraft_engine::Session::new(), crate::Services::default());
        let ctx = egui::Context::default();
        assert!(!app.updates_rt.updater.prompt_open());
        app.updates_rt.updater.offer(craft_update::Release { tag: "v99.0.0".into(), ..Default::default() });
        ctx.run_ui(egui::RawInput::default(), |ui| show(&mut app, ui.ctx())).textures_delta.clear();
        assert!(app.updates_rt.updater.prompt_open());
    }
}
