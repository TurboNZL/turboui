//! TurboUI desktop backend. It draws the `Scene` that `turboui-core` produced and decides nothing
//! about layout: every item is placed at the position the core computed.

use gpui_kit::prelude::*;
use gpui_kit::TestSupportExt as _;
use gpui_kit::*;
use std::borrow::Cow;
use std::sync::Arc;
use turboui_core::{layout, paint_radials, Color, DrawItem, Font, Measurer, Node, Scene, FONT_BOLD, FONT_MONO, FONT_REGULAR};

fn c(col: Color) -> Rgba {
    rgba(col.to_rgba_u32())
}

/// Straight-alpha RGBA bytes to a GPUI image. GPUI wants BGRA and premultiplies in its own shader, so the
/// colours must stay straight (premultiplying here made soft edges twice as faint).
fn to_image(w: u32, h: u32, rgba: &[u8]) -> Arc<RenderImage> {
    let mut bgra = Vec::with_capacity(rgba.len());
    for px in rgba.chunks_exact(4) {
        bgra.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
    }
    let img = image::RgbaImage::from_raw(w, h, bgra).expect("image size matches");
    Arc::new(RenderImage::new(smallvec::smallvec![image::Frame::new(img)]))
}

/// A picture painted into the box `(x, y, w, h)`.
fn picture(x: f32, y: f32, w: f32, h: f32, img: Arc<RenderImage>) -> AnyElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let _ = window.paint_image(bounds, bounds, Corners::default(), img, 0, false);
        },
    )
    .absolute()
    .left(px(x))
    .top(px(y))
    .w(px(w))
    .h(px(h))
    .into_any_element()
}

fn item_to_element(it: &DrawItem, image: Option<&Arc<RenderImage>>) -> AnyElement {
    match it {
        DrawItem::Radials { x, y, w, h, .. } | DrawItem::Glass { x, y, w, h, .. } => {
            let img = image.expect("pictures are prepared for radials and glass").clone();
            let glass_shadow = if let DrawItem::Glass { radius, glass, .. } = it {
                let hsla: Hsla = c(glass.shadow).into();
                Some(div().absolute().left(px(*x)).top(px(*y)).w(px(*w)).h(px(*h)).rounded(px(*radius)).shadow(vec![BoxShadow {
                    color: hsla,
                    offset: point(px(0.0), px(glass.shadow_dy)),
                    blur_radius: px(glass.shadow_blur),
                    spread_radius: px(0.0),
                    inset: false,
                }]))
            } else {
                None
            };
            let pic = picture(*x, *y, *w, *h, img);
            match glass_shadow {
                Some(s) => div().child(s).child(pic).into_any_element(),
                None => pic,
            }
        }
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
    /// Pictures for the radial and glass items of the last scene, valid while the scene and scale stay the same.
    cache: Option<(Scene, f32, Vec<Option<Arc<RenderImage>>>)>,
    build: Box<dyn Fn(&S) -> Node>,
    update: Box<dyn Fn(&mut S, &str)>,
    measurer: Measurer,
}

impl<S: 'static> Render for SceneView<S> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let size = window.viewport_size();
        let scene = layout(&(self.build)(&self.state), f32::from(size.width), f32::from(size.height), &self.measurer);
        let scale = window.scale_factor();
        let fresh = !matches!(&self.cache, Some((s, sc, _)) if *s == scene && *sc == scale);
        if fresh {
            // Glass cannot be blurred or bent by GPUI itself: the reference renderer makes the finished surface
            // and GPUI paints it as a picture. Ambient light is painted the same way.
            let pics = scene
                .items
                .iter()
                .enumerate()
                .map(|(i, it)| match it {
                    DrawItem::Glass { .. } => {
                        let p = turboui_raster::glass_patch(&scene, i, Color::rgb(0), &self.measurer, scale);
                        Some(to_image(p.w, p.h, &p.rgba))
                    }
                    DrawItem::Radials { w, h, list, .. } => {
                        let (wd, hd) = ((w * scale).round() as u32, (h * scale).round() as u32);
                        Some(to_image(wd, hd, &paint_radials(wd, hd, list)))
                    }
                    _ => None,
                })
                .collect();
            self.cache = Some((scene.clone(), scale, pics));
        }
        let pics = &self.cache.as_ref().expect("cache was just filled").2;
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
        div().relative().size_full().children(scene.items.iter().zip(pics.iter()).map(|(it, pic)| item_to_element(it, pic.as_ref()))).children(hits)
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
            move |_, cx| cx.new(|_| SceneView { state, cache: None, build: Box::new(build), update: Box::new(update), measurer: Measurer::new() }),
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
                move |_, cx| cx.new(|_| SceneView { state, cache: None, build: Box::new(build), update: Box::new(update), measurer: Measurer::new() }),
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
