//! Tinta Presenta: converting audio and video this build can't play (H.264, HEVC, AAC, WMV…) to
//! WebM (VP9 and Opus) with the FFmpeg the person installed (`deckcraft_media::transcode`).
//!
//! - `media.replace` swaps a clip's media for other bytes (one undo step), keeping its size,
//!   trimming, fades, volume and playback options. The UI calls it with the converted file.
//! - `media.convert` converts and replaces in one go, waiting for FFmpeg: the command-line, MCP
//!   and test path. The app converts in the background instead, with a progress bar and Cancel.
//! - `media.convertOptions` reads or sets whether the app asks before converting, and reports the
//!   FFmpeg it would use.
//! - Inserting a clip that doesn't play leaves [`OFFER`] for the UI, which offers to convert it.
//!
//! Which program runs is never a command parameter: an agent can't make the app start another
//! program. The FFmpeg path comes from the person's settings ([`ConvertPrefs::ffmpeg_path`], set in
//! the app's dialog), the `DECKCRAFT_FFMPEG` environment variable, or the search in
//! `deckcraft_media::transcode::find`.

use std::sync::Arc;

use deckcraft_model::{MediaClip, ShapeId, ShapeKind};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::cmd::{CommandSpec, NO_ID_SPACE, bad, base64_decode, bool_param, cmd, has_doc, id_param, opt_usize_param, str_param};
use crate::{EngineError, Result, Session, UiRequest};

/// The [`UiRequest::Media`] action asking the UI to offer converting a clip.
pub const OFFER: &str = "convertOffer";

/// The saved choices.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ConvertPrefs {
    /// The FFmpeg program the person chose; empty to search for one.
    pub ffmpeg_path: String,
    /// Ask before converting an inserted clip (otherwise convert straight away).
    pub ask: bool,
}

impl Default for ConvertPrefs {
    fn default() -> Self {
        ConvertPrefs { ffmpeg_path: String::new(), ask: true }
    }
}

/// The longest path kept.
const MAX_PATH: usize = 4096;

/// A damaged saved value reads as the defaults; an overlong path is dropped.
pub fn lenient<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<ConvertPrefs, D::Error> {
    let value = Value::deserialize(d)?;
    let mut p: ConvertPrefs = serde_json::from_value(value).unwrap_or_default();
    if p.ffmpeg_path.len() > MAX_PATH || p.ffmpeg_path.contains('\0') {
        p.ffmpeg_path.clear();
    }
    Ok(p)
}

/// The FFmpeg path to use: the settings, else `DECKCRAFT_FFMPEG`, else none (search).
pub fn configured_ffmpeg(prefs: &ConvertPrefs) -> Option<std::path::PathBuf> {
    if !prefs.ffmpeg_path.trim().is_empty() {
        return Some(prefs.ffmpeg_path.trim().into());
    }
    std::env::var_os("DECKCRAFT_FFMPEG").filter(|v| !v.is_empty()).map(Into::into)
}

/// Whether a clip (a video when `video`) needs converting before it plays: something in it can't
/// be decoded, or it couldn't be read at all.
pub fn needs_conversion(info: Option<&deckcraft_media::MediaInfo>, video: bool) -> bool {
    match info {
        None => true,
        Some(i) if video => !i.video.as_ref().is_some_and(|v| v.decodable) || i.audio.as_ref().is_some_and(|a| !a.decodable),
        Some(i) => !i.audio.as_ref().is_some_and(|a| a.decodable),
    }
}

/// The name of a converted clip: `talk.mp4` → `talk.webm`.
pub fn webm_name(name: &str) -> String {
    let stem = match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() && ext.len() <= 5 && !ext.contains(['/', '\\']) => stem,
        _ => name,
    };
    let stem = if stem.trim().is_empty() { "media" } else { stem };
    format!("{stem}.webm")
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "media.replace",
            "Replace Media",
            [],
            None,
            "{id?, slide?: index (default: the current slide), name, data?: base64 | path?} → {id, name, contentType, durationMs, width, height}: the clip plays the new media (which must play in this build), keeping its options",
            has_doc,
            replace
        ),
        cmd!(
            "media.convert",
            "Convert to WebM",
            [],
            None,
            "{id?, slide?: index} → {id, name, contentType, durationMs, width, height, ffmpeg}: converts a clip this build can't play to WebM (VP9/Opus) with the FFmpeg installed on this computer, waiting for it",
            has_doc,
            convert
        ),
        cmd!(
            query "media.convertOptions",
            "Video Conversion Options",
            [],
            None,
            "{ask?: bool (ask before converting an inserted clip)} → {ask, ffmpegPath, ffmpeg: {path, version} | {error}}",
            crate::cmd::always,
            options
        ),
    ]
}

/// The media shape `p` names (`id`, else the only selected or only media shape) on slide `slide`
/// (default: the current one), with its clip.
fn find_clip(s: &Session, p: &Value, cmd: &str) -> Result<(Option<usize>, ShapeId, MediaClip)> {
    let slide = opt_usize_param(p, "slide", cmd)?;
    let Some(index) = slide else {
        let (id, clip) = crate::cmd::media::target(s, p, cmd)?;
        return Ok((None, id, clip));
    };
    let st = s.doc()?;
    let sl = st.doc.slides.get(index).ok_or_else(|| bad(cmd, format!("no slide {index}")))?;
    let id = id_param(p, "id", cmd)?.ok_or_else(|| bad(cmd, "`slide` needs `id`"))?;
    match deckcraft_model::find_shape(&sl.shapes, id).map(|sh| &sh.kind) {
        Some(ShapeKind::Media(m)) => Ok((Some(index), id, m.clone())),
        Some(_) => Err(bad(cmd, "not a media shape")),
        None => Err(bad(cmd, "no such shape")),
    }
}

fn input_bytes(p: &Value, cmd: &str) -> Result<Vec<u8>> {
    if let Some(d) = str_param(p, "data") {
        return base64_decode(d).ok_or_else(|| bad(cmd, "invalid base64 `data`"));
    }
    if let Some(path) = str_param(p, "path") {
        return crate::io::read_file(path).map_err(EngineError::Other);
    }
    Err(bad(cmd, "missing `path` or `data`"))
}

fn replace(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "media.replace";
    let name = str_param(p, "name").filter(|n| !n.trim().is_empty()).unwrap_or("media.webm").chars().take(255).collect::<String>();
    let bytes = input_bytes(p, cmd)?;
    replace_with(s, p, cmd, name, bytes)
}

/// Points the clip `p` names at `bytes`, which must play: a converted clip that doesn't is never
/// put in place of the original.
fn replace_with(s: &mut Session, p: &Value, cmd: &str, name: String, bytes: Vec<u8>) -> Result<Value> {
    let (slide, id, clip) = find_clip(s, p, cmd)?;
    let bytes = Arc::new(bytes);
    let info = deckcraft_media::probe(&bytes).map_err(|e| bad(cmd, format!("{name}: {e}")))?;
    if needs_conversion(Some(&info), clip.video) {
        return Err(bad(cmd, format!("{name} doesn't play in this build either")));
    }
    let content_type = match (info.container, info.video.is_some()) {
        ("webm", true) => "video/webm",
        ("webm", false) => "audio/webm",
        (_, true) => "video/mp4",
        (_, false) => "audio/mp4",
    };
    let poster_png = if clip.video && clip.poster.is_none() { crate::cmd::media::poster_png(&bytes, 0) } else { None };
    let (w, h) = info.video.as_ref().map(|v| (v.width, v.height)).unwrap_or((clip.width, clip.height));
    let duration_ms = u32::try_from(info.duration_ms).unwrap_or(u32::MAX);
    let data = Arc::try_unwrap(bytes).unwrap_or_else(|a| (*a).clone());
    s.edit(|doc, sel| {
        let media = doc.add_media(&name, content_type, data).ok_or_else(|| bad(cmd, NO_ID_SPACE))?;
        let poster = match poster_png {
            Some(png) => Some(doc.add_media(&format!("{name}-poster.png"), "image/png", png).ok_or_else(|| bad(cmd, NO_ID_SPACE))?),
            None => clip.poster,
        };
        let shapes = match slide {
            Some(i) => doc.slides.get_mut(i).map(|sl| &mut Arc::make_mut(sl).shapes),
            None => crate::shapes_mut(doc, sel),
        }
        .ok_or_else(|| bad(cmd, "no slide"))?;
        let Some(ShapeKind::Media(m)) = deckcraft_model::find_shape_mut(shapes, id).map(|sh| &mut sh.kind) else {
            return Err(bad(cmd, "no such shape"));
        };
        m.media = media;
        m.poster = poster;
        m.duration_ms = duration_ms;
        m.width = w;
        m.height = h;
        // A trim past the new end (a clip a few milliseconds shorter) would play nothing.
        if duration_ms > 0 {
            m.trim_start_ms = m.trim_start_ms.min(duration_ms);
            m.trim_end_ms = m.trim_end_ms.min(duration_ms.saturating_sub(m.trim_start_ms));
        }
        Ok(())
    })?;
    Ok(json!({"id": id, "name": name, "contentType": content_type, "durationMs": duration_ms, "width": w, "height": h}))
}

#[cfg(not(target_arch = "wasm32"))]
fn convert(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "media.convert";
    let (_, _, clip) = find_clip(s, p, cmd)?;
    let item = s.doc()?.doc.media(clip.media).cloned().ok_or_else(|| EngineError::Other("the media data is missing".into()))?;
    let info = deckcraft_media::probe(&item.data).ok();
    if !needs_conversion(info.as_ref(), clip.video) {
        return Err(bad(cmd, format!("{} already plays", item.name)));
    }
    let ffmpeg = deckcraft_media::transcode::find(configured_ffmpeg(&s.prefs.tinta_convert).as_deref()).map_err(|e| bad(cmd, e.to_string()))?;
    let duration = info.map(|i| i.duration_ms).unwrap_or(u64::from(clip.duration_ms));
    let webm = deckcraft_media::transcode::convert(&ffmpeg, &item.data, clip.video, duration).map_err(|e| bad(cmd, format!("FFmpeg: {e}")))?;
    let mut r = replace_with(s, p, cmd, webm_name(&item.name), webm)?;
    r["ffmpeg"] = json!(ffmpeg.path.to_string_lossy());
    Ok(r)
}

#[cfg(target_arch = "wasm32")]
fn convert(_: &mut Session, _: &Value) -> Result<Value> {
    Err(bad("media.convert", "converting needs the desktop app"))
}

fn options(s: &mut Session, p: &Value) -> Result<Value> {
    match p.get("ask") {
        None | Some(Value::Null) => {}
        Some(Value::Bool(b)) => s.prefs.tinta_convert.ask = *b,
        Some(_) => return Err(bad("media.convertOptions", "`ask` must be true or false")),
    }
    if bool_param(p, "ffmpegPath").is_some() || str_param(p, "ffmpegPath").is_some() {
        return Err(bad("media.convertOptions", "the FFmpeg path is set in the app, not by a command"));
    }
    let prefs = &s.prefs.tinta_convert;
    #[cfg(not(target_arch = "wasm32"))]
    let ffmpeg = match deckcraft_media::transcode::find(configured_ffmpeg(prefs).as_deref()) {
        Ok(f) => json!({"path": f.path.to_string_lossy(), "version": f.version}),
        Err(e) => json!({"error": e.to_string()}),
    };
    #[cfg(target_arch = "wasm32")]
    let ffmpeg = json!({"error": "converting needs the desktop app"});
    Ok(json!({"ask": prefs.ask, "ffmpegPath": prefs.ffmpeg_path, "ffmpeg": ffmpeg}))
}

/// After inserting a clip that doesn't play: ask the UI to offer converting it.
pub(crate) fn offer(s: &mut Session, shape: ShapeId) {
    s.ui_requests.push(UiRequest::Media { action: OFFER.into(), shape, ms: None });
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::cmd::base64_encode;

    fn session_with(name: &str, bytes: &[u8]) -> (Session, ShapeId) {
        let mut s = Session::new();
        s.execute("file.new", &json!({})).expect("new");
        let r = s.execute("insert.video", &json!({"name": name, "data": base64_encode(bytes)})).expect("insert");
        let id = ShapeId(r["id"].as_u64().and_then(|v| u32::try_from(v).ok()).expect("id"));
        (s, id)
    }

    const VP9: &[u8] = include_bytes!("../../media/tests/fixtures/clip-vp9.webm");
    const H264: &[u8] = include_bytes!("../../media/tests/fixtures/clip-h264.mp4");

    #[test]
    fn webm_names() {
        assert_eq!(webm_name("talk.mp4"), "talk.webm");
        assert_eq!(webm_name("my.talk.MOV"), "my.talk.webm");
        assert_eq!(webm_name("noext"), "noext.webm");
        assert_eq!(webm_name(".mp4"), ".mp4.webm");
        assert_eq!(webm_name(""), "media.webm");
    }

    #[test]
    fn inserting_a_clip_that_does_not_play_offers_to_convert_it() {
        if deckcraft_media::PATENTED_CODECS {
            return;
        }
        let (s, id) = session_with("clip.mp4", H264);
        assert!(s.ui_requests.contains(&UiRequest::Media { action: OFFER.into(), shape: id, ms: None }), "{:?}", s.ui_requests);
        let (s, _) = session_with("clip.webm", VP9);
        assert!(!s.ui_requests.iter().any(|r| matches!(r, UiRequest::Media { action, .. } if action == OFFER)));
    }

    #[test]
    fn replace_swaps_the_media_in_one_undo_step_and_keeps_the_options() {
        let (mut s, id) = session_with("clip.mp4", H264);
        // Options first, so the replacement must keep them.
        s.execute("media.options", &json!({"id": id, "loop": true, "volume": 0.5})).expect("options");
        let before = s.execute("media.info", &json!({"id": id})).expect("info");
        let r = s.execute("media.replace", &json!({"id": id, "name": "clip.webm", "data": base64_encode(VP9)})).expect("replace");
        assert_eq!(r["contentType"], "video/webm");
        let after = s.execute("media.info", &json!({"id": id})).expect("info");
        assert_eq!(after["name"], "clip.webm");
        assert_eq!(after["probe"]["video"]["codec"], "VP9");
        assert_eq!(after["options"], before["options"]);
        s.execute("edit.undo", &json!({})).expect("undo");
        let undone = s.execute("media.info", &json!({"id": id})).expect("info");
        assert_eq!(undone["name"], before["name"]);
        assert_eq!(undone["contentType"], before["contentType"]);
    }

    #[test]
    fn replace_refuses_media_that_does_not_play_either() {
        if deckcraft_media::PATENTED_CODECS {
            return;
        }
        let (mut s, id) = session_with("clip.webm", VP9);
        let e = s.execute("media.replace", &json!({"id": id, "name": "x.mp4", "data": base64_encode(H264)})).expect_err("refused");
        assert!(e.to_string().contains("doesn't play"), "{e}");
        let e = s.execute("media.replace", &json!({"id": id, "name": "x.webm", "data": base64_encode(b"junk")})).expect_err("refused");
        assert!(!e.to_string().is_empty());
        let info = s.execute("media.info", &json!({"id": id})).expect("info");
        assert_eq!(info["name"], "clip.webm");
    }

    #[test]
    fn replace_finds_the_clip_on_a_named_slide() {
        let (mut s, id) = session_with("clip.mp4", H264);
        s.execute("slide.new", &json!({})).expect("new slide");
        let r = s.execute("media.replace", &json!({"id": id, "slide": 0, "name": "c.webm", "data": base64_encode(VP9)})).expect("replace");
        assert_eq!(r["name"], "c.webm");
        let e = s.execute("media.replace", &json!({"id": id, "slide": 99, "name": "c.webm", "data": base64_encode(VP9)})).expect_err("no slide");
        assert!(e.to_string().contains("no slide"));
        assert!(s.execute("media.replace", &json!({"slide": 0, "name": "c.webm", "data": base64_encode(VP9)})).is_err());
    }

    #[test]
    fn the_ffmpeg_path_is_not_a_command_parameter() {
        let mut s = Session::new();
        let e = s.execute("media.convertOptions", &json!({"ffmpegPath": "/bin/sh"})).expect_err("refused");
        assert!(e.to_string().contains("set in the app"));
        let r = s.execute("media.convertOptions", &json!({"ask": false})).expect("options");
        assert_eq!(r["ask"], false);
        assert!(!s.prefs.tinta_convert.ask);
        assert!(s.execute("media.convertOptions", &json!({"ask": "no"})).is_err());
    }

    #[test]
    fn a_damaged_setting_reads_as_the_defaults() {
        let p: crate::Prefs = serde_json::from_value(json!({"tintaConvert": 5})).expect("prefs");
        assert_eq!(p.tinta_convert, ConvertPrefs::default());
        let long = "x".repeat(MAX_PATH + 1);
        let p: crate::Prefs = serde_json::from_value(json!({"tintaConvert": {"ffmpegPath": long, "ask": false}})).expect("prefs");
        assert_eq!(p.tinta_convert, ConvertPrefs { ffmpeg_path: String::new(), ask: false });
    }

    #[test]
    fn convert_refuses_a_clip_that_already_plays() {
        let (mut s, id) = session_with("clip.webm", VP9);
        let e = s.execute("media.convert", &json!({"id": id})).expect_err("refused");
        assert!(e.to_string().contains("already plays"), "{e}");
    }

    /// With FFmpeg installed (not in CI), an H.264 clip is converted and plays.
    #[test]
    fn convert_with_a_real_ffmpeg() {
        if deckcraft_media::PATENTED_CODECS || deckcraft_media::transcode::find(None).is_err() {
            return;
        }
        let (mut s, id) = session_with("clip.mp4", H264);
        let r = s.execute("media.convert", &json!({"id": id})).expect("convert");
        assert_eq!(r["name"], "clip.webm");
        let info = s.execute("media.info", &json!({"id": id})).expect("info");
        assert_eq!(info["probe"]["playable"], true);
        assert_eq!(info["poster"], true);
    }
}
