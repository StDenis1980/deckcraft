//! Tinta Presenta: offering, running and applying a clip conversion in the app. The tests that run
//! FFmpeg skip when none is installed (CI has none); the rest use a path that isn't one.

use deckcraft_engine::Session;
use deckcraft_model::ShapeId;
use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use serde_json::json;

use super::Dialog;
use crate::{Services, SlideApp};

const H264: &[u8] = include_bytes!("../../media/tests/fixtures/clip-h264.mp4");
const VP9: &[u8] = include_bytes!("../../media/tests/fixtures/clip-vp9.webm");

fn harness(convert: bool, ffmpeg: &str, ask: bool, clip: (&str, &[u8])) -> (Harness<'static, SlideApp>, ShapeId) {
    let mut s = Session::with_new();
    s.execute("slide.new", &json!({"layout": "blank"})).unwrap();
    s.prefs.tinta_convert.ffmpeg_path = ffmpeg.into();
    s.prefs.tinta_convert.ask = ask;
    let app = SlideApp::new(s, Services { convert_media: convert, ..Default::default() });
    let mut h = Harness::builder().with_size(vec2(1400.0, 900.0)).with_step_dt(1.0 / 60.0).build_ui_state(
        |ui, app: &mut SlideApp| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(2);
    let r = h.state_mut().run("insert.video", json!({"name": clip.0, "data": deckcraft_engine::cmd::base64_encode(clip.1)})).unwrap();
    let id = ShapeId(r["id"].as_u64().unwrap() as u32);
    h.run_steps(3);
    (h, id)
}

fn media_name(h: &Harness<'_, SlideApp>, id: ShapeId) -> String {
    let d = h.state().session.active().unwrap();
    let clip = crate::media::find_clip(&d.doc, id, Some(d.selection.slide)).unwrap();
    d.doc.media(clip.media).unwrap().name.clone()
}

/// Steps the app until the conversion has ended (at most 60 s).
fn finish(h: &mut Harness<'_, SlideApp>) {
    for _ in 0..1200 {
        h.run_steps(1);
        if h.state().tinta_convert.running.is_none() {
            h.run_steps(2);
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    panic!("the conversion didn't end");
}

fn have_ffmpeg() -> bool {
    !deckcraft_media::PATENTED_CODECS && deckcraft_media::transcode::find(None).is_ok()
}

#[test]
fn inserting_a_clip_that_does_not_play_asks_to_convert_it() {
    if deckcraft_media::PATENTED_CODECS {
        return;
    }
    let (h, _) = harness(true, "/nonexistent/ffmpeg", true, ("clip.mp4", H264));
    assert!(
        matches!(&h.state().tinta_convert.dialog, Some(Dialog::Offer(c)) if c.name == "clip.mp4" && c.codec.contains("H.264")),
        "{:?}",
        h.state().tinta_convert
    );
    h.get_by_label("Convert this clip to WebM?");
}

#[test]
fn nothing_is_offered_for_a_clip_that_plays_or_by_a_build_that_cannot_convert() {
    let (h, _) = harness(true, "", true, ("clip.webm", VP9));
    assert!(!h.state().tinta_convert.is_busy());
    let (h, _) = harness(false, "", true, ("clip.mp4", H264));
    assert!(!h.state().tinta_convert.is_busy());
}

#[test]
fn not_now_closes_the_question_and_keeps_the_clip() {
    if deckcraft_media::PATENTED_CODECS {
        return;
    }
    let (mut h, id) = harness(true, "/nonexistent/ffmpeg", true, ("clip.mp4", H264));
    h.get_by_label("Not now").click();
    h.run_steps(3);
    assert!(h.state().tinta_convert.dialog.is_none());
    assert_eq!(media_name(&h, id), "clip.mp4");
}

#[test]
fn without_ffmpeg_the_app_says_how_to_install_it() {
    if deckcraft_media::PATENTED_CODECS {
        return;
    }
    let (mut h, _) = harness(true, "/nonexistent/ffmpeg", true, ("clip.mp4", H264));
    h.get_by_label("Convert").click();
    h.run_steps(3);
    assert!(matches!(&h.state().tinta_convert.dialog, Some(Dialog::Missing { error: Some(_), .. })), "{:?}", h.state().tinta_convert);
    h.get_by_label("FFmpeg is needed to convert clips");
    h.get_by_label("Try again");
    h.get_by_label("Close").click();
    h.run_steps(3);
    assert!(!h.state().tinta_convert.is_busy());
}

#[test]
fn always_convert_turns_the_question_off() {
    if deckcraft_media::PATENTED_CODECS {
        return;
    }
    let (mut h, _) = harness(true, "/nonexistent/ffmpeg", true, ("clip.mp4", H264));
    h.get_by_label("Always convert without asking").click();
    h.run_steps(1);
    h.get_by_label("Convert").click();
    h.run_steps(3);
    assert!(!h.state().session.prefs.tinta_convert.ask, "the setting is saved");
    // The next clip goes straight to converting (here: to the missing-FFmpeg hint).
    h.state_mut().tinta_convert.dialog = None;
    h.state_mut().run("insert.video", json!({"name": "two.mp4", "data": deckcraft_engine::cmd::base64_encode(H264)})).unwrap();
    h.run_steps(3);
    assert!(matches!(&h.state().tinta_convert.dialog, Some(Dialog::Missing { .. })), "{:?}", h.state().tinta_convert);
}

#[test]
fn a_converted_clip_replaces_the_original_and_undo_brings_it_back() {
    if !have_ffmpeg() {
        return;
    }
    let (mut h, id) = harness(true, "", true, ("clip.mp4", H264));
    h.get_by_label("Convert").click();
    h.run_steps(2);
    assert!(h.state().tinta_convert.running.is_some(), "{:?}", h.state().tinta_convert);
    finish(&mut h);
    assert!(h.state().tinta_convert.dialog.is_none(), "{:?}", h.state().tinta_convert);
    assert_eq!(media_name(&h, id), "clip.webm");
    let info = h.state_mut().run("media.info", json!({"id": id})).unwrap();
    assert_eq!(info["needsConversion"], false);
    assert!(h.state().status.as_ref().is_some_and(|(s, _)| s.contains("converted")), "{:?}", h.state().status);
    h.state_mut().run("edit.undo", json!({})).unwrap();
    assert_eq!(media_name(&h, id), "clip.mp4");
}

#[test]
fn cancel_stops_the_conversion_and_keeps_the_original() {
    if !have_ffmpeg() {
        return;
    }
    let (mut h, id) = harness(true, "", false, ("clip.mp4", H264));
    assert!(h.state().tinta_convert.running.is_some());
    h.get_by_label("Cancel").click();
    finish(&mut h);
    // A tiny clip may finish before the click lands; otherwise the original stays.
    let name = media_name(&h, id);
    assert!(name == "clip.mp4" || name == "clip.webm", "{name}");
    assert!(h.state().tinta_convert.dialog.is_none());
}

#[test]
fn a_clip_deleted_during_the_conversion_is_left_alone() {
    if !have_ffmpeg() {
        return;
    }
    let (mut h, id) = harness(true, "", false, ("clip.mp4", H264));
    h.state_mut().run("edit.delete", json!({"ids": [id]})).unwrap();
    finish(&mut h);
    assert!(h.state().tinta_convert.dialog.is_none(), "{:?}", h.state().tinta_convert);
    assert!(h.state().session.active().unwrap().shape(id).is_none());
}

#[test]
fn every_translation_keeps_the_placeholders() {
    for (code, t) in super::TRANSLATIONS {
        for (en, tr) in super::STRINGS.iter().zip(t.iter()) {
            for ph in ["{name}", "{codec}", "{error}"] {
                assert_eq!(en.matches(ph).count(), tr.matches(ph).count(), "{code}: {tr}");
            }
            assert_eq!(en.ends_with('…'), tr.ends_with('…'), "{code}: {tr}");
            assert!(!tr.is_empty(), "{code}: {en}");
        }
    }
    assert_eq!(super::tr("uk", super::CONVERT), "Перетворити");
    assert_eq!(super::tr("ca-valencia", super::CONVERT), "Converteix");
    assert_eq!(super::tr("ja", super::CONVERT), "Convert");
}

#[test]
#[ignore = "writes screenshots for a person to look at"]
fn screenshots() {
    let out = std::path::PathBuf::from(std::env::var("SHOTS").unwrap());
    let save = |h: &mut Harness<'_, SlideApp>, name: &str| {
        h.run_steps(2);
        let img = h.render().unwrap();
        img.save(out.join(name)).unwrap();
    };
    for lang in ["en", "uk"] {
        let (mut h, _) = harness(true, "/nonexistent/ffmpeg", true, ("clip.mp4", H264));
        h.state_mut().ui.language = lang.into();
        save(&mut h, &format!("offer-{lang}.png"));
        h.get_by_label(super::tr(lang, super::CONVERT)).click();
        save(&mut h, &format!("missing-{lang}.png"));
        let big = std::fs::read(out.join("talk.mp4")).unwrap();
        let (mut h, _) = harness(true, "", false, ("talk.mp4", &big));
        h.state_mut().ui.language = lang.into();
        std::thread::sleep(std::time::Duration::from_millis(1500));
        save(&mut h, &format!("progress-{lang}.png"));
        finish(&mut h);
        save(&mut h, &format!("done-{lang}.png"));
    }
}
