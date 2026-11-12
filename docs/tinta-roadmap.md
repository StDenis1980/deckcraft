# Tinta Presenta — Tinta Office roadmap

> **Status:** 2026-11-12 · approved by the owner (Denys) · **Revision history:** 2026-10-27 created; 2026-10-27 stage 1 started: upstream sync (88c2cef), unplayable videos say why and open in the system player; 2026-11-03 Flatpak bundle manifest under the Tinta id; 2026-11-03 plan revision 2 (owner-approved): Preview 3 and a closed beta first, stage gates, next steps per stage; 2026-11-03 owner decisions recorded: Preview 3 on 12 November, Certum signing from January, beta sign-up form on tintaoffice.com/beta; 2026-11-12 Preview 4 work: media playback options kept in .pptx (#74); 2026-11-12 a .pptx test set written from the spec.

Tinta Presenta is the Tinta Office build of DeckCraft and replaces Microsoft PowerPoint. Upstream keeps the
feature-parity roadmap ([ROADMAP.md](../ROADMAP.md), [roadmap.md](roadmap.md),
[gaps.md](gaps.md)); this page lists only what the fork does and which upstream gaps matter most
for users in Spain and Ukraine. It is a fork-only file, so upstream syncs never conflict on it.

## Where it stands (2026-10-27)

Ready for real work ≈ 55% (essentials user ≈ 62%, upstream estimate); 68 commits behind upstream.
The patented codecs (H.264, HEVC, AAC) are off by default (`patented-codecs` feature, deckcraft#7),
so MP4 video in other people's decks does not play. The priority now is not losing work and not
crashing on Windows.

## Priorities

| # | Task | Where | Stage |
|---|---|---|---|
| 1 | No data loss, no crashes: the save prompt on close (#60), Windows GPU crashes (#40, #59), image paste (#53); investigate the STATUS_ACCESS_VIOLATION in parallel tests on the owner's PC | upstream + checks on the PC | 1 |
| 2 | MP4 through the operating system's codecs (Media Foundation on Windows, AVFoundation on macOS), whose patent licence the OS carries; until then a clear message instead of an empty frame | fork, then upstream | 1–2 |
| 3 | Real .pptx: a test set of decks, round-trip losses (#70, #74, #77), EMF and SVG pictures | upstream | 1–2 |
| 4 | Vector PDF and handout printing | upstream | 2 |
| 5 | Presenta's translations (only in this fork, 12 languages) offered upstream, so syncs stop conflicting | upstream | 1 |
| 6 | Tinta presentation templates (our design, azulejo motifs) | fork | 2 |
| 7 | Depth: font weights (#69), chart data, the Animation Pane, presenter view on the second display | upstream | 2–3 |
| 8 | Opening .odp and legacy .ppt | upstream | 3 |

## Stage 1 progress

- 2026-10-27: synced with upstream 88c2cef (upstream split the big UI files into modules; the
  fork's translations moved with them). #60 (save prompt on close) and #53 (image paste) are
  already fixed upstream. A video this build can't decode (H.264/HEVC without
  `patented-codecs`) now says so over its poster, and the control bar's link runs
  `media.openExternal`, which opens the clip in the system's player (Media Player on Windows,
  whose codec licence the OS carries; a download on the web). In-slide decoding through Media
  Foundation would need `unsafe` code, which the workspace forbids, so it waits for an owner and
  upstream decision. Next: media playback options lost on .pptx save (#74).
- 2026-11-12 (Preview 4): media playback options survive a .pptx save (#74). The writer now
  emits each clip's `p:video`/`p:audio` > `p:cMediaNode` (volume, mute, Play Across Slides,
  Hide While Not Playing, Loop until Stopped, Rewind after Playing, Play Full Screen) and Start
  Automatically as a Play effect at the head of the main sequence; the reader turns that effect
  back into the flag. Rewind is written as the node's `fill="remove"` from the spec; it still
  needs a check in PowerPoint itself. Offer upstream.
- 2026-11-12 (Preview 4): a test set of decks written by hand from ECMA-376
  (`crates/pptx/tests/spec/`, run by `tests/it/spec_corpus.rs`): text (runs, levels, uk/es,
  links, lists), shapes (presets, gradients, connectors, groups, freeforms), a table with merged
  cells, pictures (crop, alt text, a shared image), a video (poster, trim, bookmark, playback
  options), animations with a timed transition, and hidden slides, sections, notes and a slide
  jump. Each deck is checked on import and after a save and reopen. All seven pass today; new
  losses found in real decks get a deck here first. Next: #70 and #77.

## Plan revision 2 (2026-11-03, approved by the owner)

Since 2026-10-27 the suite shipped more than stage 1 planned, but no new build has reached people:
the last releases are from 2026-10-26/27. So November is about delivery, not new features.

Suite priorities for November (stage 1), in order:

1. **Preview 3 for all five apps on 12 November**, after the 9 November upstream sync: Windows MSI
   and portable, Linux .deb, AppImage and .flatpak, the extra dictionary pack as its own file,
   release notes in es/uk/en, the site's Download section; every installer smoke-tested on the
   owner's PC first.
2. **Closed beta in Spain** from Preview 3: a sign-up form on tintaoffice.com/beta and a guide
   for testers at /beta/testers/ (what to check, how to report), 20–50 autónomos and 1–2 gestorías, Ukrainians in Spain
   as testers; weekly triage of error reports and issues.
3. **Tinta Presenta catches up** (it has barely moved since 2026-10-27): #74, a .pptx test set, #70/#77.
4. **Tinta Correo finishes M5**: calendar and contacts from .pst, export, the Outlook migration guide.
5. **Translation review**: Ukrainian by the owner, Spanish by the first testers, ca/gl/eu through
   Softcatalà and Proxecto Trasno.
6. **Tests and stability**: a coverage audit of fork changes per app, an installer smoke-test script,
   fuzzing file opening before each release.
7. **A smaller fork**: port Texto's proofing to upstream's engine, see the open upstream PRs through
   (Calculo ×4, Texto #4952), ask GitHub Support to re-enable Actions.

Gates: stage 2 starts when Preview 3 is out and its installers passed the smoke test; the **public
beta (February 2027)** starts when the Windows installers are code-signed and the es/uk
translations are reviewed; 1.0 when no critical reports remain. Public beta: Tinta PDF, Texto and
Calculo; Presenta if its test set passes by January; Correo stays a preview without promotion.
Ukraine not before Q2 2027 (KEP), the rest of Europe in H2 2027; no paid ads or support sales
before the public beta.

This app's next steps:

| Stage | Next steps |
|---|---|
| 1 — November 2026 | Media playback options kept on .pptx save (#74); a test set of decks written from the spec, like Texto's and Calculo's; round-trip losses #70 and #77, EMF and SVG pictures |
| 2 — December 2026 – January 2027 | Tinta presentation templates (our design, azulejo motifs); vector PDF and handout printing |
| 3 — 2027 | Animation Pane, presenter view on the second display; .odp and legacy .ppt |

Code signing for Windows, decided 2026-11-03: Certum Open Source, registered at the start of January
2027 once the owner has his new ID card (December); Preview 3 and the closed beta stay unsigned.
Certum's identity check takes days to two weeks, so signed installers are expected late January,
before the public beta in February.

Beta sign-up, decided 2026-11-03: a form on tintaoffice.com
([tintaoffice.com/beta](https://tintaoffice.com/beta/), with a testers' guide at `/beta/testers/`).

Preview 3 date, decided 2026-11-03: 12 November 2026.

Owner decisions still open: which apps enter the public beta, time
for the Ukrainian review, the video channel and playlists.

## Suite-wide work (all five Tinta Office apps)

Stages: **1** = November 2026 (Preview 3), **2** = December 2026 – January 2027 (public beta of
the most ready apps), **3** = 2027 (1.0). Dates are indicative.

| Work | Stage | What |
|---|---|---|
| Upstream sync | ongoing | Weekly, Mondays. Catch up before Preview 3. |
| Our changes upstream | 1 | Offer storytold the configurable brand, Presenta's translations, the codec feature flag and the ca/gl/eu catalogs (craft-i18n#53), so the fork stays small. |
| GitHub Actions | 1 | GitHub disabled Actions in pdfcraft, wordcraft and deckcraft; ask GitHub Support. Until then, run the gates locally. |
| Releases | 1 | A preview every month for all five apps, release notes in es/uk/en, one build script. |
| Linux | 1 | AppImage + .deb with Tinta names for all five (built in the cloud); Flatpak in stage 2. |
| Flatpak | 2 | Bundle manifest under the Tinta id (`packaging/linux/flatpak/io.github.stdenis1980.tintapresenta.bundle.yml`, generated from upstream's by `packaging/linux/tinta-flatpak.sh generate`; `tinta-flatpak.sh bundle` turns the Linux tarball into a `.flatpak`). Requested by Vilna OS (tintaoffice-web#14): bundles in releases by April 2027; Flathub only with Denys's approval. |
| Windows installers | 1 | Desktop shortcuts for Texto and Calculo; Presenta registered for .pptx. |
| Code signing | 2 | Windows: Certum Open Source (decided 2026-11-03), registration early January 2027, signed installers before the February public beta; scripts exist. macOS (Apple Developer + notarization) later. |
| Automated tests | 1–2 | Coverage audit per app; tests for every fork change (brand, languages, installers); a smoke test of each release installer on Windows; investigate the flaky tests upstream. |
| User guide | 2 | A short "first steps" guide per app in Spanish and Ukrainian on tintaoffice.com (Correo has one). |
| Fork documentation | 1 | This file: what the fork changes, how to build a release, how to sync. |
| Translations | 1–2 | Ukrainian reviewed by the owner, Spanish by a native tester, ca/gl/eu through communities. Marked as drafts until reviewed. |
| Feedback | 1 | Help › Report a problem → the fork's issues or hola@tintaoffice.com; issue templates in Spanish and Ukrainian. |
| Error reports | 1 | Automatic, only with consent (decided 2026-10-30): the first start asks "Send error reports automatically?"; File › Options (Preferences) changes it, shows the installation id and deletes the reports from the server. Panics and a window busy for 15 s write a scrubbed report (no document text, file names, paths, e-mail addresses, long numbers) to `error-reports/pending` in the app's data folder; while allowed it goes to report.tintaoffice.com (tintaoffice-web `report-worker/`: R2 in the EU, 90 days, one GitHub issue per distinct crash labelled `auto-report`). Presenta: `crates/engine/src/tinta_reports.rs` (`file.errorReports`), `crates/ui-egui/src/tinta_reports_ui.rs`, `apps/deckcraft/src/tinta_report.rs`, address in `.cargo/config.toml` (`TINTA_REPORT_URL`). A distribution can answer the first-start question for all five apps (2026-11-03, for Vilna OS, tintaoffice-web#16): `TINTA_OFFICE_REPORTS=true|false` (seen inside Flatpak too), or a line `enabled=true|false` in `$XDG_CONFIG_DIRS/tinta-office/reports.conf` (default `/etc/xdg`); read only while the person has not answered, saved as their answer, and changeable in the settings. |

Rules: upstream first (fixes useful to everyone go to storytold as PRs); every change ships with
automated tests and updated docs in the same PR; files people exchange (.docx, .xlsx, .pptx, PDF)
come first; Spain and Ukraine first (UI language, spelling, number/date formats, national
certificates).
