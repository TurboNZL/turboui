//! Reference renderer: draws a [`Scene`] to pixels in software, with no GPU and no window.
//!
//! It exists so that screens can be checked (and compared against saved "golden" images) on any machine,
//! and it is also the effects engine: backends that cannot do Liquid Glass natively ask it for the glass.

pub mod glass;

use tiny_skia::{Color as SkColor, FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};
use turboui_core::{paint_radials, Color, DrawItem, Measurer, Scene};

fn sk(c: Color) -> SkColor {
    SkColor::from_rgba8(c.r, c.g, c.b, c.a)
}

fn rounded(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<tiny_skia::Path> {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    let mut pb = PathBuilder::new();
    if r <= 0.0 {
        pb.push_rect(tiny_skia::Rect::from_xywh(x, y, w, h)?);
    } else {
        // Corners as cubic Béziers (kappa = 0.5523).
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

/// Draws `scene` over `background` at 1x and returns the pixels.
pub fn render(scene: &Scene, background: Color, m: &Measurer) -> Pixmap {
    render_scaled(scene, background, m, 1.0, scene.items.len())
}

/// Draws the first `upto` items of `scene` at `scale` device pixels per logical pixel.
pub fn render_scaled(scene: &Scene, background: Color, m: &Measurer, scale: f32, upto: usize) -> Pixmap {
    render_impl(scene, background, m, scale, upto, 0.0, 0.0, (scene.width * scale).round() as u32, (scene.height * scale).round() as u32)
}

/// Logical bounding box `(x, y, w, h)` of an item (shadows excluded).
pub fn bbox(it: &DrawItem) -> (f32, f32, f32, f32) {
    match it {
        DrawItem::Rect { x, y, w, h, .. } | DrawItem::Radials { x, y, w, h, .. } | DrawItem::Glass { x, y, w, h, .. } | DrawItem::Text { x, y, w, h, .. } => (*x, *y, *w, *h),
        DrawItem::Poly { points, .. } => {
            let (x0, y0) = points.iter().fold((f32::MAX, f32::MAX), |(a, b), (x, y)| (a.min(*x), b.min(*y)));
            let (x1, y1) = points.iter().fold((f32::MIN, f32::MIN), |(a, b), (x, y)| (a.max(*x), b.max(*y)));
            (x0, y0, x1 - x0, y1 - y0)
        }
    }
}

/// Draws the part of the scene inside the logical box starting at `(rx, ry)`, as a `pw` x `ph` device-pixel picture.
fn render_impl(scene: &Scene, background: Color, m: &Measurer, s: f32, upto: usize, rx: f32, ry: f32, pw: u32, ph: u32) -> Pixmap {
    let mut pm = Pixmap::new(pw.max(1), ph.max(1)).expect("non-empty scene");
    pm.fill(sk(background));
    let (rw, rh) = (pw as f32 / s, ph as f32 / s);
    for item in scene.items.iter().take(upto) {
        let (bx, by, bw, bh) = bbox(item);
        // shadows reach a little outside a glass box
        let reach = if let DrawItem::Glass { glass, .. } = item { glass.shadow_blur * 2.0 + glass.shadow_dy.abs() } else { 1.0 };
        if bx - rx > rw + reach || by - ry > rh + reach || bx - rx + bw < -reach || by - ry + bh < -reach {
            continue;
        }
        match item {
            DrawItem::Rect { x, y, w, h, bg, radius, border } => {
                let (x, y, w, h, radius) = ((x - rx) * s, (y - ry) * s, w * s, h * s, radius * s);
                if let Some(bg) = bg {
                    if let Some(p) = rounded(x, y, w, h, radius) {
                        let mut paint = Paint::default();
                        paint.set_color(sk(*bg));
                        paint.anti_alias = true;
                        pm.fill_path(&p, &paint, FillRule::Winding, Transform::identity(), None);
                    }
                }
                if let Some((bw, bc)) = border {
                    // The border sits inside the box, like CSS border-box.
                    let bw = bw * s;
                    let half = bw / 2.0;
                    if let Some(p) = rounded(x + half, y + half, w - bw, h - bw, (radius - half).max(0.0)) {
                        let mut paint = Paint::default();
                        paint.set_color(sk(*bc));
                        paint.anti_alias = true;
                        let stroke = Stroke { width: bw, ..Default::default() };
                        pm.stroke_path(&p, &paint, &stroke, Transform::identity(), None);
                    }
                }
            }
            DrawItem::Poly { points, color } => {
                let mut pb = PathBuilder::new();
                for (i, (px, py)) in points.iter().enumerate() {
                    let (px, py) = ((px - rx) * s, (py - ry) * s);
                    if i == 0 {
                        pb.move_to(px, py)
                    } else {
                        pb.line_to(px, py)
                    }
                }
                pb.close();
                if let Some(p) = pb.finish() {
                    let mut paint = Paint::default();
                    paint.set_color(sk(*color));
                    paint.anti_alias = true;
                    pm.fill_path(&p, &paint, FillRule::Winding, Transform::identity(), None);
                }
            }
            DrawItem::Radials { x, y, w, h, list } => {
                // only the part of the box that falls inside this picture
                let (wd, hd) = ((w * s).round() as i64, (h * s).round() as i64);
                let (ox, oy) = (((x - rx) * s).round() as i64, ((y - ry) * s).round() as i64);
                let (sx0, sy0) = ((-ox).max(0), (-oy).max(0));
                let (sx1, sy1) = (wd.min(pw as i64 - ox), hd.min(ph as i64 - oy));
                if sx1 > sx0 && sy1 > sy0 {
                    let (cw, ch) = ((sx1 - sx0) as u32, (sy1 - sy0) as u32);
                    let rgba = turboui_core::paint_radials_rect(wd as u32, hd as u32, list, sx0 as u32, sy0 as u32, cw, ch);
                    glass::composite(&mut pm, (ox + sx0) as i32, (oy + sy0) as i32, cw, ch, &rgba);
                }
            }
            DrawItem::Glass { x, y, w, h, radius, glass } => {
                let (x, y) = (x - rx, y - ry);
                let patch = glass::patch(&pm, x, y, *w, *h, *radius, glass, s);
                glass::shadow(&mut pm, x, y, *w, *h, *radius, glass, s);
                glass::composite(&mut pm, patch.x, patch.y, patch.w, patch.h, &patch.rgba);
            }
            DrawItem::Text { x, y, text, size, color, font, baseline, .. } => {
                let f = m.font(*font);
                let mut pen = (x - rx) * s;
                for ch in text.chars() {
                    let (met, cov) = f.rasterize(ch, size * s);
                    let gx = (pen + met.xmin as f32).round() as i32;
                    let gy = ((y - ry) * s + baseline * s - met.height as f32 - met.ymin as f32).round() as i32;
                    blend_glyph(&mut pm, &cov, met.width, met.height, gx, gy, *color);
                    pen += met.advance_width;
                }
            }
        }
    }
    pm
}

/// The finished glass for item number `index` of `scene` (which must be a `DrawItem::Glass`), drawn over
/// everything that comes before it. Used by backends that cannot blur or bend what is behind a surface.
/// Only the part of the scene under the glass is drawn, so this is much cheaper than a full render.
pub fn glass_patch(scene: &Scene, index: usize, background: Color, m: &Measurer, scale: f32) -> glass::Patch {
    let DrawItem::Glass { x, y, w, h, radius, glass } = &scene.items[index] else { panic!("item {index} is not glass") };
    let (rx, ry) = (x.floor(), y.floor());
    let (pw, ph) = (((x + w).ceil() - rx) * scale, ((y + h).ceil() - ry) * scale);
    let backdrop = render_impl(scene, background, m, scale, index, rx, ry, pw.round() as u32, ph.round() as u32);
    glass::patch(&backdrop, x - rx, y - ry, *w, *h, *radius, glass, scale)
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
