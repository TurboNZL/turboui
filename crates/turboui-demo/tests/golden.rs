//! Golden-image test: the sample screen must render to exactly the saved picture.
//! After an intentional change, regenerate with `TURBOUI_BLESS=1 cargo test` and review the PNG.
use turboui_core::{layout, Measurer, Theme};
use turboui_demo::{screen, THEME};

fn render(which: &str, w: f32, h: f32) -> Vec<u8> {
    let theme = Theme::from_toml(THEME).unwrap();
    let m = Measurer::new();
    let node = match which {
        "main" => screen::server_manager(&theme, &screen::State::sample()),
        "window" => screen::server_window(&theme),
        _ => unreachable!(),
    };
    let scene = layout(&node, w, h, &m);
    turboui_raster::png_bytes(&turboui_raster::render(&scene, theme.color("window"), &m))
}

fn check(name: &str, which: &str, w: f32, h: f32) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../goldens").join(name);
    let got = render(which, w, h);
    if std::env::var("TURBOUI_BLESS").is_ok() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &got).unwrap();
        return;
    }
    let want = std::fs::read(&path).unwrap_or_else(|_| panic!("missing golden {name}; run with TURBOUI_BLESS=1"));
    assert!(got == want, "{name} no longer matches its golden image; if intended, re-run with TURBOUI_BLESS=1");
}

#[test]
fn server_manager_wide() {
    check("server_manager_1000x640.png", "main", 1000.0, 640.0);
}

#[test]
fn server_manager_narrow_wraps_the_grid() {
    check("server_manager_560x420.png", "main", 560.0, 420.0);
}

#[test]
fn server_window_console() {
    check("server_window_900x560.png", "window", 900.0, 560.0);
}
