//! Tests that render a harness run one at a time in each test process.
//!
//! `Harness::render` creates a wgpu device on its first call, and egui_kittest prefers a CPU
//! adapter. When `cargo test` runs several of these tests on parallel threads, the test binary can
//! exit with `0xc0000005` (`STATUS_ACCESS_VIOLATION`). Under cargo-nextest each test is its own
//! process and the lock has no effect.
//!
//! Every test that calls `render()` on a harness takes [`gpu_lock`] on its first line
//! (`let _gpu = crate::gpu_lock::gpu_lock();`) and holds the guard for its whole run.
//!
//! Taking turns was not enough on Windows: the adapter there is Direct3D WARP, and creating and
//! dropping a WARP device per test still crashed late in the run inside `d3d10warp.dll`. So the
//! harnesses of those tests are built with [`builder`], whose renderer draws on one device that
//! the whole test process shares and never drops.

use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};

use egui_kittest::wgpu::{WgpuTestRenderer, create_render_state, default_wgpu_setup};
use egui_kittest::{HarnessBuilder, LazyRenderer};

static GPU_LOCK: Mutex<()> = Mutex::new(());

/// Returns a guard that the test holds for its whole run, so tests that render run one at a time.
/// A test that failed while holding it does not block the tests after it.
pub(crate) fn gpu_lock() -> MutexGuard<'static, ()> {
    GPU_LOCK.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A harness builder whose renderer draws on the process's one shared device. The device is
/// created on the first render (so tests that never render need no adapter) and lives until
/// the process exits.
pub(crate) fn builder<State>() -> HarnessBuilder<State> {
    HarnessBuilder::default().renderer(LazyRenderer::new(renderer))
}

/// A fresh egui renderer on the shared device: each harness keeps its own textures.
fn renderer() -> WgpuTestRenderer {
    static SHARED: OnceLock<egui_wgpu::RenderState> = OnceLock::new();
    let options = egui_wgpu::RendererOptions::PREDICTABLE;
    let shared = SHARED.get_or_init(|| create_render_state(default_wgpu_setup(), options));
    let renderer = egui_wgpu::Renderer::new(&shared.device, shared.target_format, options);
    WgpuTestRenderer::from_render_state(egui_wgpu::RenderState { renderer: Arc::new(egui::mutex::RwLock::new(renderer)), ..shared.clone() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_panicked_holder_does_not_block_the_next_test() {
        let _ = std::thread::spawn(|| {
            let _gpu = gpu_lock();
            panic!("a GPU test failed");
        })
        .join();
        drop(gpu_lock());
    }

    #[test]
    fn holders_take_turns() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static INSIDE: AtomicUsize = AtomicUsize::new(0);
        static MAX: AtomicUsize = AtomicUsize::new(0);
        let threads: Vec<_> = (0..8)
            .map(|_| {
                std::thread::spawn(|| {
                    let _gpu = gpu_lock();
                    let now = INSIDE.fetch_add(1, Ordering::SeqCst) + 1;
                    MAX.fetch_max(now, Ordering::SeqCst);
                    std::thread::sleep(std::time::Duration::from_millis(2));
                    INSIDE.fetch_sub(1, Ordering::SeqCst);
                })
            })
            .collect();
        for t in threads {
            assert!(t.join().is_ok());
        }
        assert_eq!(MAX.load(Ordering::SeqCst), 1);
    }
}
