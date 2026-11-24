//! TurboUI desktop backend. It draws the `Scene` that `turboui-core` produced and decides nothing
//! about layout: every item is placed at the position the core computed.

use gpui_kit::prelude::*;
use gpui_kit::*;
use std::borrow::Cow;
use turboui_core::{layout, Color, DrawItem, Font, Measurer, Node, FONT_BOLD, FONT_MONO, FONT_REGULAR};

fn c(col: Color) -> Rgba {
    rgba(col.to_rgba_u32())
}

fn item_to_element(it: &DrawItem) -> Div {
    match it {
        DrawItem::Rect { x, y, w, h, bg, radius, border } => {
            let mut d = div().absolute().left(px(*x)).top(px(*y)).w(px(*w)).h(px(*h)).rounded(px(*radius));
            if let Some(bg) = bg {
                d = d.bg(c(*bg));
            }
            if let Some((bw, bc)) = border {
                d = d.border_color(c(*bc));
                d = if *bw <= 1.5 { d.border_1() } else if *bw <= 2.5 { d.border_2() } else { d.border_4() };
            }
            d
        }
        DrawItem::Text { x, y, w, h, text, size, color, font, line_height, .. } => {
            let family = if *font == Font::Mono { "DejaVu Sans Mono" } else { "DejaVu Sans" };
            let mut d = div()
                .absolute()
                .left(px(*x))
                .top(px(*y))
                .w(px(*w))
                .h(px(*h))
                .whitespace_nowrap()
                .font_family(family)
                .text_size(px(*size))
                .line_height(px(*line_height))
                .text_color(c(*color));
            if *font == Font::Bold {
                d = d.font_weight(FontWeight::BOLD);
            }
            d.child(SharedString::from(text.clone()))
        }
    }
}

struct SceneView {
    build: Box<dyn Fn() -> Node>,
    measurer: Measurer,
}

impl Render for SceneView {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let size = window.viewport_size();
        let scene = layout(&(self.build)(), f32::from(size.width), f32::from(size.height), &self.measurer);
        div().relative().size_full().children(scene.items.iter().map(item_to_element))
    }
}

fn add_fonts(cx: &mut App) {
    cx.text_system()
        .add_fonts(vec![Cow::Borrowed(FONT_REGULAR), Cow::Borrowed(FONT_BOLD), Cow::Borrowed(FONT_MONO)])
        .expect("bundled fonts load");
}

/// Opens a window showing the screen returned by `build`, laid out again whenever the window resizes.
pub fn run(title: &str, size: (f32, f32), build: impl Fn() -> Node + 'static) {
    let title: SharedString = title.to_string().into();
    gpui_kit::application().run(move |cx: &mut App| {
        gpui_kit::init(cx);
        add_fonts(cx);
        let bounds = Bounds::centered(None, gpui_kit::size(px(size.0), px(size.1)), cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions { title: Some(title), ..Default::default() }),
                ..Default::default()
            },
            cx,
            move |_, cx| cx.new(|_| SceneView { build: Box::new(build), measurer: Measurer::new() }),
        )
        .expect("open window");
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        cx.activate(true);
    });
}

/// Draws the screen offscreen with the real GPUI renderer and returns `(width, height, RGBA bytes)`.
/// Needs a Vulkan driver (software lavapipe is fine). Used to compare GPUI against the reference renderer.
#[cfg(feature = "headless")]
pub fn screenshot(build: impl Fn() -> Node + 'static, width: f32, height: f32) -> (u32, u32, Vec<u8>) {
    use std::sync::Arc;
    use gpui_kit::test::TestWindowExt as _;
    let mut cx = HeadlessAppContext::with_platform(
        gpui_kit::platform::current_platform(true).text_system(),
        Arc::new(gpui_kit::assets::Assets),
        gpui_kit::platform::current_headless_renderer,
    );
    cx.update(gpui_kit::init);
    cx.update(add_fonts);
    let (handle, _) = cx
        .update(|cx| {
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds { origin: Default::default(), size: gpui_kit::size(px(width), px(height)) })),
                    focus: false,
                    show: false,
                    ..Default::default()
                },
                cx,
                move |_, cx| cx.new(|_| SceneView { build: Box::new(build), measurer: Measurer::new() }),
            )
        })
        .expect("open offscreen window");
    cx.update_window(handle, |_, window, cx| window.render_frame(cx)).expect("render frame");
    let img = cx.capture_screenshot(handle).expect("offscreen rendering must be available");
    (img.width(), img.height(), img.into_raw())
}
