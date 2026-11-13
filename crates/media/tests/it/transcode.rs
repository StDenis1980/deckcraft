//! Tinta Presenta: converting with a real FFmpeg. Skipped when none is installed (CI has none).

use std::sync::Arc;

use deckcraft_media::transcode::{self, Outcome};

fn ffmpeg() -> Option<transcode::Ffmpeg> {
    match transcode::find(None) {
        Ok(f) => Some(f),
        Err(e) => {
            eprintln!("skipped: {e}");
            None
        }
    }
}

#[test]
fn an_h264_mp4_becomes_a_webm_that_plays() {
    let Some(ff) = ffmpeg() else { return };
    let mp4 = include_bytes!("../fixtures/clip-h264.mp4");
    let info = deckcraft_media::probe(&Arc::new(mp4.to_vec())).expect("probe");
    let webm = transcode::convert(&ff, mp4, true, info.duration_ms).expect("convert");
    let out = deckcraft_media::probe(&Arc::new(webm.clone())).expect("probe webm");
    assert_eq!(out.container, "webm");
    let v = out.video.as_ref().expect("video");
    assert_eq!(v.codec, "VP9");
    assert!(v.decodable);
    assert!(out.duration_ms.abs_diff(info.duration_ms) < 200, "{} vs {}", out.duration_ms, info.duration_ms);
    deckcraft_media::poster_png(&Arc::new(webm), 0.0).expect("a frame decodes");
}

#[test]
fn an_aac_clip_becomes_opus() {
    let Some(ff) = ffmpeg() else { return };
    let m4a = include_bytes!("../fixtures/tone.m4a");
    let webm = transcode::convert(&ff, m4a, false, 0).expect("convert");
    let out = deckcraft_media::probe(&Arc::new(webm.clone())).expect("probe webm");
    assert!(out.video.is_none());
    let a = out.audio.as_ref().expect("audio");
    assert!(a.decodable, "{a:?}");
    deckcraft_media::audio::decode(&Arc::new(webm)).expect("decodes");
}

#[test]
fn a_cancelled_job_stops_and_cleans_up() {
    let Some(ff) = ffmpeg() else { return };
    let mp4 = include_bytes!("../fixtures/clip-h264.mp4");
    let job = transcode::Job::start(&ff, mp4, true, 1000).expect("start");
    let dir = job.folder().to_path_buf();
    job.cancel();
    assert!(matches!(job.wait(), Outcome::Cancelled | Outcome::Done(_)));
    drop(job);
    // The worker removes the folder once FFmpeg has stopped.
    for _ in 0..50 {
        if !dir.exists() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(!dir.exists());
}

#[test]
fn garbage_input_fails_with_a_message() {
    let Some(ff) = ffmpeg() else { return };
    let r = transcode::convert(&ff, b"this is not a video", true, 0);
    let e = r.expect_err("fails");
    assert!(!e.is_empty());
}
