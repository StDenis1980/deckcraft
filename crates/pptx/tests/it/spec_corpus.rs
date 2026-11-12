//! The spec corpus (`tests/spec/`): decks written by hand from ECMA-376, not by our writer.
//! Each deck is packaged here, imported, checked, saved, imported again and checked again, so a
//! loss on either side shows up against XML DeckCraft did not produce.

use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use deckcraft_model::anim::{AnimClass, AnimStart};
use deckcraft_model::style::Fill;
use deckcraft_model::text::Action;
use deckcraft_model::{Geom, PhType, Presentation, Shape, ShapeKind};

const R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const CT: &str = "application/vnd.openxmlformats-officedocument.presentationml";

/// One deck of the corpus: its folder under `tests/spec/decks`, the layout each slide uses when the
/// folder has no `.rels` for it (1 = Title and Content, 2 = Blank), the slide size in EMU and its
/// sections (name, 1-based slide numbers).
struct Deck {
    name: &'static str,
    layouts: &'static [u32],
    size: (u32, u32),
    sections: &'static [(&'static str, &'static [usize])],
}

const WIDE: (u32, u32) = (12_192_000, 6_858_000);

fn spec_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/spec")
}

fn read(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// A 4x4 PNG of one colour, made here so the corpus holds no binary files.
fn png() -> Vec<u8> {
    let mut out = Cursor::new(vec![]);
    image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(4, 4, image::Rgb([0xD9, 0x82, 0x2B])))
        .write_to(&mut out, image::ImageFormat::Png)
        .expect("png");
    out.into_inner()
}

fn rels(entries: &[(String, &str, String)]) -> Vec<u8> {
    let mut s = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
    );
    for (id, ty, target) in entries {
        s.push_str(&format!(r#"<Relationship Id="{id}" Type="{ty}" Target="{target}"/>"#));
    }
    s.push_str("</Relationships>");
    s.into_bytes()
}

/// Package a deck: the shared parts from `common/`, the deck's own parts, generated content types,
/// package and presentation relationships, and the media its slides point at.
fn package(deck: &Deck) -> Vec<u8> {
    let dir = spec_dir();
    let own = dir.join("decks").join(deck.name);
    let common = dir.join("common");
    let mut files: Vec<(String, Vec<u8>)> = vec![];
    let mut overrides: Vec<(String, String)> = vec![];
    let add = |files: &mut Vec<(String, Vec<u8>)>, name: &str, data: Vec<u8>| files.push((name.to_string(), data));

    add(&mut files, "_rels/.rels", rels(&[("rId1".into(), &format!("{R}/officeDocument"), "ppt/presentation.xml".into())]));
    add(&mut files, "ppt/theme/theme1.xml", read(&common.join("theme1.xml")));
    overrides.push(("/ppt/theme/theme1.xml".into(), "application/vnd.openxmlformats-officedocument.theme+xml".into()));
    add(&mut files, "ppt/slideMasters/slideMaster1.xml", read(&common.join("slideMaster1.xml")));
    overrides.push(("/ppt/slideMasters/slideMaster1.xml".into(), format!("{CT}.slideMaster+xml")));
    add(
        &mut files,
        "ppt/slideMasters/_rels/slideMaster1.xml.rels",
        rels(&[
            ("rId1".into(), &format!("{R}/slideLayout"), "../slideLayouts/slideLayout1.xml".into()),
            ("rId2".into(), &format!("{R}/slideLayout"), "../slideLayouts/slideLayout2.xml".into()),
            ("rId3".into(), &format!("{R}/theme"), "../theme/theme1.xml".into()),
        ]),
    );
    for n in 1..=2 {
        add(&mut files, &format!("ppt/slideLayouts/slideLayout{n}.xml"), read(&common.join(format!("slideLayout{n}.xml"))));
        add(
            &mut files,
            &format!("ppt/slideLayouts/_rels/slideLayout{n}.xml.rels"),
            rels(&[("rId1".into(), &format!("{R}/slideMaster"), "../slideMasters/slideMaster1.xml".into())]),
        );
        overrides.push((format!("/ppt/slideLayouts/slideLayout{n}.xml"), format!("{CT}.slideLayout+xml")));
    }

    let mut pres_rels = vec![
        ("rId1".to_string(), format!("{R}/slideMaster"), "slideMasters/slideMaster1.xml".to_string()),
        ("rId2".to_string(), format!("{R}/theme"), "theme/theme1.xml".to_string()),
    ];
    let has_notes = own.join("notesSlide1.xml").exists();
    if has_notes {
        add(&mut files, "ppt/notesMasters/notesMaster1.xml", read(&common.join("notesMaster1.xml")));
        add(
            &mut files,
            "ppt/notesMasters/_rels/notesMaster1.xml.rels",
            rels(&[("rId1".into(), &format!("{R}/theme"), "../theme/theme1.xml".into())]),
        );
        overrides.push(("/ppt/notesMasters/notesMaster1.xml".into(), format!("{CT}.notesMaster+xml")));
        pres_rels.push(("rId3".into(), format!("{R}/notesMaster"), "notesMasters/notesMaster1.xml".into()));
    }

    let mut sld_ids = String::new();
    for (i, layout) in deck.layouts.iter().enumerate() {
        let n = i + 1;
        add(&mut files, &format!("ppt/slides/slide{n}.xml"), read(&own.join(format!("slide{n}.xml"))));
        let own_rels = own.join(format!("slide{n}.xml.rels"));
        let slide_rels = if own_rels.exists() {
            read(&own_rels)
        } else {
            rels(&[("rId1".into(), &format!("{R}/slideLayout"), format!("../slideLayouts/slideLayout{layout}.xml"))])
        };
        add(&mut files, &format!("ppt/slides/_rels/slide{n}.xml.rels"), slide_rels);
        overrides.push((format!("/ppt/slides/slide{n}.xml"), format!("{CT}.slide+xml")));
        pres_rels.push((format!("rId{}", 10 + n), format!("{R}/slide"), format!("slides/slide{n}.xml")));
        sld_ids.push_str(&format!(r#"<p:sldId id="{}" r:id="rId{}"/>"#, 255 + n, 10 + n));
        let notes = own.join(format!("notesSlide{n}.xml"));
        if notes.exists() {
            add(&mut files, &format!("ppt/notesSlides/notesSlide{n}.xml"), read(&notes));
            add(&mut files, &format!("ppt/notesSlides/_rels/notesSlide{n}.xml.rels"), read(&own.join(format!("notesSlide{n}.xml.rels"))));
            overrides.push((format!("/ppt/notesSlides/notesSlide{n}.xml"), format!("{CT}.notesSlide+xml")));
        }
    }
    let rel_refs: Vec<(String, &str, String)> = pres_rels.iter().map(|(i, t, g)| (i.clone(), t.as_str(), g.clone())).collect();
    add(&mut files, "ppt/_rels/presentation.xml.rels", rels(&rel_refs));

    let sections = if deck.sections.is_empty() {
        String::new()
    } else {
        let mut s = String::from(
            r#"<p:extLst><p:ext uri="{521415D9-36F7-43E2-AB2F-B90AF26B5E84}"><p14:sectionLst xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main">"#,
        );
        for (k, (name, slides)) in deck.sections.iter().enumerate() {
            s.push_str(&format!(r#"<p14:section name="{name}" id="{{0A1B2C3D-0000-4000-8000-00000000000{k}}}"><p14:sldIdLst>"#));
            for n in *slides {
                s.push_str(&format!(r#"<p14:sldId id="{}"/>"#, 255 + n));
            }
            s.push_str("</p14:sldIdLst></p14:section>");
        }
        s.push_str("</p14:sectionLst></p:ext></p:extLst>");
        s
    };
    let notes_ids = if has_notes { r#"<p:notesMasterIdLst><p:notesMasterId r:id="rId3"/></p:notesMasterIdLst>"# } else { "" };
    let presentation = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="{R}" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" saveSubsetFonts="1"><p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst>{notes_ids}<p:sldIdLst>{sld_ids}</p:sldIdLst><p:sldSz cx="{}" cy="{}"/><p:notesSz cx="6858000" cy="9144000"/>{sections}</p:presentation>"#,
        deck.size.0, deck.size.1
    );
    add(&mut files, "ppt/presentation.xml", presentation.into_bytes());
    overrides.push(("/ppt/presentation.xml".into(), format!("{CT}.presentation.main+xml")));

    // Media the slides point at, generated rather than stored.
    let wants = |part: &str| files.iter().any(|(n, d)| n.ends_with(".rels") && String::from_utf8_lossy(d).contains(part));
    let media: Vec<(&str, Vec<u8>)> = [
        ("image1.png", wants("media/image1.png").then(png)),
        ("media1.mp4", wants("media/media1.mp4").then(|| include_bytes!("../../../media/tests/fixtures/clip-h264.mp4").to_vec())),
    ]
    .into_iter()
    .filter_map(|(n, d)| d.map(|d| (n, d)))
    .collect();
    for (n, d) in media {
        add(&mut files, &format!("ppt/media/{n}"), d);
    }

    let mut ct = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Default Extension="png" ContentType="image/png"/><Default Extension="mp4" ContentType="video/mp4"/>"#,
    );
    for (part, ty) in &overrides {
        ct.push_str(&format!(r#"<Override PartName="{part}" ContentType="{ty}"/>"#));
    }
    ct.push_str("</Types>");

    let mut z = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let o = zip::write::SimpleFileOptions::default();
    z.start_file("[Content_Types].xml", o).expect("start");
    z.write_all(ct.as_bytes()).expect("write");
    for (n, d) in &files {
        z.start_file(n.as_str(), o).expect("start");
        z.write_all(d).expect("write");
    }
    z.finish().expect("finish").into_inner()
}

/// Import the deck, run `check`, save, import the result and run `check` again.
fn check_deck(deck: &Deck, check: impl Fn(&Presentation, &str)) {
    let bytes = package(deck);
    let p = deckcraft_pptx::import(&bytes).unwrap_or_else(|e| panic!("{}: import failed: {e}", deck.name));
    check(&p, "as written");
    let saved = deckcraft_pptx::export(&p).unwrap_or_else(|e| panic!("{}: export failed: {e}", deck.name));
    let q = deckcraft_pptx::import(&saved).unwrap_or_else(|e| panic!("{}: reimport failed: {e}", deck.name));
    check(&q, "after a save");
}

fn named<'a>(shapes: &'a [Shape], name: &str) -> &'a Shape {
    shapes.iter().find(|s| s.name == name).unwrap_or_else(|| panic!("no shape {name:?} in {:?}", shapes.iter().map(|s| &s.name).collect::<Vec<_>>()))
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.01
}

#[test]
fn text_runs_levels_languages_and_links() {
    let deck = Deck { name: "text", layouts: &[1, 2], size: WIDE, sections: &[] };
    check_deck(&deck, |p, when| {
        assert_eq!(p.slides.len(), 2, "{when}");
        let s = &p.slides[0];
        let title = s.shapes.iter().find(|s| s.ph.as_ref().is_some_and(|ph| ph.kind == PhType::Title)).expect("title");
        assert_eq!(title.text.as_ref().map(|t| t.text()).as_deref(), Some("Квартальний звіт"), "{when}");
        let body = s.shapes.iter().find(|s| s.ph.as_ref().is_some_and(|ph| ph.idx == 1)).expect("body");
        let t = body.text.as_ref().expect("body text");
        assert_eq!(t.paragraphs.len(), 4, "{when}");
        assert_eq!(t.paragraphs.iter().map(|p| p.level).collect::<Vec<_>>(), [0, 1, 1, 0], "{when}");
        assert_eq!(t.paragraphs[1].text(), "Señal ñ, acentos á é í ó ú", "{when}");
        assert_eq!(t.paragraphs[2].text(), "Україна: ґ є і ї", "{when}");
        let runs = &t.paragraphs[0].runs;
        assert_eq!(runs.iter().map(|r| r.text.as_str()).collect::<String>(), "Ventas en España: +12 %", "{when}");
        assert_eq!(runs[0].props.bold, Some(true), "{when}");
        let last = runs.last().expect("run");
        assert_eq!((last.props.italic, last.props.size, last.props.underline.as_deref()), (Some(true), Some(32.0), Some("sng")), "{when}");
        assert!(matches!(&last.props.fill, Some(Fill::Solid { .. })), "{when}: run colour {:?}", last.props.fill);
        let link = t.paragraphs[3].runs[0].props.link.as_ref().expect("hyperlink");
        assert!(matches!(&link.action, Action::Url { url } if url == "https://tintaoffice.com/"), "{when}: {:?}", link.action);

        let tb = named(&p.slides[1].shapes, "TextBox 1");
        assert!(tb.text_box, "{when}");
        let x = tb.xfrm.expect("xfrm");
        assert!(close(x.x, 100.0) && close(x.w, 500.0), "{when}: {x:?}");
        let t = tb.text.as_ref().expect("text");
        assert_eq!(t.paragraphs.len(), 3, "{when}");
        assert!(t.paragraphs[0].props.align.is_some(), "{when}: centred");
        assert!(t.paragraphs[0].props.line_spacing.is_some(), "{when}: 150% line spacing");
        assert!(t.paragraphs[1].props.bullet.is_some(), "{when}: numbered");
        assert!(t.paragraphs[2].text().contains("Two") && t.paragraphs[2].text().contains("after a break"), "{when}: {:?}", t.paragraphs[2].text());
    });
}

#[test]
fn shapes_geometry_fills_lines_groups_and_freeforms() {
    let deck = Deck { name: "shapes", layouts: &[2], size: WIDE, sections: &[] };
    check_deck(&deck, |p, when| {
        let sh = &p.slides[0].shapes;
        let round = named(sh, "Rounded Rectangle 1");
        match &round.geom {
            Geom::Preset { name, adj } => {
                assert_eq!(name, "roundRect", "{when}");
                assert_eq!(adj.first().copied(), Some(30000.0), "{when}: adj {adj:?}");
            }
            g => panic!("{when}: {g:?}"),
        }
        let line = round.line.as_ref().expect("outline");
        assert_eq!(line.width, Some(2.0), "{when}");
        assert!(line.dash.is_some(), "{when}: dashed");
        assert!(matches!(named(sh, "Oval 2").fill, Some(Fill::Gradient(_))), "{when}");
        let arrow = named(sh, "Right Arrow 3");
        let x = arrow.xfrm.expect("xfrm");
        assert!(close(x.rot, 45.0) && x.flip_h, "{when}: {x:?}");
        assert!(arrow.style.is_some(), "{when}: theme style refs kept");
        let cxn = named(sh, "Straight Arrow Connector 4");
        assert!(matches!(cxn.kind, ShapeKind::Connector { start: Some(_), end: Some(_) }), "{when}: {:?}", cxn.kind);
        let l = cxn.line.as_ref().expect("connector line");
        assert!(l.head.is_some() && l.tail.is_some(), "{when}: arrowheads");
        let group = named(sh, "Group 5");
        match &group.kind {
            ShapeKind::Group { children, child } => {
                assert_eq!(children.len(), 2, "{when}");
                let gx = group.xfrm.expect("group xfrm");
                assert!(close(gx.w, child.w * 2.0), "{when}: children scaled 2x ({gx:?} over {child:?})");
            }
            k => panic!("{when}: {k:?}"),
        }
        assert!(matches!(named(sh, "Freeform 8").geom, Geom::Custom { .. }), "{when}");
        assert!(!sh.iter().any(|s| matches!(s.kind, ShapeKind::Opaque { .. })), "{when}: nothing left unread");
    });
}

#[test]
fn tables_with_merged_cells_fills_and_borders() {
    let deck = Deck { name: "table", layouts: &[2], size: WIDE, sections: &[] };
    check_deck(&deck, |p, when| {
        let ShapeKind::Table(t) = &named(&p.slides[0].shapes, "Table 1").kind else { panic!("{when}: not a table") };
        assert_eq!((t.cols.len(), t.rows.len()), (3, 3), "{when}");
        assert!(t.first_row && t.band_row, "{when}");
        let c = |r: usize, k: usize| &t.rows[r].cells[k];
        assert_eq!(c(0, 0).grid_span, 2, "{when}");
        assert!(c(0, 1).h_merge, "{when}");
        assert_eq!(c(1, 0).row_span, 2, "{when}");
        assert!(c(2, 0).v_merge, "{when}");
        assert_eq!(c(0, 0).text.text(), "Región", "{when}");
        assert_eq!(c(2, 1).text.text(), "Gandía", "{when}");
        assert!(c(0, 0).fill.is_some() && c(1, 2).fill.is_some(), "{when}: cell fills");
        assert!(c(1, 2).borders.iter().any(Option::is_some), "{when}: bottom border");
    });
}

#[test]
fn pictures_with_crop_alt_text_and_a_shared_image() {
    let deck = Deck { name: "picture", layouts: &[2], size: WIDE, sections: &[] };
    check_deck(&deck, |p, when| {
        let sh = &p.slides[0].shapes;
        let a = named(sh, "Picture 1");
        assert_eq!(a.descr, "Orange square, test image", "{when}");
        let ShapeKind::Picture { fill } = &a.kind else { panic!("{when}: {:?}", a.kind) };
        assert!(close(fill.crop[0], 0.1) && close(fill.crop[2], 0.1), "{when}: crop {:?}", fill.crop);
        let ShapeKind::Picture { fill: other } = &named(sh, "Picture 2").kind else { panic!("{when}: second picture") };
        assert_eq!(fill.media, other.media, "{when}: one image, two pictures");
        assert!(p.media(fill.media).is_some_and(|m| m.data.starts_with(b"\x89PNG")), "{when}");
        assert!(matches!(&named(sh, "Picture 2").geom, Geom::Preset { name, .. } if name == "ellipse"), "{when}");
    });
}

#[test]
fn a_video_with_poster_trim_bookmark_and_playback_options() {
    let deck = Deck { name: "media", layouts: &[2], size: WIDE, sections: &[] };
    check_deck(&deck, |p, when| {
        let s = &p.slides[0];
        let ShapeKind::Media(m) = &named(&s.shapes, "clip").kind else { panic!("{when}: not media") };
        assert!(m.video && m.poster.is_some(), "{when}");
        assert_eq!(m.trim_start_ms, 500, "{when}");
        assert_eq!(m.bookmarks, vec![("Start".to_string(), 750)], "{when}");
        assert!(close(m.volume, 0.6), "{when}: volume {}", m.volume);
        assert_eq!((m.autoplay, m.loop_play, m.hide_while_not_playing), (true, true, true), "{when}");
        assert!(!m.full_screen && !m.rewind && !m.play_across_slides, "{when}");
        assert!(s.animations.is_empty(), "{when}: Start Automatically isn't a pane entry: {:?}", s.animations);
        assert!(p.media(m.media).is_some_and(|i| i.data.len() > 1000), "{when}: the clip's bytes");
    });
}

#[test]
fn click_sequence_effects_and_a_timed_transition() {
    let deck = Deck { name: "animation", layouts: &[2], size: WIDE, sections: &[] };
    check_deck(&deck, |p, when| {
        let s = &p.slides[0];
        let id = |n: &str| named(&s.shapes, n).id;
        let a = &s.animations;
        assert_eq!(a.len(), 4, "{when}: {a:?}");
        let got: Vec<_> = a.iter().map(|x| (x.shape, x.class, x.start)).collect();
        assert_eq!(
            got,
            [
                (id("Box"), AnimClass::Entrance, AnimStart::OnClick),
                (id("Badge"), AnimClass::Entrance, AnimStart::AfterPrevious),
                (id("Box"), AnimClass::Emphasis, AnimStart::WithPrevious),
                (id("Box"), AnimClass::Exit, AnimStart::OnClick),
            ],
            "{when}"
        );
        assert_eq!(a[0].effect, "fade", "{when}");
        assert_eq!(a[1].duration_ms, 500, "{when}");
        assert_eq!(a[2].duration_ms, 2000, "{when}");
        let t = s.transition.as_ref().expect("transition");
        assert_eq!(t.kind, "fade", "{when}");
        assert_eq!(t.advance_after_ms, Some(3000), "{when}");
        assert!(t.advance_on_click, "{when}");
    });
}

#[test]
fn hidden_slides_sections_notes_and_a_slide_jump() {
    let deck = Deck { name: "structure", layouts: &[1, 1, 1], size: (9_144_000, 6_858_000), sections: &[("Intro", &[1]), ("Body", &[2, 3])] };
    check_deck(&deck, |p, when| {
        assert_eq!(p.slides.len(), 3, "{when}");
        assert!(close(p.slide_size.width, 720.0) && close(p.slide_size.height, 540.0), "{when}: 4:3 {:?}", p.slide_size);
        assert_eq!(p.slides.iter().map(|s| s.hidden).collect::<Vec<_>>(), [false, true, false], "{when}");
        let names: Vec<(&str, usize)> = p.sections.iter().map(|s| (s.name.as_str(), s.slides.len())).collect();
        assert_eq!(names, [("Intro", 1), ("Body", 2)], "{when}");
        assert_eq!(p.sections[0].slides, vec![p.slides[0].id], "{when}");
        let notes = p.slides[0].notes.text();
        assert!(notes.contains("Нотатки доповідача") && notes.contains("Segunda línea."), "{when}: {notes:?}");
        assert!(p.slides[1].notes.text().is_empty(), "{when}");
        let button = named(&p.slides[2].shapes, "Back button");
        let jump = button.click.as_ref().map(|h| &h.action);
        assert!(matches!(jump, Some(Action::Slide { slide }) if *slide == p.slides[0].id), "{when}: {jump:?}");
    });
}
