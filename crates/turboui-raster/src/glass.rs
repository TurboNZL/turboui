//! The Liquid Glass material, in software: blur what is behind, bend it at the rim (Snell's law through a
//! convex squircle bezel, as in turbothemes' `docs/LIQUID_GLASS.md`), saturate, tint, add a diagonal sheen and a
//! rim lit from the top left. Other backends that cannot do this natively ask for a [`Patch`] and paint it.

use tiny_skia::Pixmap;
use turboui_core::Glass;

/// A finished glass surface: straight-alpha RGBA (alpha 0 outside the rounded corners), at device pixels.
pub struct Patch {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
}

/// Distance from the point to the rounded rectangle's edge: negative inside. Also gives the outward normal.
fn sdf(px: f32, py: f32, hw: f32, hh: f32, r: f32) -> (f32, (f32, f32)) {
    let (qx, qy) = (px.abs() - (hw - r), py.abs() - (hh - r));
    let (sx, sy) = (if px < 0.0 { -1.0 } else { 1.0 }, if py < 0.0 { -1.0 } else { 1.0 });
    let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
    let d = outside + qx.max(qy).min(0.0) - r;
    let n = if qx > 0.0 && qy > 0.0 {
        (sx * qx / outside, sy * qy / outside)
    } else if qx > qy {
        (sx, 0.0)
    } else {
        (0.0, sy)
    };
    (d, n)
}

/// Box blur, three passes (close to a Gaussian), edges repeat. `buf` is `w*h*3` floats.
fn blur(buf: &mut [f32], w: usize, h: usize, sigma: f32) {
    if sigma < 0.5 {
        return;
    }
    let box_w = ((4.0 * sigma * sigma + 1.0).sqrt().round() as usize).max(1);
    let r = (box_w / 2) as isize;
    let mut tmp = vec![0f32; buf.len()];
    for _ in 0..3 {
        // horizontal
        for y in 0..h {
            for c in 0..3 {
                let at = |x: isize| buf[(y * w + x.clamp(0, w as isize - 1) as usize) * 3 + c];
                let mut sum: f32 = (-r..=r).map(at).sum();
                for x in 0..w {
                    tmp[(y * w + x) * 3 + c] = sum / (2 * r + 1) as f32;
                    sum += at(x as isize + r + 1) - at(x as isize - r);
                }
            }
        }
        // vertical
        for x in 0..w {
            for c in 0..3 {
                let at = |y: isize| tmp[(y.clamp(0, h as isize - 1) as usize * w + x) * 3 + c];
                let mut sum: f32 = (-r..=r).map(at).sum();
                for y in 0..h {
                    buf[(y * w + x) * 3 + c] = sum / (2 * r + 1) as f32;
                    sum += at(y as isize + r + 1) - at(y as isize - r);
                }
            }
        }
    }
}

fn smoothstep(a: f32, b: f32, v: f32) -> f32 {
    let t = ((v - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// How far, towards the inside, a vertical ray is bent at depth `d` from the rim (all in device pixels).
fn lens_shift(d: f32, bezel: f32) -> f32 {
    if d >= bezel {
        return 0.0;
    }
    let t = (d / bezel).clamp(0.002, 1.0);
    let one = 1.0 - t;
    let height = (1.0 - one.powi(4)).powf(0.25);
    let slope = one.powi(3) * (1.0 - one.powi(4)).powf(-0.75);
    let th1 = slope.atan();
    let th2 = (th1.sin() / 1.5).asin();
    let thickness = bezel * (0.6 + height);
    (thickness * (th1 - th2).tan()).min(bezel * 1.2)
}

/// Builds the glass for the box `(x, y, w, h)` (logical pixels) over `backdrop` (everything painted so far, at `scale`).
pub fn patch(backdrop: &Pixmap, x: f32, y: f32, w: f32, h: f32, radius: f32, g: &Glass, scale: f32) -> Patch {
    let (x0, y0) = ((x * scale).round() as i32, (y * scale).round() as i32);
    let (wd, hd) = (((w * scale).round() as u32).max(1), ((h * scale).round() as u32).max(1));
    let (wu, hu) = (wd as usize, hd as usize);
    let r = (radius * scale).min(wd.min(hd) as f32 / 2.0);
    let (hw, hh) = (wd as f32 / 2.0, hd as f32 / 2.0);

    // 1. what is behind
    let (bw, bh) = (backdrop.width() as i32, backdrop.height() as i32);
    let src = backdrop.data();
    let mut buf = vec![0f32; wu * hu * 3];
    for j in 0..hu {
        for i in 0..wu {
            let (sx, sy) = ((x0 + i as i32).clamp(0, bw - 1), (y0 + j as i32).clamp(0, bh - 1));
            let o = ((sy * bw + sx) * 4) as usize;
            for c in 0..3 {
                buf[(j * wu + i) * 3 + c] = src[o + c] as f32;
            }
        }
    }
    blur(&mut buf, wu, hu, g.blur * scale);

    // 2. bend at the rim
    let bezel = (0.22 * wd.min(hd) as f32).clamp(8.0 * scale, 28.0 * scale);
    let blurred = buf.clone();
    let mut out = vec![0u8; wu * hu * 4];
    let (sat, bright) = (g.saturate, g.brightness);
    let (fr, fg, fb, fa) = (g.fill.r as f32, g.fill.g as f32, g.fill.b as f32, g.fill.a as f32 / 255.0);
    let (dx, dy) = ((140f32.to_radians()).sin(), -(140f32.to_radians()).cos());
    let glen = (wd as f32 * dx.abs() + hd as f32 * dy.abs()).max(1.0);
    let (lx, ly) = (-std::f32::consts::FRAC_1_SQRT_2, -std::f32::consts::FRAC_1_SQRT_2);
    let ring = (1.1 * scale).max(1.0);
    for j in 0..hu {
        for i in 0..wu {
            let (px, py) = (i as f32 + 0.5 - hw, j as f32 + 0.5 - hh);
            let (dout, n) = sdf(px, py, hw, hh, r);
            let inside = -dout;
            // sample position after the lens
            let (mut sx, mut sy) = (i as f32, j as f32);
            if g.lens && inside < bezel {
                let shift = lens_shift(inside.max(0.0), bezel);
                sx -= n.0 * shift;
                sy -= n.1 * shift;
            }
            let (sx, sy) = (sx.clamp(0.0, wu as f32 - 1.0), sy.clamp(0.0, hu as f32 - 1.0));
            // bilinear read of the blurred picture
            let (ix, iy) = (sx.floor() as usize, sy.floor() as usize);
            let (fx, fy) = (sx - ix as f32, sy - iy as f32);
            let (ix1, iy1) = ((ix + 1).min(wu - 1), (iy + 1).min(hu - 1));
            let mut c3 = [0f32; 3];
            for c in 0..3 {
                let a = blurred[(iy * wu + ix) * 3 + c] * (1.0 - fx) + blurred[(iy * wu + ix1) * 3 + c] * fx;
                let b = blurred[(iy1 * wu + ix) * 3 + c] * (1.0 - fx) + blurred[(iy1 * wu + ix1) * 3 + c] * fx;
                c3[c] = a * (1.0 - fy) + b * fy;
            }
            // saturate, brighten
            let lum = 0.2126 * c3[0] + 0.7152 * c3[1] + 0.0722 * c3[2];
            for v in c3.iter_mut() {
                *v = ((lum + (*v - lum) * sat) * bright).clamp(0.0, 255.0);
            }
            // tint
            c3 = [c3[0] * (1.0 - fa) + fr * fa, c3[1] * (1.0 - fa) + fg * fa, c3[2] * (1.0 - fa) + fb * fa];
            // diagonal sheen
            let t = (((px * dx + py * dy) / glen) + 0.5).clamp(0.0, 1.0);
            let sheen_a = if t < 0.44 { g.sheen + (g.sheen * 0.107 - g.sheen) * (t / 0.44) } else { g.sheen * 0.107 + (g.sheen * 0.43 - g.sheen * 0.107) * ((t - 0.44) / 0.56) };
            for v in c3.iter_mut() {
                *v += (255.0 - *v) * sheen_a;
            }
            // rim lit from the top left (and a little from the opposite side)
            if inside < ring + 1.0 && inside > -1.0 {
                let facing = n.0 * lx + n.1 * ly;
                let wgt = (0.22 + facing.max(0.0) + 0.35 * (-facing).max(0.0)).min(1.0);
                let cover = 1.0 - smoothstep(ring - 0.5, ring + 0.5, inside.max(0.0));
                let a = g.rim.a as f32 / 255.0 * wgt * cover;
                let rim = [g.rim.r as f32, g.rim.g as f32, g.rim.b as f32];
                for c in 0..3 {
                    c3[c] = c3[c] * (1.0 - a) + rim[c] * a;
                }
            }
            // rounded-corner mask, anti-aliased
            let alpha = (0.5 + inside).clamp(0.0, 1.0);
            let o = (j * wu + i) * 4;
            out[o] = c3[0].round().clamp(0.0, 255.0) as u8;
            out[o + 1] = c3[1].round().clamp(0.0, 255.0) as u8;
            out[o + 2] = c3[2].round().clamp(0.0, 255.0) as u8;
            out[o + 3] = (alpha * 255.0).round() as u8;
        }
    }
    Patch { x: x0, y: y0, w: wd, h: hd, rgba: out }
}

/// Source-over of straight-alpha RGBA `src` (size `w` x `h`) onto `pm` at `(x, y)`.
pub fn composite(pm: &mut Pixmap, x: i32, y: i32, w: u32, h: u32, src: &[u8]) {
    let (pw, ph) = (pm.width() as i32, pm.height() as i32);
    let dst = pm.data_mut();
    for j in 0..h as i32 {
        for i in 0..w as i32 {
            let (px, py) = (x + i, y + j);
            if px < 0 || py < 0 || px >= pw || py >= ph {
                continue;
            }
            let s = ((j as u32 * w + i as u32) * 4) as usize;
            let a = src[s + 3] as u32;
            if a == 0 {
                continue;
            }
            let d = ((py * pw + px) * 4) as usize;
            for c in 0..3 {
                let premul = (src[s + c] as u32 * a + 127) / 255;
                dst[d + c] = (premul + dst[d + c] as u32 * (255 - a) / 255).min(255) as u8;
            }
            dst[d + 3] = (a + dst[d + 3] as u32 * (255 - a) / 255).min(255) as u8;
        }
    }
}

/// The soft shadow around a glass box (not drawn under the box itself).
pub fn shadow(pm: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, radius: f32, g: &Glass, scale: f32) {
    if g.shadow.a == 0 || g.shadow_blur <= 0.0 {
        return;
    }
    let sigma = g.shadow_blur * scale / 2.0;
    let m = (sigma * 3.0).ceil() as i32 + 2;
    let (wd, hd) = (((w * scale).round() as i32).max(1), ((h * scale).round() as i32).max(1));
    let (tw, th) = ((wd + 2 * m) as usize, (hd + 2 * m) as usize);
    let r = (radius * scale).min(wd.min(hd) as f32 / 2.0);
    let dy = (g.shadow_dy * scale).round() as i32;
    // a white-on-black mask of the shadow shape, offset down, then blurred
    let mut buf = vec![0f32; tw * th * 3];
    for j in 0..th {
        for i in 0..tw {
            let (px, py) = (i as f32 + 0.5 - (m + wd / 2) as f32, j as f32 + 0.5 - (m + hd / 2) as f32 - dy as f32);
            let (d, _) = sdf(px, py, wd as f32 / 2.0, hd as f32 / 2.0, r);
            let v = (0.5 - d).clamp(0.0, 1.0) * 255.0;
            for c in 0..3 {
                buf[(j * tw + i) * 3 + c] = v;
            }
        }
    }
    blur(&mut buf, tw, th, sigma);
    let (x0, y0) = ((x * scale).round() as i32 - m, (y * scale).round() as i32 - m);
    let mut rgba = vec![0u8; tw * th * 4];
    for j in 0..th {
        for i in 0..tw {
            // keep it out of the box itself
            let (px, py) = (i as f32 + 0.5 - (m + wd / 2) as f32, j as f32 + 0.5 - (m + hd / 2) as f32);
            let (d, _) = sdf(px, py, wd as f32 / 2.0, hd as f32 / 2.0, r);
            let hole = ((d + 0.5).clamp(0.0, 1.0)) as f32;
            let a = buf[(j * tw + i) * 3] / 255.0 * g.shadow.a as f32 * hole;
            let o = (j * tw + i) * 4;
            rgba[o] = g.shadow.r;
            rgba[o + 1] = g.shadow.g;
            rgba[o + 2] = g.shadow.b;
            rgba[o + 3] = a.round().clamp(0.0, 255.0) as u8;
        }
    }
    composite(pm, x0, y0, tw as u32, th as u32, &rgba);
}
