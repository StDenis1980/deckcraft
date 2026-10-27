//! Test-only rendering through one GPU at a time.
//!
//! Each kittest harness creates its own wgpu device on its first render, and kittest prefers a
//! software adapter. On Windows that is Direct3D WARP, which crashes the test process
//! (STATUS_ACCESS_VIOLATION in d3d10warp.dll) when many tests render at once. Rendering here
//! takes a process-wide lock, so only one harness talks to the adapter at a time.

use std::sync::{Mutex, PoisonError};

use egui_kittest::Harness;

static GPU: Mutex<()> = Mutex::new(());

/// [`Harness::render`], one harness at a time.
pub fn render<State>(harness: &mut Harness<'_, State>) -> Result<image::RgbaImage, String> {
    let _gpu = GPU.lock().unwrap_or_else(PoisonError::into_inner);
    harness.render()
}
