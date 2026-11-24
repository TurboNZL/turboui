//! TurboUI desktop backend. It draws the `Scene` that `turboui-core` produced and decides nothing
//! about layout: every item is placed at the position the core computed.

use gpui_kit::prelude::*;
use gpui_kit::TestSupportExt as _;
use gpui_kit::*;
use std::borrow::Cow;
use turboui_core::{layout, Color, DrawItem, Font, Measurer, Node, FONT_BOLD, FONT_MONO, FONT_REGULAR};

fn c(col: Color) -> Rgba {
    rgba(col.to_rgba_u32())
}

fn item_to_element(it: &DrawItem) -> AnyElement {
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
            d.into_any_element()
        }
        DrawItem::Poly { points, color } => {
            let (min_x, min_y) = points.iter().fold((f32::MAX, f32::MAX), |(a, b), (x, y)| (a.min(*x), b.min(*y)));
            let (max_x, max_y) = points.iter().fold((f32::MIN, f32::MIN), |(a, b), (x, y)| (a.max(*x), b.max(*y)));
            let local: Vec<(f32, f32)> = points.iter().map(|(x, y)| (x - min_x, y - min_y)).collect();
            let fill = c(*color);
            canvas(
                |_, _, _| (),
                move |bounds, _, window, _| {
                    let mut pb = PathBuilder::fill();
                    for (i, (x, y)) in local.iter().enumerate() {
                        let p = bounds.origin + point(px(*x), px(*y));
                        if i == 0 { pb.move_to(p) } else { pb.line_to(p) }
                    }
                    pb.close();
                    if let Ok(path) = pb.build() {
                        window.paint_path(path, fill);
                    }
                },
            )
            .absolute()
            .left(px(min_x))
            .top(px(min_y))
            .w(px(max_x - min_x))
            .h(px(max_y - min_y))
            .into_any_element()
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
            d.child(SharedString::from(text.clone())).into_any_element()
        }
    }
}

struct SceneView<S> {
    state: S,
    build: Box<dyn Fn(&S) -> Node>,
    update: Box<dyn Fn(&mut S, &str)>,
    measurer: Measurer,
}

impl<S: 'static> Render for SceneView<S> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let size = window.viewport_size();
        let scene = layout(&(self.build)(&self.state), f32::from(size.width), f32::from(size.height), &self.measurer);
        let hits = scene.hits.iter().map(|h| {
            let id = h.id.clone();
            div()
                .id(SharedString::from(h.id.clone()))
                .absolute()
                .left(px(h.x))
                .top(px(h.y))
                .w(px(h.w))
                .h(px(h.h))
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    (this.update)(&mut this.state, &id);
                    cx.notify();
                }))
                .test_support()
        });
        div().relative().size_full().children(scene.items.iter().map(item_to_element)).children(hits)
    }
}

fn add_fonts(cx: &mut App) {
    cx.text_system()
        .add_fonts(vec![Cow::Borrowed(FONT_REGULAR), Cow::Borrowed(FONT_BOLD), Cow::Borrowed(FONT_MONO)])
        .expect("bundled fonts load");
}

/// Opens a window showing the screen returned by `build`, laid out again whenever the window resizes.
pub fn run<S: 'static>(
    title: &str,
    size: (f32, f32),
    state: S,
    build: impl Fn(&S) -> Node + 'static,
    update: impl Fn(&mut S, &str) + 'static,
) {
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
            move |_, cx| cx.new(|_| SceneView { state, build: Box::new(build), update: Box::new(update), measurer: Measurer::new() }),
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
pub fn screenshot<S: 'static>(state: S, build: impl Fn(&S) -> Node + 'static, width: f32, height: f32) -> (u32, u32, Vec<u8>) {
    screenshot_after_clicks(state, build, |_, _| {}, &[], width, height)
}

/// Like [`screenshot`], but first clicks the elements with the given ids, in order, as a user would.
#[cfg(feature = "headless")]
pub fn screenshot_after_clicks<S: 'static>(
    state: S,
    build: impl Fn(&S) -> Node + 'static,
    update: impl Fn(&mut S, &str) + 'static,
    clicks: &[&str],
    width: f32,
    height: f32,
) -> (u32, u32, Vec<u8>) {
    use gpui_kit::test::TestWindowExt as _;
    use std::sync::Arc;
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
                move |_, cx| cx.new(|_| SceneView { state, build: Box::new(build), update: Box::new(update), measurer: Measurer::new() }),
            )
        })
        .expect("open offscreen window");
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        for id in clicks {
            window.click(SharedString::from(id.to_string()), cx);
            window.render_frame(cx);
        }
    })
    .expect("render frame");
    let img = cx.capture_screenshot(handle).expect("offscreen rendering must be available");
    (img.width(), img.height(), img.into_raw())
}
