//! Golden-image test: the sample screen must render to exactly the saved picture.
//! After an intentional change, regenerate with `TURBOUI_BLESS=1 cargo test` and review the PNG.
use turboui_core::{layout, Measurer};
use turboui_demo::{screen, theme_from_arg};

fn render(which: &str, look: &str, w: f32, h: f32) -> Vec<u8> {
    let theme = theme_from_arg(look);
    let m = Measurer::new();
    let node = match which {
        "main" => screen::server_manager(&theme, &screen::State::sample()),
        "window" => screen::server_window(&theme),
        "controls" => screen::controls(&theme, 0.0, false),
        "controls-scrolled" => screen::controls(&theme, 120.0, false),
        "dialog" => screen::controls(&theme, 0.0, true),
        _ => unreachable!(),
    };
    let scene = layout(&node, w, h, &m);
    turboui_raster::png_bytes(&turboui_raster::render(&scene, theme.color("bg"), &m))
}

fn check(name: &str, which: &str, look: &str, w: f32, h: f32) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../goldens").join(name);
    let got = render(which, look, w, h);
    if std::env::var("TURBOUI_BLESS").is_ok() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &got).unwrap();
        return;
    }
    let want = std::fs::read(&path).unwrap_or_else(|_| panic!("missing golden {name}; run with TURBOUI_BLESS=1"));
    assert!(got == want, "{name} no longer matches its golden image; if intended, re-run with TURBOUI_BLESS=1");
}

#[test]
fn flat_nord() {
    check("main_nord_flat_1000x640.png", "main", "nord/flat", 1000.0, 640.0);
}

#[test]
fn flat_dracula_wraps_the_grid() {
    check("main_dracula_flat_560x420.png", "main", "dracula/flat", 560.0, 420.0);
}

#[test]
fn flat_macos_dark() {
    check("main_macos-dark_flat_1000x640.png", "main", "macos-dark/flat", 1000.0, 640.0);
}

#[test]
fn liquid_glass_dark() {
    check("main_macos-dark_liquid_1000x640.png", "main", "macos-dark/liquid", 1000.0, 640.0);
}

#[test]
fn liquid_glass_light() {
    check("main_macos_liquid_1000x640.png", "main", "macos/liquid", 1000.0, 640.0);
}

#[test]
fn server_window_flat() {
    check("window_nord_flat_900x560.png", "window", "nord/flat", 900.0, 560.0);
}

#[test]
fn server_window_liquid() {
    check("window_macos-dark_liquid_900x560.png", "window", "macos-dark/liquid", 900.0, 560.0);
}

#[test]
fn controls_flat() {
    check("controls_nord_flat_760x520.png", "controls", "nord/flat", 760.0, 520.0);
}

#[test]
fn controls_scrolled_is_cut_off_at_its_box() {
    check("controls-scrolled_nord_flat_760x520.png", "controls-scrolled", "nord/flat", 760.0, 520.0);
}

#[test]
fn dialog_on_glass() {
    check("dialog_macos-dark_liquid_760x520.png", "dialog", "macos-dark/liquid", 760.0, 520.0);
}
