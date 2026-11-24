//! Draws the sample screen with the real GPUI renderer (offscreen) and with the software reference
//! renderer, and checks they agree closely. Needs a Vulkan driver (lavapipe is fine):
//!   VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json cargo test -p turboui-demo --features headless --test gpui_matches_reference
#![cfg(feature = "headless")]
use turboui_core::{layout, Measurer, Theme};
use turboui_demo::{screen, THEME};

/// Average per-channel difference (0 to 255) between the GPUI picture, shrunk to the reference size, and the reference.
fn mean_diff(gpui: (u32, u32, Vec<u8>), reference: &tiny_skia::Pixmap) -> f64 {
    let (gw, gh, g) = gpui;
    let (w, h) = (reference.width(), reference.height());
    assert_eq!((gw % w, gh % h), (0, 0), "GPUI picture is not a whole multiple of the reference size");
    let (sx, sy) = (gw / w, gh / h);
    let r = reference.data();
    let mut total = 0u64;
    for y in 0..h {
        for x in 0..w {
            for c in 0..3 {
                let mut sum = 0u32;
                for dy in 0..sy {
                    for dx in 0..sx {
                        sum += g[(((y * sy + dy) * gw + x * sx + dx) * 4 + c) as usize] as u32;
                    }
                }
                let avg = sum / (sx * sy);
                total += (avg as i32 - r[((y * w + x) * 4 + c) as usize] as i32).unsigned_abs() as u64;
            }
        }
    }
    total as f64 / (w as u64 * h as u64 * 3) as f64
}

#[test]
fn gpui_and_reference_agree() {
    let theme = Theme::from_toml(THEME).unwrap();
    let m = Measurer::new();
    let scene = layout(&screen::server_manager(&theme), 1000.0, 640.0, &m);
    let reference = turboui_raster::render(&scene, theme.color("bg"), &m);
    let t = theme.clone();
    let shot = turboui_gpui::screenshot(move || screen::server_manager(&t), 1000.0, 640.0);
    let d = mean_diff(shot, &reference);
    println!("mean difference between GPUI and the reference renderer: {d:.3} / 255");
    assert!(d < 4.0, "GPUI differs from the reference by {d:.2}/255 on average");
}
