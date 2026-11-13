//! Tinta Presenta: converting media this build can't decode (H.264, HEVC, AAC, WMV…) to WebM
//! (VP9 video, Opus audio), which it can, with the FFmpeg the person installed themselves.
//!
//! DeckCraft never ships FFmpeg or links to it: [`find`] looks for an `ffmpeg` program (a path from
//! the settings, then `PATH`, then the usual install folders), and [`Job`] runs it as a separate
//! process on a copy of the clip in a temporary folder, reading its `-progress` output for a
//! progress bar and killing it on [`Job::cancel`]. The patented formats are decoded by that
//! program on the person's computer; what comes back is VP9 and Opus only, and is checked with
//! our own [`crate::probe`] before anyone uses it.
//!
//! The program is started directly with fixed arguments (no shell), so a file name can't turn
//! into a command. Desktop only: there are no processes on the web.

use std::ffi::OsString;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

/// The largest clip converted, and the largest result read back.
pub const MAX_BYTES: u64 = 2 << 30;
/// How long `ffmpeg -version` / `-encoders` may take before the program is not one we can use.
const CHECK_TIMEOUT: Duration = Duration::from_secs(15);
/// The last part of FFmpeg's error output kept for the message shown when it fails.
const STDERR_TAIL: usize = 4096;
/// A WebM clip is never wider or taller than this (our VP9 decoder plays it in real time).
const MAX_WIDTH: u32 = 1920;
const MAX_HEIGHT: u32 = 1080;

/// A usable FFmpeg.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ffmpeg {
    pub path: PathBuf,
    /// The first line of `ffmpeg -version`.
    pub version: String,
}

/// Why no FFmpeg can be used.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum FindError {
    #[error("FFmpeg was not found")]
    NotFound,
    /// It runs, but has no VP9 or Opus encoder (some minimal builds leave them out).
    #[error("{path} can't write WebM (it has no {missing} encoder)")]
    NoWebm { path: PathBuf, missing: String },
    /// The configured program doesn't run, or isn't FFmpeg.
    #[error("{path} isn't a working FFmpeg: {error}")]
    Broken { path: PathBuf, error: String },
}

/// The program name on this system.
fn exe_name() -> &'static str {
    if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" }
}

/// Where to look, in order: the `configured` path, every `PATH` folder, then the folders the usual
/// installers use but a desktop app may not have on its `PATH` (Homebrew on macOS, `winget` on
/// Windows, Snap on Linux).
pub fn candidates(configured: Option<&Path>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(p) = configured.filter(|p| !p.as_os_str().is_empty()) {
        out.push(p.to_path_buf());
        return out;
    }
    if let Some(path) = std::env::var_os("PATH") {
        out.extend(std::env::split_paths(&path).filter(|d| !d.as_os_str().is_empty()).map(|d| d.join(exe_name())));
    }
    let mut extra: Vec<PathBuf> = Vec::new();
    if cfg!(target_os = "macos") {
        extra.extend(["/opt/homebrew/bin", "/usr/local/bin", "/opt/local/bin"].map(PathBuf::from));
    } else if cfg!(windows) {
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            extra.push(Path::new(&local).join("Microsoft").join("WinGet").join("Links"));
        }
        if let Some(home) = std::env::var_os("USERPROFILE") {
            extra.push(Path::new(&home).join("scoop").join("shims"));
        }
        extra.push(PathBuf::from(r"C:\ProgramData\chocolatey\bin"));
        extra.push(PathBuf::from(r"C:\ffmpeg\bin"));
    } else {
        extra.extend(["/usr/bin", "/usr/local/bin", "/snap/bin"].map(PathBuf::from));
    }
    for d in extra {
        let p = d.join(exe_name());
        if !out.contains(&p) {
            out.push(p);
        }
    }
    out
}

/// The first usable FFmpeg among [`candidates`]. With a `configured` path only that one is tried,
/// so a wrong setting is reported instead of quietly using another program.
pub fn find(configured: Option<&Path>) -> Result<Ffmpeg, FindError> {
    let explicit = configured.is_some_and(|p| !p.as_os_str().is_empty());
    let mut first_error = None;
    for path in candidates(configured) {
        if !explicit && !path.is_file() {
            continue;
        }
        match check(&path) {
            Ok(f) => return Ok(f),
            Err(e) if explicit => return Err(e),
            Err(e) => {
                first_error.get_or_insert(e);
            }
        }
    }
    Err(first_error.unwrap_or(FindError::NotFound))
}

/// Whether the program at `path` is an FFmpeg that can write WebM.
pub fn check(path: &Path) -> Result<Ffmpeg, FindError> {
    let broken = |error: String| FindError::Broken { path: path.to_path_buf(), error };
    if !path.is_file() {
        return Err(broken("no such file".into()));
    }
    let version = run_captured(path, &["-hide_banner", "-version"]).map_err(broken)?;
    let version = version.lines().next().unwrap_or("").trim().to_string();
    if !version.to_ascii_lowercase().starts_with("ffmpeg version") {
        return Err(broken("it doesn't answer like FFmpeg".into()));
    }
    let encoders = run_captured(path, &["-hide_banner", "-encoders"]).map_err(broken)?;
    let missing: Vec<&str> = ["libvpx-vp9", "libopus"].into_iter().filter(|e| !has_encoder(&encoders, e)).collect();
    if !missing.is_empty() {
        return Err(FindError::NoWebm { path: path.to_path_buf(), missing: missing.join(", ") });
    }
    Ok(Ffmpeg { path: path.to_path_buf(), version })
}

/// Whether `ffmpeg -encoders` output lists `name` (lines like ` V....D libvpx-vp9  libvpx VP9`).
fn has_encoder(list: &str, name: &str) -> bool {
    list.lines().any(|l| l.split_whitespace().nth(1) == Some(name))
}

fn command(program: &Path) -> Command {
    let mut c = Command::new(program);
    c.stdin(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        /// No console window flashing up for a program started from a window app.
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        c.creation_flags(CREATE_NO_WINDOW);
    }
    c
}

/// Runs `program args` and returns what it printed (stdout, then stderr), killing it after
/// [`CHECK_TIMEOUT`].
fn run_captured(program: &Path, args: &[&str]) -> Result<String, String> {
    let mut child = command(program).args(args).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e| e.to_string())?;
    let out = drain(child.stdout.take(), 1 << 20);
    let err = drain(child.stderr.take(), 1 << 20);
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if start.elapsed() > CHECK_TIMEOUT => {
                kill(&mut child);
                return Err("it didn't answer in time".into());
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(e) => {
                kill(&mut child);
                return Err(e.to_string());
            }
        }
    };
    let mut text = out.join().unwrap_or_default();
    text.push_str(&err.join().unwrap_or_default());
    if !status.success() {
        return Err(format!("it exited with {status}"));
    }
    Ok(text)
}

fn kill(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

/// Reads a pipe to its end on a thread, keeping at most `cap` bytes (the last ones).
fn drain(pipe: Option<impl Read + Send + 'static>, cap: usize) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let Some(mut pipe) = pipe else { return String::new() };
        let mut kept: Vec<u8> = Vec::new();
        let mut buf = [0u8; 8192];
        loop {
            match pipe.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    kept.extend_from_slice(buf.get(..n).unwrap_or_default());
                    if kept.len() > cap {
                        let cut = kept.len() - cap;
                        kept.drain(..cut);
                    }
                }
            }
        }
        String::from_utf8_lossy(&kept).into_owned()
    })
}

/// FFmpeg's arguments for converting `input` to the WebM file `output`: VP9 (8-bit 4:2:0, at most
/// 1920×1080, constant quality) and Opus (stereo at most) when `video`, Opus only otherwise. The
/// first video and audio streams are kept; progress goes to stdout as `key=value` lines.
pub fn args(input: &Path, output: &Path, video: bool) -> Vec<OsString> {
    let mut a: Vec<OsString> = ["-hide_banner", "-nostdin", "-y", "-i"].iter().map(OsString::from).collect();
    a.push(input.as_os_str().to_owned());
    let s = |v: &[&str]| v.iter().map(OsString::from).collect::<Vec<_>>();
    if video {
        a.extend(s(&["-map", "0:v:0", "-map", "0:a:0?", "-sn", "-dn"]));
        a.extend(s(&[
            "-c:v",
            "libvpx-vp9",
            "-pix_fmt",
            "yuv420p",
            "-crf",
            "32",
            "-b:v",
            "0",
            "-row-mt",
            "1",
            "-deadline",
            "good",
            "-cpu-used",
            "4",
            "-vf",
        ]));
        a.push(OsString::from(format!(
            "scale=w='min({MAX_WIDTH},iw)':h='min({MAX_HEIGHT},ih)':force_original_aspect_ratio=decrease,scale=trunc(iw/2)*2:trunc(ih/2)*2"
        )));
    } else {
        a.extend(s(&["-map", "0:a:0", "-vn", "-sn", "-dn"]));
    }
    a.extend(s(&["-c:a", "libopus", "-b:a", "128k", "-af", "aformat=channel_layouts=mono|stereo"]));
    a.extend(s(&["-progress", "pipe:1", "-nostats", "-f", "webm"]));
    a.push(output.as_os_str().to_owned());
    a
}

/// The media time (µs) a `-progress` line reports, if it is one (`out_time_us=…`; older FFmpeg
/// versions only write `out_time_ms=…`, also in microseconds).
pub fn progress_us(line: &str) -> Option<u64> {
    let (key, value) = line.trim().split_once('=')?;
    match key {
        "out_time_us" | "out_time_ms" => value.trim().parse::<u64>().ok(),
        _ => None,
    }
}

/// How a [`Job`] ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The WebM file, checked: it plays in DeckCraft.
    Done(PathBuf),
    Failed(String),
    Cancelled,
}

struct Shared {
    cancel: AtomicBool,
    /// Converted media time, µs.
    done_us: AtomicU64,
    outcome: Mutex<Option<Outcome>>,
}

/// A conversion running in the background. Dropping it cancels the conversion and removes its
/// temporary folder once FFmpeg has stopped.
pub struct Job {
    shared: Arc<Shared>,
    duration_us: u64,
    /// The temporary folder, removed on drop (after the worker has finished with it).
    dir: Arc<TempDir>,
}

impl std::fmt::Debug for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Job").field("progress", &self.progress()).field("outcome", &self.outcome()).finish()
    }
}

/// A folder removed when the last holder lets go.
struct TempDir(PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn temp_dir() -> Result<TempDir, String> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let base = std::env::temp_dir();
    for _ in 0..16 {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0);
        let dir = base.join(format!("tinta-convert-{}-{n}-{nanos:08x}", std::process::id()));
        match std::fs::create_dir(&dir) {
            Ok(()) => return Ok(TempDir(dir)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("{}: {e}", dir.display())),
        }
    }
    Err("no temporary folder could be made".into())
}

impl Job {
    /// Starts converting `input` (a clip `duration_ms` long, with a picture when `video`) with
    /// `ffmpeg`. Fails at once when the clip is too large or the temporary copy can't be written.
    pub fn start(ffmpeg: &Ffmpeg, input: &[u8], video: bool, duration_ms: u64) -> Result<Job, String> {
        if input.len() as u64 > MAX_BYTES {
            return Err("the clip is larger than 2 GB".into());
        }
        let dir = Arc::new(temp_dir()?);
        let src = dir.0.join("input");
        let dst = dir.0.join("output.webm");
        std::fs::write(&src, input).map_err(|e| e.to_string())?;
        let shared = Arc::new(Shared { cancel: AtomicBool::new(false), done_us: AtomicU64::new(0), outcome: Mutex::new(None) });
        let mut child = command(&ffmpeg.path)
            .args(args(&src, &dst, video))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("{}: {e}", ffmpeg.path.display()))?;
        let stdout = child.stdout.take();
        let stderr = drain(child.stderr.take(), STDERR_TAIL);
        let worker = Arc::clone(&shared);
        let keep = Arc::clone(&dir);
        let reader = {
            let shared = Arc::clone(&shared);
            std::thread::spawn(move || {
                let Some(out) = stdout else { return };
                for line in BufReader::new(out).lines() {
                    let Ok(line) = line else { break };
                    if let Some(us) = progress_us(&line) {
                        shared.done_us.store(us, Ordering::Relaxed);
                    }
                }
            })
        };
        std::thread::Builder::new()
            .name("tinta-convert".into())
            .spawn(move || {
                let outcome = supervise(&mut child, &worker.cancel, stderr, reader, &dst, video);
                *worker.outcome.lock().unwrap_or_else(PoisonError::into_inner) = Some(outcome);
                drop(keep);
            })
            .map_err(|e| e.to_string())?;
        Ok(Job { shared, duration_us: duration_ms.saturating_mul(1000), dir })
    }

    /// How far it is, 0–1 (0 while the length is unknown).
    pub fn progress(&self) -> f32 {
        if self.duration_us == 0 {
            return 0.0;
        }
        (self.shared.done_us.load(Ordering::Relaxed) as f64 / self.duration_us as f64).clamp(0.0, 1.0) as f32
    }

    /// Stops FFmpeg; [`Job::outcome`] becomes [`Outcome::Cancelled`] once it has.
    pub fn cancel(&self) {
        self.shared.cancel.store(true, Ordering::Relaxed);
    }

    /// How it ended, once it has. The WebM file of [`Outcome::Done`] stays until the job is
    /// dropped.
    pub fn outcome(&self) -> Option<Outcome> {
        self.shared.outcome.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }

    /// Waits for the end (the command-line path: no progress bar, no cancel).
    pub fn wait(&self) -> Outcome {
        loop {
            if let Some(o) = self.outcome() {
                return o;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// The temporary folder (for tests).
    #[doc(hidden)]
    pub fn folder(&self) -> &Path {
        &self.dir.0
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        self.cancel();
    }
}

fn supervise(
    child: &mut Child,
    cancel: &AtomicBool,
    stderr: std::thread::JoinHandle<String>,
    reader: std::thread::JoinHandle<()>,
    dst: &Path,
    video: bool,
) -> Outcome {
    let status = loop {
        if cancel.load(Ordering::Relaxed) {
            kill(child);
            let _ = reader.join();
            let _ = stderr.join();
            return Outcome::Cancelled;
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
            Err(e) => {
                kill(child);
                return Outcome::Failed(e.to_string());
            }
        }
    };
    let _ = reader.join();
    let tail = stderr.join().unwrap_or_default();
    if !status.success() {
        let last = tail.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("").trim();
        return Outcome::Failed(if last.is_empty() { format!("FFmpeg exited with {status}") } else { last.chars().take(300).collect() });
    }
    match verify(dst, video) {
        Ok(()) => Outcome::Done(dst.to_path_buf()),
        Err(e) => Outcome::Failed(e),
    }
}

/// Checks that FFmpeg's result is a clip DeckCraft plays: VP9/Opus WebM, with a picture when the
/// original had one.
pub fn verify(path: &Path, video: bool) -> Result<(), String> {
    let len = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
    if len == 0 || len > MAX_BYTES {
        return Err(format!("FFmpeg wrote {len} bytes"));
    }
    let bytes = Arc::new(std::fs::read(path).map_err(|e| e.to_string())?);
    let info = crate::probe(&bytes).map_err(|e| e.to_string())?;
    let ok = if video { info.video.as_ref().is_some_and(|v| v.decodable) } else { info.audio.as_ref().is_some_and(|a| a.decodable) };
    if ok { Ok(()) } else { Err("the converted clip doesn't play".into()) }
}

/// Converts `input` and returns the WebM bytes, waiting for FFmpeg (the command-line path).
pub fn convert(ffmpeg: &Ffmpeg, input: &[u8], video: bool, duration_ms: u64) -> Result<Vec<u8>, String> {
    let job = Job::start(ffmpeg, input, video, duration_ms)?;
    match job.wait() {
        Outcome::Done(path) => std::fs::read(path).map_err(|e| e.to_string()),
        Outcome::Failed(e) => Err(e),
        Outcome::Cancelled => Err("cancelled".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_lines_are_read_in_microseconds() {
        assert_eq!(progress_us("out_time_us=1500000"), Some(1_500_000));
        assert_eq!(progress_us("out_time_ms=2500000\n"), Some(2_500_000));
        assert_eq!(progress_us("out_time=00:00:01.500000"), None);
        assert_eq!(progress_us("out_time_us=N/A"), None);
        assert_eq!(progress_us("progress=end"), None);
        assert_eq!(progress_us(""), None);
    }

    #[test]
    fn the_encoder_list_is_read_by_column() {
        let list = " V....D libvpx-vp9           libvpx VP9 (codec vp9)\n A....D libopus              libopus Opus (codec opus)\n";
        assert!(has_encoder(list, "libvpx-vp9"));
        assert!(has_encoder(list, "libopus"));
        assert!(!has_encoder(list, "libvpx"));
        assert!(!has_encoder(" V....D libx264   says libvpx-vp9 in its description", "libvpx-vp9"));
    }

    #[test]
    fn arguments_keep_the_paths_whole_and_ask_for_webm() {
        let a = args(Path::new("/tmp/a b/input"), Path::new("/tmp/a b/output.webm"), true);
        let s: Vec<String> = a.iter().map(|x| x.to_string_lossy().into_owned()).collect();
        assert!(s.contains(&"/tmp/a b/input".to_string()));
        assert_eq!(s.last().map(String::as_str), Some("/tmp/a b/output.webm"));
        assert!(s.windows(2).any(|w| w == ["-c:v", "libvpx-vp9"]));
        assert!(s.windows(2).any(|w| w == ["-c:a", "libopus"]));
        assert!(s.windows(2).any(|w| w == ["-f", "webm"]));
        let audio: Vec<String> = args(Path::new("i"), Path::new("o"), false).iter().map(|x| x.to_string_lossy().into_owned()).collect();
        assert!(audio.contains(&"-vn".to_string()));
        assert!(!audio.iter().any(|x| x == "libvpx-vp9"));
    }

    #[test]
    fn a_configured_path_is_the_only_one_tried() {
        let missing = Path::new("/nonexistent/tinta/ffmpeg");
        assert_eq!(candidates(Some(missing)), vec![missing.to_path_buf()]);
        assert!(matches!(find(Some(missing)), Err(FindError::Broken { .. })));
        assert!(!candidates(None).is_empty() || std::env::var_os("PATH").is_none());
    }

    #[test]
    fn a_program_that_is_not_ffmpeg_is_refused() {
        // Any program that exits successfully without saying "ffmpeg version".
        let not_ffmpeg = if cfg!(windows) { PathBuf::from(r"C:\Windows\System32\whoami.exe") } else { PathBuf::from("/bin/true") };
        if !not_ffmpeg.is_file() {
            return;
        }
        assert!(matches!(check(&not_ffmpeg), Err(FindError::Broken { .. })));
    }
}
