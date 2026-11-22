//! Golden-image test: the sample screen must render to exactly the saved picture.
//! After an intentional change, regenerate with `TURBOUI_BLESS=1 cargo test` and review the PNG.
use turboui_core::{layout, Measurer, Theme};
use turboui_demo::{screen, THEME};

fn render(w: f32, h: f32) -> Vec<u8> {
    let theme = Theme::from_toml(THEME).unwrap();
    let m = Measurer::new();
    let scene = layout(&screen::server_manager(&theme), w, h, &m);
    turboui_raster::png_bytes(&turboui_raster::render(&scene, theme.color("bg"), &m))
}

fn check(name: &str, w: f32, h: f32) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../goldens").join(name);
    let got = render(w, h);
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
    check("server_manager_1000x640.png", 1000.0, 640.0);
}

#[test]
fn server_manager_narrow() {
    check("server_manager_760x520.png", 760.0, 520.0);
}
