use super::{Rect, Sense, SlideApp, Target, Ui, Xf, json, pos2, screen_rect, vec2};

/// Play/pause on an animated GIF under the pointer or selected on its own.
pub(super) fn gif_buttons(app: &mut SlideApp, ui: &mut Ui, xf: Xf, gifs: &[crate::media::Gif]) {
    let Some(d) = app.session.active() else { return };
    if d.selection.text.is_some() || app.session.tool.dragging() {
        return;
    }
    let only = if d.selection.shapes.len() == 1 { d.selection.shapes.first().copied() } else { None };
    let pointer = ui.input(|i| i.pointer.hover_pos());
    let mut toggle = None;
    for g in gifs {
        let r = screen_rect(xf, &g.rect);
        if only != Some(g.id) && !pointer.is_some_and(|p| r.contains(p)) {
            continue;
        }
        let b = crate::media::gif_button_rect(r);
        // Sensing drags too keeps the press from reaching the canvas (selecting or moving the GIF).
        let resp = ui.interact(b, ui.scope_id().with(("gif_button", g.id.0)), Sense::click_and_drag());
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, if g.paused { "Play animated GIF" } else { "Pause animated GIF" }));
        crate::media::paint_gif_button(ui.painter(), b, g.paused, resp.hovered());
        if resp.clicked() {
            toggle = Some(g.id);
        }
    }
    if let Some(id) = toggle {
        let _ = app.run("media.gifPlay", json!({"id": id}));
    }
}

/// Video playing in the editor: its current frame over the poster.
pub(super) fn media_frames(app: &SlideApp, painter: &egui::Painter, xf: Xf) {
    let Some(d) = app.session.active() else { return };
    if d.selection.target != Target::Slides || !app.media.any_active() {
        return;
    }
    for (id, x, clip) in crate::media::slide_media(&d.doc, d.selection.slide) {
        if clip.video {
            crate::media::paint_frame(painter, &app.media, id, screen_rect(xf, &x), clip.crop);
        }
    }
}

/// The play/pause and timeline bar under a selected audio or video (PowerPoint shows it on
/// selection or hover).
pub(super) fn media_bar(app: &mut SlideApp, ui: &mut Ui, xf: Xf) {
    let Some(d) = app.session.active() else { return };
    let uid = d.uid;
    if d.selection.target != Target::Slides || d.selection.text.is_some() || app.session.tool.dragging() {
        return;
    }
    let pointer = ui.input(|i| i.pointer.hover_pos());
    let media = crate::media::slide_media(&d.doc, d.selection.slide);
    let pick = media.iter().find(|(id, _, _)| d.selection.shapes.len() == 1 && d.selection.shapes[0] == *id).or_else(|| {
        media.iter().find(|(id, x, _)| {
            let r = screen_rect(xf, x);
            app.media.status(*id).is_some() || pointer.is_some_and(|p| r.expand2(vec2(0.0, 40.0)).contains(p))
        })
    });
    let Some((id, x, clip)) = pick.cloned() else { return };
    let doc = d.doc.clone();
    let r = screen_rect(xf, &x);
    let w = r.width().max(230.0);
    let bar = Rect::from_min_size(pos2(r.center().x - w / 2.0, r.max.y + 6.0), vec2(w, 26.0));
    let status = app.media.status(id);
    let video_error = app.media.video_error(id).map(str::to_string);
    // Tinta Presenta: a clip that doesn't play (no picture, or no sound) can be converted to WebM.
    let unplayable = video_error.is_some() || status.as_ref().is_some_and(|s| s.error.is_some());
    if unplayable && crate::tinta_convert::available(app) && convert_link(ui, bar) {
        crate::tinta_convert::convert_now(app, id);
    }
    match crate::media::control_bar(ui, bar, status.as_ref(), &clip, video_error.as_deref()) {
        Some(crate::media::BarAction::Toggle) => app.media.toggle(uid, &doc, id, &clip, crate::media::Owner::Editor),
        Some(crate::media::BarAction::Seek(t)) => app.media.seek(uid, &doc, id, &clip, t, crate::media::Owner::Editor),
        Some(crate::media::BarAction::Nudge(dt)) => {
            let pos = status.map(|s| s.position).unwrap_or(0.0);
            app.media.seek(uid, &doc, id, &clip, (pos + dt).max(0.0), crate::media::Owner::Editor);
        }
        Some(crate::media::BarAction::OpenExternal) => {
            let _ = app.run("media.openExternal", json!({"id": id}));
        }
        None => {}
    }
}

/// Tinta Presenta: the "Convert to WebM" link under the media bar's own note. `true` when clicked.
fn convert_link(ui: &mut Ui, bar: Rect) -> bool {
    let text = crate::tinta_convert::tr(crate::i18n::current().code(), crate::tinta_convert::LINK);
    let link =
        egui::Label::new(egui::RichText::new(text).size(11.0).color(egui::Color32::from_rgb(0x8A, 0xB8, 0xFF)).underline()).sense(Sense::click());
    let r = Rect::from_min_size(pos2(bar.min.x, bar.max.y + 21.0), vec2(bar.width().max(260.0), 16.0));
    let resp = ui.put(r, link);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Link, true, crate::tinta_convert::LINK));
    resp.clicked()
}
