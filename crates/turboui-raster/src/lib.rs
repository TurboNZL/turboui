//! Reference renderer: draws a [`Scene`] to a PNG in software, with no GPU and no window.
//!
//! It exists so that screens can be checked (and compared against saved "golden" images) on any
//! machine. The real backends should look the same as this, give or take anti-aliasing.

use tiny_skia::{Color as SkColor, FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};
use turboui_core::{Color, DrawItem, Measurer, Scene};

fn sk(c: Color) -> SkColor {
    SkColor::from_rgba8(c.r, c.g, c.b, c.a)
}

fn rounded(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<tiny_skia::Path> {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    let mut pb = PathBuilder::new();
    if r <= 0.0 {
        pb.push_rect(tiny_skia::Rect::from_xywh(x, y, w, h)?);
    } else {
        // Corners as quadratic-ish arcs using cubic Béziers (kappa = 0.5523).
        let k = r * 0.5523;
        pb.move_to(x + r, y);
        pb.line_to(x + w - r, y);
        pb.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
        pb.line_to(x + w, y + h - r);
        pb.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
        pb.line_to(x + r, y + h);
        pb.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
        pb.line_to(x, y + r);
        pb.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
        pb.close();
    }
    pb.finish()
}

/// Draws `scene` over `background` and returns the pixels.
pub fn render(scene: &Scene, background: Color, m: &Measurer) -> Pixmap {
    let mut pm = Pixmap::new(scene.width.round() as u32, scene.height.round() as u32).expect("non-empty scene");
    pm.fill(sk(background));
    for item in &scene.items {
        match item {
            DrawItem::Rect { x, y, w, h, bg, radius, border } => {
                if let Some(bg) = bg {
                    if let Some(p) = rounded(*x, *y, *w, *h, *radius) {
                        let mut paint = Paint::default();
                        paint.set_color(sk(*bg));
                        paint.anti_alias = true;
                        pm.fill_path(&p, &paint, FillRule::Winding, Transform::identity(), None);
                    }
                }
                if let Some((bw, bc)) = border {
                    // The border sits inside the box, like CSS border-box.
                    let half = bw / 2.0;
                    if let Some(p) = rounded(x + half, y + half, w - bw, h - bw, (radius - half).max(0.0)) {
                        let mut paint = Paint::default();
                        paint.set_color(sk(*bc));
                        paint.anti_alias = true;
                        let stroke = Stroke { width: *bw, ..Default::default() };
                        pm.stroke_path(&p, &paint, &stroke, Transform::identity(), None);
                    }
                }
            }
            DrawItem::Poly { points, color } => {
                let mut pb = PathBuilder::new();
                for (i, (px, py)) in points.iter().enumerate() {
                    if i == 0 { pb.move_to(*px, *py) } else { pb.line_to(*px, *py) }
                }
                pb.close();
                if let Some(p) = pb.finish() {
                    let mut paint = Paint::default();
                    paint.set_color(sk(*color));
                    paint.anti_alias = true;
                    pm.fill_path(&p, &paint, FillRule::Winding, Transform::identity(), None);
                }
            }
            DrawItem::Text { x, y, text, size, color, font, baseline, .. } => {
                let f = m.font(*font);
                let mut pen = *x;
                for ch in text.chars() {
                    let (met, cov) = f.rasterize(ch, *size);
                    let gx = (pen + met.xmin as f32).round() as i32;
                    let gy = (y + baseline - met.height as f32 - met.ymin as f32).round() as i32;
                    blend_glyph(&mut pm, &cov, met.width, met.height, gx, gy, *color);
                    pen += met.advance_width;
                }
            }
        }
    }
    pm
}

fn blend_glyph(pm: &mut Pixmap, cov: &[u8], gw: usize, gh: usize, gx: i32, gy: i32, c: Color) {
    let (pw, ph) = (pm.width() as i32, pm.height() as i32);
    let data = pm.data_mut();
    for row in 0..gh as i32 {
        for col in 0..gw as i32 {
            let (px, py) = (gx + col, gy + row);
            if px < 0 || py < 0 || px >= pw || py >= ph {
                continue;
            }
            let a = cov[(row as usize) * gw + col as usize] as u32 * c.a as u32 / 255;
            if a == 0 {
                continue;
            }
            let i = ((py * pw + px) * 4) as usize;
            // The pixmap is premultiplied RGBA; blend the source colour over it.
            for (k, s) in [c.r, c.g, c.b].into_iter().enumerate() {
                let dst = data[i + k] as u32;
                data[i + k] = ((s as u32 * a + dst * (255 - a)) / 255) as u8;
            }
            let da = data[i + 3] as u32;
            data[i + 3] = (a + da * (255 - a) / 255) as u8;
        }
    }
}

/// PNG bytes for a pixmap.
pub fn png_bytes(pm: &Pixmap) -> Vec<u8> {
    pm.encode_png().expect("png encoding")
}

/// PNG bytes for raw RGBA pixels (used for pictures taken by other backends).
pub fn png_from_rgba(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, width, height);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut w = enc.write_header().expect("png header");
        w.write_image_data(rgba).expect("png data");
    }
    out
}
