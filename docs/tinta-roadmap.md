# Tinta Presenta — Tinta Office roadmap

> **Status:** 2026-10-27 · approved by the owner (Denys) · **Revision history:** 2026-10-27 created; 2026-10-27 stage 1 started: upstream sync (88c2cef), unplayable videos say why and open in the system player.

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
| 5 | Presenta's translations (only in this fork, 9 languages) offered upstream, so syncs stop conflicting | upstream | 1 |
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
| Windows installers | 1 | Desktop shortcuts for Texto and Calculo; Presenta registered for .pptx. |
| Code signing | 2 | Windows (Certum Open Source or Azure Trusted Signing; scripts exist), macOS (Apple Developer + notarization). Owner decision and payment. |
| Automated tests | 1–2 | Coverage audit per app; tests for every fork change (brand, languages, installers); a smoke test of each release installer on Windows; investigate the flaky tests upstream. |
| User guide | 2 | A short "first steps" guide per app in Spanish and Ukrainian on tintaoffice.com (Correo has one). |
| Fork documentation | 1 | This file: what the fork changes, how to build a release, how to sync. |
| Translations | 1–2 | Ukrainian reviewed by the owner, Spanish by a native tester, ca/gl/eu through communities. Marked as drafts until reviewed. |
| Feedback | 1 | Help › Report a problem → the fork's issues or hola@tintaoffice.com; issue templates in Spanish and Ukrainian. |

Rules: upstream first (fixes useful to everyone go to storytold as PRs); every change ships with
automated tests and updated docs in the same PR; files people exchange (.docx, .xlsx, .pptx, PDF)
come first; Spain and Ukraine first (UI language, spelling, number/date formats, national
certificates).
