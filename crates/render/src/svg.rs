//! SVG pictures (`image/svg+xml`): parsed by `usvg` and drawn by `resvg` into a bitmap of the size
//! the slide needs, so they go through the same crop, tile and recolour path as any other picture.
//!
//! PowerPoint stores an SVG picture as a PNG fallback with the SVG beside it (`asvg:svgBlip`);
//! DeckCraft draws the SVG when it reads and the fallback otherwise.
//!
//! The file is untrusted: it may reach nothing outside itself (no files, no nested SVG documents;
//! PNG, JPEG, GIF and WebP images embedded in it are drawn, within a size cap read from their
//! header), compressed SVGZ is refused (it can inflate without bound), and the input and the bitmap
//! are capped. Not drawn yet: text in an SVG (no fonts are given to it).

use resvg::{tiny_skia, usvg};

/// Largest SVG file read.
const MAX_BYTES: usize = 16 << 20;
/// Largest side drawn.
const MAX_SIDE: u32 = 8192;

/// Does `bytes` look like an SVG document (an `<svg` element near the start, after any XML
/// declaration, comments or doctype)?
pub fn is_svg(bytes: &[u8]) -> bool {
    let head = bytes.get(..bytes.len().min(4096)).unwrap_or(bytes);
    let text = String::from_utf8_lossy(head);
    let t = text.trim_start_matches('\u{feff}').trim_start();
    t.starts_with('<') && text.contains("<svg")
}

/// Largest raster image embedded in an SVG, in RGBA bytes once decoded.
const MAX_EMBEDDED_RGBA: u64 = 64 << 20;

/// A raster image embedded in an SVG (a `data:` URL), when its header names a format `resvg`
/// draws and a size within [`MAX_EMBEDDED_RGBA`]. Never a nested SVG: that could recurse.
fn embedded(data: std::sync::Arc<Vec<u8>>) -> Option<usvg::ImageKind> {
    let reader = image::ImageReader::new(std::io::Cursor::new(data.as_slice())).with_guessed_format().ok()?;
    let format = reader.format()?;
    let (w, h) = reader.into_dimensions().ok()?;
    if w == 0 || h == 0 || u64::from(w) * u64::from(h) * 4 > MAX_EMBEDDED_RGBA {
        return None;
    }
    match format {
        image::ImageFormat::Png => Some(usvg::ImageKind::PNG(data)),
        image::ImageFormat::Jpeg => Some(usvg::ImageKind::JPEG(data)),
        image::ImageFormat::Gif => Some(usvg::ImageKind::GIF(data)),
        image::ImageFormat::WebP => Some(usvg::ImageKind::WEBP(data)),
        _ => None,
    }
}

fn options() -> usvg::Options<'static> {
    usvg::Options {
        resources_dir: None,
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: Box::new(|_, data, _| embedded(data)),
            // Never a file or URL: a picture in a deck must not read the computer it is opened on.
            resolve_string: Box::new(|_, _| None),
        },
        ..Default::default()
    }
}

fn tree(bytes: &[u8]) -> Option<usvg::Tree> {
    if bytes.len() > MAX_BYTES || bytes.starts_with(&[0x1f, 0x8b]) || !is_svg(bytes) {
        return None;
    }
    let text = std::str::from_utf8(bytes).ok()?;
    usvg::Tree::from_str(text, &options()).ok()
}

/// The picture's own size in CSS pixels (96 dpi), from its `width`/`height` or `viewBox`.
pub fn natural_size(bytes: &[u8]) -> Option<(f64, f64)> {
    let s = tree(bytes)?.size();
    let (w, h) = (s.width() as f64, s.height() as f64);
    (w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0).then_some((w, h))
}

/// Draw the SVG `bytes` into straight RGBA with its longer side `side` px (transparent where it
/// draws nothing). `None` when it doesn't parse or has no size.
pub fn rasterize(bytes: &[u8], side: u32) -> Option<image::RgbaImage> {
    let tree = tree(bytes)?;
    let size = tree.size();
    let (pw, ph) = (size.width() as f64, size.height() as f64);
    if !(pw.is_finite() && ph.is_finite() && pw > 0.0 && ph > 0.0) {
        return None;
    }
    let side = side.clamp(16, MAX_SIDE) as f64;
    let k = side / pw.max(ph);
    let w = (pw * k).round().clamp(1.0, MAX_SIDE as f64) as u32;
    let h = (ph * k).round().clamp(1.0, MAX_SIDE as f64) as u32;
    let mut pm = tiny_skia::Pixmap::new(w, h)?;
    let t = tiny_skia::Transform::from_scale((w as f64 / pw) as f32, (h as f64 / ph) as f32);
    resvg::render(&tree, t, &mut pm.as_mut());
    let mut out = image::RgbaImage::new(w, h);
    for (dst, src) in out.pixels_mut().zip(pm.pixels()) {
        let c = src.demultiply();
        *dst = image::Rgba([c.red(), c.green(), c.blue(), c.alpha()]);
    }
    Some(out)
}

/// A PNG of the SVG with its longer side `side` px: the fallback PowerPoint stores beside an
/// SVG picture for readers that can't draw SVG.
pub fn to_png(bytes: &[u8], side: u32) -> Option<Vec<u8>> {
    let img = rasterize(bytes, side)?;
    let mut out = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(img).write_to(&mut out, image::ImageFormat::Png).ok()?;
    Some(out.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base64(bytes: &[u8]) -> String {
        const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for c in bytes.chunks(3) {
            let n = (u32::from(c[0]) << 16) | (u32::from(*c.get(1).unwrap_or(&0)) << 8) | u32::from(*c.get(2).unwrap_or(&0));
            for k in 0..4 {
                out.push(if k <= c.len() { T[(n >> (18 - 6 * k) & 63) as usize] as char } else { '=' });
            }
        }
        out
    }

    const SQUARE: &[u8] =
        br##"<?xml version="1.0"?><svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><rect width="20" height="20" fill="#ff0000"/></svg>"##;

    #[test]
    fn an_svg_draws_at_the_asked_size_with_its_shapes_and_transparency() {
        let img = rasterize(SQUARE, 200).unwrap();
        assert_eq!(img.dimensions(), (200, 100));
        assert_eq!(img.get_pixel(50, 50).0, [255, 0, 0, 255], "the red square");
        assert_eq!(img.get_pixel(150, 50).0[3], 0, "transparent where nothing is drawn");
        assert_eq!(natural_size(SQUARE), Some((40.0, 20.0)));
        let png = to_png(SQUARE, 64).unwrap();
        assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));
    }

    #[test]
    fn hostile_svgs_are_refused_or_drawn_without_reaching_outside() {
        // Not SVG, broken XML, no size, compressed, too large.
        for bad in [&b"hello"[..], b"<svg", b"<svg xmlns='http://www.w3.org/2000/svg' width='0' height='0'/>", &[0x1f, 0x8b, 8, 0]] {
            assert!(rasterize(bad, 64).is_none(), "{:?}", String::from_utf8_lossy(bad));
        }
        let mut huge = b"<svg xmlns='http://www.w3.org/2000/svg' width='10' height='10'>".to_vec();
        huge.resize(MAX_BYTES + 1, b' ');
        huge.extend_from_slice(b"</svg>");
        assert!(rasterize(&huge, 64).is_none());
        // A real picture file on this computer and a nested SVG are not followed.
        let file = std::env::temp_dir().join(format!("deckcraft-svg-test-{}.png", std::process::id()));
        image::RgbaImage::from_pixel(10, 10, image::Rgba([0, 255, 0, 255])).save(&file).unwrap();
        let nested = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0naHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmcnIHdpZHRoPScxMCcgaGVpZ2h0PScxMCc+PHJlY3Qgd2lkdGg9JzEwJyBoZWlnaHQ9JzEwJy8+PC9zdmc+";
        for href in [file.to_string_lossy().to_string(), format!("file://{}", file.display()), nested.to_string()] {
            let doc = format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="10" height="10"><image width="10" height="10" xlink:href="{href}"/></svg>"#
            );
            let img = rasterize(doc.as_bytes(), 32).unwrap();
            assert!(img.pixels().all(|p| p.0[3] == 0), "{href} was drawn");
        }
        let _ = std::fs::remove_file(&file);
        // An embedded picture draws; one too large once decoded (here a grey PNG of 4200 × 4200,
        // 70 MB as RGBA, though small as a file) is skipped before it is decoded.
        let embed = |img: image::DynamicImage| {
            let mut png = std::io::Cursor::new(Vec::new());
            img.write_to(&mut png, image::ImageFormat::Png).unwrap();
            let href = format!("data:image/png;base64,{}", base64(&png.into_inner()));
            let doc =
                format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><image width="10" height="10" href="{href}"/></svg>"#);
            rasterize(doc.as_bytes(), 32).unwrap()
        };
        let green = image::RgbaImage::from_pixel(10, 10, image::Rgba([0, 255, 0, 255]));
        assert_eq!(embed(green.into()).get_pixel(16, 16).0, [0, 255, 0, 255], "an embedded PNG draws");
        let big = image::GrayImage::from_pixel(4200, 4200, image::Luma([128]));
        assert!(embed(big.into()).pixels().all(|p| p.0[3] == 0), "an embedded picture past the cap is skipped");
        // An absurd size is clamped, not allocated.
        let wide = b"<svg xmlns='http://www.w3.org/2000/svg' width='1e30' height='1'/>";
        assert!(rasterize(wide, u32::MAX).is_none_or(|i| i.width() <= MAX_SIDE && i.height() <= MAX_SIDE));
    }
}
