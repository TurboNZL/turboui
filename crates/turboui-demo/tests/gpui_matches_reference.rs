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
    let scene = layout(&screen::server_manager(&theme, &screen::State::sample()), 1000.0, 640.0, &m);
    let reference = turboui_raster::render(&scene, theme.color("window"), &m);
    let t = theme.clone();
    let shot = turboui_gpui::screenshot(screen::State::sample(), move |st| screen::server_manager(&t, st), 1000.0, 640.0);
    let d = mean_diff(shot, &reference);
    println!("mean difference between GPUI and the reference renderer: {d:.3} / 255");
    assert!(d < 4.0, "GPUI differs from the reference by {d:.2}/255 on average");
}

/// Clicking a server tile in the real GPUI window must select it, exactly as the app state says.
#[test]
fn clicking_a_tile_selects_it() {
    let theme = Theme::from_toml(THEME).unwrap();
    let m = Measurer::new();
    let mut after = screen::State::sample();
    after.click("srv:2");
    assert_eq!(after.selected, Some(2));
    let want = turboui_raster::render(&layout(&screen::server_manager(&theme, &after), 1000.0, 640.0, &m), theme.color("window"), &m);
    let unchanged = turboui_raster::render(&layout(&screen::server_manager(&theme, &screen::State::sample()), 1000.0, 640.0, &m), theme.color("window"), &m);
    let t = theme.clone();
    let shot = turboui_gpui::screenshot_after_clicks(
        screen::State::sample(),
        move |st| screen::server_manager(&t, st),
        |st, id| st.click(id),
        &["srv:2"],
        1000.0,
        640.0,
    );
    let (d_right, d_wrong) = (mean_diff(shot.clone(), &want), mean_diff(shot, &unchanged));
    println!("after click: {d_right:.3} from the expected picture, {d_wrong:.3} from the unclicked one");
    assert!(d_right < 4.0, "the click did not produce the expected picture ({d_right:.2})");
    assert!(d_wrong > d_right * 2.0, "the click had no visible effect");
}

#[test]
fn server_window_agrees_too() {
    let theme = Theme::from_toml(THEME).unwrap();
    let m = Measurer::new();
    let reference = turboui_raster::render(&layout(&screen::server_window(&theme), 900.0, 560.0, &m), theme.color("window"), &m);
    let t = theme.clone();
    let shot = turboui_gpui::screenshot((), move |_| screen::server_window(&t), 900.0, 560.0);
    let d = mean_diff(shot, &reference);
    println!("server window: {d:.3} / 255");
    assert!(d < 4.0, "GPUI differs from the reference by {d:.2}/255 on average");
}
