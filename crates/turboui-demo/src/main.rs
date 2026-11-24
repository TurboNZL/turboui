//! `turboui-demo --png out.png [WIDTH HEIGHT]` renders the sample screen in software.
//! Built with `--features gpui`, running it without arguments opens a real GPUI window.

use turboui_core::{layout, Measurer, Theme};
use turboui_demo::{screen, THEME};



fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let theme = Theme::from_toml(THEME).expect("bundled theme");
    if args.first().map(String::as_str) == Some("--png") {
        let out = args.get(1).expect("--png needs an output path");
        let w: f32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1000.0);
        let h: f32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(640.0);
        let m = Measurer::new();
        let scene = layout(&screen::server_manager(&theme), w, h, &m);
        let pm = turboui_raster::render(&scene, theme.color("bg"), &m);
        std::fs::write(out, turboui_raster::png_bytes(&pm)).expect("write png");
        println!("wrote {out} ({} items)", scene.items.len());
        return;
    }
    #[cfg(feature = "headless")]
    if args.first().map(String::as_str) == Some("--gpui-png") {
        let out = args.get(1).expect("--gpui-png needs an output path");
        let t = theme.clone();
        let (w, h, px) = turboui_gpui::screenshot(move || screen::server_manager(&t), 1000.0, 640.0);
        std::fs::write(out, turboui_raster::png_from_rgba(w, h, &px)).expect("write png");
        println!("wrote {out} ({w}x{h}) from the real GPUI renderer");
        return;
    }
    #[cfg(feature = "gpui")]
    {
        let t = theme.clone();
        turboui_gpui::run("TurboUI demo", (1000.0, 640.0), move || screen::server_manager(&t));
    }
    #[cfg(not(feature = "gpui"))]
    eprintln!("usage: turboui-demo --png out.png [W H]   (build with --features gpui for a window)");
}
