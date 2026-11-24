use crate::style::Radial;

/// Paints soft round patches into straight-alpha RGBA bytes of size `w` x `h`. All renderers use this, so the
/// ambient background is identical everywhere. Each patch fades linearly to nothing at 62% of its radius,
/// like the CSS `radial-gradient(color, transparent 62%)` it replaces.
pub fn paint_radials(w: u32, h: u32, list: &[Radial]) -> Vec<u8> {
    paint_radials_rect(w, h, list, 0, 0, w, h)
}

/// Like [`paint_radials`] for a `w` x `h` box, but only the pixels of the `rw` x `rh` rectangle at `(rx, ry)` (cheaper when
/// only a part is needed, e.g. what is behind a glass surface).
pub fn paint_radials_rect(w: u32, h: u32, list: &[Radial], rx: u32, ry: u32, rw: u32, rh: u32) -> Vec<u8> {
    let mut out = vec![0u8; (rw * rh * 4) as usize];
    for y in 0..rh {
        for x in 0..rw {
            let (px, py) = ((rx + x) as f32 + 0.5, (ry + y) as f32 + 0.5);
            let (mut r, mut g, mut b, mut a) = (0f32, 0f32, 0f32, 0f32); // premultiplied
            for p in list {
                let d = ((px - p.x * w as f32).powi(2) + (py - p.y * h as f32).powi(2)).sqrt();
                let t = (1.0 - d / (0.62 * p.radius * w as f32)).clamp(0.0, 1.0);
                let pa = p.color.a as f32 / 255.0 * t;
                if pa <= 0.0 {
                    continue;
                }
                r = p.color.r as f32 / 255.0 * pa + r * (1.0 - pa);
                g = p.color.g as f32 / 255.0 * pa + g * (1.0 - pa);
                b = p.color.b as f32 / 255.0 * pa + b * (1.0 - pa);
                a = pa + a * (1.0 - pa);
            }
            if a > 0.0 {
                let i = ((y * rw + x) * 4) as usize;
                out[i] = (r / a * 255.0).round().clamp(0.0, 255.0) as u8;
                out[i + 1] = (g / a * 255.0).round().clamp(0.0, 255.0) as u8;
                out[i + 2] = (b / a * 255.0).round().clamp(0.0, 255.0) as u8;
                out[i + 3] = (a * 255.0).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    out
}
