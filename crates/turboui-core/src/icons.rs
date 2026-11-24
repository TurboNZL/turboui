//! Simple built-in icons made of filled polygons, so every backend draws the same thing.
//! These are placeholders for a proper icon set; callers only use the names.
use crate::node::{shape, Face, Node};
use crate::style::Color;

type P = Vec<(f32, f32)>;

fn rect(x0: f32, y0: f32, x1: f32, y1: f32) -> P {
    vec![(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
}
fn ngon(n: usize, r: f32) -> P {
    (0..n).map(|i| {
        let a = i as f32 / n as f32 * std::f32::consts::TAU;
        (0.5 + r * a.cos(), 0.5 + r * a.sin())
    }).collect()
}
fn gear() -> P {
    (0..16).map(|i| {
        let a = i as f32 / 16.0 * std::f32::consts::TAU;
        let r = if (i / 2) % 2 == 0 { 0.46 } else { 0.34 };
        (0.5 + r * a.cos(), 0.5 + r * a.sin())
    }).collect()
}

/// An icon of `size` pixels in one colour. Unknown names stop loudly.
pub fn icon(name: &str, size: f32, color: Color) -> Node {
    let f = |points: P| Face { points, color };
    let faded = |points: P, a: f32| Face { points, color: color.faded(a) };
    let faces = match name {
        "plus" => vec![f(rect(0.1, 0.42, 0.9, 0.58)), f(rect(0.42, 0.1, 0.58, 0.9))],
        "play" => vec![f(vec![(0.22, 0.1), (0.9, 0.5), (0.22, 0.9)])],
        "stop" => vec![f(rect(0.15, 0.15, 0.85, 0.85))],
        "kill" => vec![
            f(vec![(0.2, 0.1), (0.9, 0.8), (0.8, 0.9), (0.1, 0.2)]),
            f(vec![(0.8, 0.1), (0.9, 0.2), (0.2, 0.9), (0.1, 0.8)]),
        ],
        "edit" => vec![f(vec![(0.68, 0.08), (0.92, 0.32), (0.36, 0.88), (0.1, 0.92), (0.14, 0.66)])],
        "folder" => vec![f(rect(0.08, 0.15, 0.42, 0.32)), f(rect(0.08, 0.28, 0.92, 0.86))],
        "export" => vec![f(rect(0.44, 0.32, 0.56, 0.74)), f(vec![(0.24, 0.46), (0.5, 0.1), (0.76, 0.46)]), f(rect(0.14, 0.8, 0.86, 0.9))],
        "copy" => vec![faded(rect(0.3, 0.1, 0.88, 0.7), 0.55), f(rect(0.12, 0.3, 0.7, 0.9))],
        "delete" => vec![f(rect(0.38, 0.06, 0.62, 0.15)), f(rect(0.12, 0.17, 0.88, 0.28)), f(vec![(0.2, 0.33), (0.8, 0.33), (0.74, 0.93), (0.26, 0.93)])],
        "settings" => vec![f(gear())],
        "help" => vec![f(ngon(20, 0.45))],
        "dot" => vec![f(ngon(16, 0.5))],
        "tag" => vec![f(vec![(0.08, 0.12), (0.55, 0.12), (0.92, 0.5), (0.55, 0.88), (0.08, 0.88)])],
        other => panic!("no built-in icon called `{other}`"),
    };
    shape(size, size, faces)
}

/// The TurboMC orange cube: three faces, lit from above.
pub fn cube(size: f32, base: Color) -> Node {
    shape(size, size, vec![
        Face { points: vec![(0.5, 0.04), (0.94, 0.27), (0.5, 0.5), (0.06, 0.27)], color: base.lighten(0.28) },
        Face { points: vec![(0.06, 0.27), (0.5, 0.5), (0.5, 0.96), (0.06, 0.73)], color: base },
        Face { points: vec![(0.94, 0.27), (0.94, 0.73), (0.5, 0.96), (0.5, 0.5)], color: base.darken(0.28) },
    ])
}
