//! A click must not make the glass slow: the picture for a glass surface is only about the part of the screen under it.
use std::time::Instant;
use turboui_core::{layout, DrawItem, Measurer};
use turboui_demo::{screen, theme_from_arg};

#[test]
fn glass_patches_are_cheap_at_2x() {
    let theme = theme_from_arg("macos-dark/liquid");
    let m = Measurer::new();
    let scene = layout(&screen::server_manager(&theme, &screen::State::sample()), 1000.0, 640.0, &m);
    for (i, it) in scene.items.iter().enumerate() {
        if matches!(it, DrawItem::Glass { .. }) {
            let t = Instant::now();
            let p = turboui_raster::glass_patch(&scene, i, theme.color("bg"), &m, 2.0);
            let ms = t.elapsed().as_millis();
            println!("glass item {i}: {}x{} px in {ms} ms", p.w, p.h);
            assert!(ms < 1500, "glass item {i} took {ms} ms");
        }
    }
}
