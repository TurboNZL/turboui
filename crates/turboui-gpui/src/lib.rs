//! TurboUI desktop backend. It draws the `Scene` that `turboui-core` produced and decides nothing
//! about layout: every item is placed at the position the core computed.

use gpui_kit::prelude::*;
use gpui_kit::TestSupportExt as _;
use gpui_kit::*;
use std::borrow::Cow;
use std::sync::Arc;
use turboui_core::{layout, paint_radials, Color, DrawItem, Event, Font, Measurer, Node, Scene, ScrollArea, FONT_BOLD, FONT_MONO, FONT_REGULAR};

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
fn picture(x: f32, y: f32, w: f32, h: f32, img: Arc<RenderImage>) -> AnyElement {  // x, y relative to the parent
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

/// One scene item as an element, positioned relative to `(ox, oy)` (the origin of the clip box it sits in).
fn item_to_element(it: &DrawItem, image: Option<&Arc<RenderImage>>, ox: f32, oy: f32) -> AnyElement {
    match it {
        DrawItem::Clip { .. } | DrawItem::EndClip => unreachable!("clips are turned into containers by the caller"),
        DrawItem::Radials { x, y, w, h, .. } | DrawItem::Glass { x, y, w, h, .. } => {
            let img = image.expect("pictures are prepared for radials and glass").clone();
            let glass_shadow = if let DrawItem::Glass { radius, glass, .. } = it {
                let hsla: Hsla = c(glass.shadow).into();
                Some(div().absolute().left(px(*x - ox)).top(px(*y - oy)).w(px(*w)).h(px(*h)).rounded(px(*radius)).shadow(vec![BoxShadow {
                    color: hsla,
                    offset: point(px(0.0), px(glass.shadow_dy)),
                    blur_radius: px(glass.shadow_blur),
                    spread_radius: px(0.0),
                    inset: false,
                }]))
            } else {
                None
            };
            let pic = picture(*x - ox, *y - oy, *w, *h, img);
            match glass_shadow {
                Some(s) => div().child(s).child(pic).into_any_element(),
                None => pic,
            }
        }
        DrawItem::Rect { x, y, w, h, bg, radius, border } => {
            let mut d = div().absolute().left(px(*x - ox)).top(px(*y - oy)).w(px(*w)).h(px(*h)).rounded(px(*radius));
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
            .left(px(min_x - ox))
            .top(px(min_y - oy))
            .w(px(max_x - min_x))
            .h(px(max_y - min_y))
            .into_any_element()
        }
        DrawItem::Text { x, y, w, h, text, size, color, font, line_height, .. } => {
            let family = if *font == Font::Mono { "DejaVu Sans Mono" } else { "DejaVu Sans" };
            let mut d = div()
                .absolute()
                .left(px(*x - ox))
                .top(px(*y - oy))
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

struct Cached {
    scale: f32,
    key: Vec<DrawItem>,
    img: Arc<RenderImage>,
}

/// What a glass or ambient picture depends on: the item itself and every earlier item that overlaps it.
fn picture_key(scene: &Scene, i: usize) -> Vec<DrawItem> {
    let it = &scene.items[i];
    let mut key = vec![it.clone()];
    if matches!(it, DrawItem::Glass { .. }) {
        let (x, y, w, h) = turboui_raster::bbox(it);
        for earlier in &scene.items[..i] {
            let (ex, ey, ew, eh) = turboui_raster::bbox(earlier);
            if ex < x + w && ex + ew > x && ey < y + h && ey + eh > y {
                key.push(earlier.clone());
            }
        }
    }
    key
}

struct SceneView<S> {
    state: S,
    /// Pictures for the radial and glass items, reused while what they depend on (their own look and whatever is
    /// painted under them) and the scale stay the same. Changing a selection elsewhere does not redo any glass.
    cache: Vec<Option<Cached>>,
    build: Box<dyn Fn(&S) -> Node>,
    /// Applies an event; returns whether the screen needs redrawing.
    update: Box<dyn Fn(&mut S, &Event) -> bool>,
    measurer: Measurer,
    focus: FocusHandle,
    hovered: Option<String>,
    scrolls: Vec<ScrollArea>,
}

impl<S: 'static> SceneView<S> {
    fn send(&mut self, event: Event, cx: &mut Context<Self>) {
        if (self.update)(&mut self.state, &event) {
            cx.notify();
        }
    }
}

impl<S: 'static> Render for SceneView<S> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.focus.is_focused(window) {
            window.focus(&self.focus, cx);
        }
        let size = window.viewport_size();
        let scene = layout(&(self.build)(&self.state), f32::from(size.width), f32::from(size.height), &self.measurer);
        let scale = window.scale_factor();
        self.scrolls = scene.scrolls.clone();
        self.cache.resize_with(scene.items.len(), || None);
        // Glass cannot be blurred or bent by GPUI itself: the reference renderer makes the finished surface
        // and GPUI paints it as a picture. Ambient light is painted the same way.
        for (i, it) in scene.items.iter().enumerate() {
            if !matches!(it, DrawItem::Glass { .. } | DrawItem::Radials { .. }) {
                self.cache[i] = None;
                continue;
            }
            let key = picture_key(&scene, i);
            if matches!(&self.cache[i], Some(c) if c.scale == scale && c.key == key) {
                continue;
            }
            let img = match it {
                DrawItem::Glass { .. } => {
                    let p = turboui_raster::glass_patch(&scene, i, Color::rgb(0), &self.measurer, scale);
                    to_image(p.w, p.h, &p.rgba)
                }
                DrawItem::Radials { w, h, list, .. } => {
                    let (wd, hd) = ((w * scale).round() as u32, (h * scale).round() as u32);
                    to_image(wd, hd, &paint_radials(wd, hd, list))
                }
                _ => unreachable!(),
            };
            self.cache[i] = Some(Cached { scale, key, img });
        }

        // Turn the flat item list into elements; each Clip..EndClip pair becomes a box that cuts its children off.
        struct Frame {
            x: f32,
            y: f32,
            w: f32,
            h: f32,
            kids: Vec<AnyElement>,
        }
        let mut stack = vec![Frame { x: 0.0, y: 0.0, w: scene.width, h: scene.height, kids: vec![] }];
        for (it, pic) in scene.items.iter().zip(self.cache.iter()) {
            match it {
                DrawItem::Clip { x, y, w, h } => stack.push(Frame { x: *x, y: *y, w: *w, h: *h, kids: vec![] }),
                DrawItem::EndClip => {
                    let done = stack.pop().expect("clips are balanced");
                    let parent = stack.last_mut().expect("root frame");
                    let (px, py) = (parent.x, parent.y);
                    parent.kids.push(
                        div().absolute().left(px_(done.x - px)).top(px_(done.y - py)).w(px_(done.w)).h(px_(done.h)).overflow_hidden().children(done.kids).into_any_element(),
                    );
                }
                _ => {
                    let top = stack.last_mut().expect("root frame");
                    let (ox, oy) = (top.x, top.y);
                    top.kids.push(item_to_element(it, pic.as_ref().map(|c| &c.img), ox, oy));
                }
            }
        }
        let root_kids = stack.pop().expect("root frame").kids;

        let hits: Vec<_> = scene
            .hits
            .iter()
            .map(|h| {
                let (click_id, hover_id) = (h.id.clone(), h.id.clone());
                div()
                    .id(SharedString::from(h.id.clone()))
                    .absolute()
                    .left(px(h.x))
                    .top(px(h.y))
                    .w(px(h.w))
                    .h(px(h.h))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| this.send(Event::Click(click_id.clone()), cx)))
                    .on_hover(cx.listener(move |this, over: &bool, _, cx| {
                        let now = if *over { Some(hover_id.clone()) } else if this.hovered.as_deref() == Some(hover_id.as_str()) { None } else { this.hovered.clone() };
                        if now != this.hovered {
                            this.hovered = now.clone();
                            this.send(Event::Hover(now), cx);
                        }
                    }))
                    .test_support()
            })
            .collect();

        div()
            .relative()
            .size_full()
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _, cx| {
                let k = &ev.keystroke;
                let text = k.key_char.clone().filter(|t| !t.is_empty() && !k.modifiers.control && !k.modifiers.platform);
                this.send(Event::Key { key: k.key.clone(), text, ctrl: k.modifiers.control || k.modifiers.platform, shift: k.modifiers.shift, alt: k.modifiers.alt }, cx);
            }))
            .on_scroll_wheel(cx.listener(|this, ev: &ScrollWheelEvent, _, cx| {
                let (px_x, px_y) = (f32::from(ev.position.x), f32::from(ev.position.y));
                let area = this.scrolls.iter().rev().find(|a| px_x >= a.x && px_x < a.x + a.w && px_y >= a.y && px_y < a.y + a.h).cloned();
                if let Some(a) = area {
                    let dy = -f32::from(ev.delta.pixel_delta(px(20.0)).y);
                    this.send(Event::Scroll { id: a.id, dy, viewport: a.h, content: a.content_h }, cx);
                }
            }))
            .children(root_kids)
            .children(hits)
    }
}

use gpui_kit::px as px_;

fn add_fonts(cx: &mut App) {
    cx.text_system()
        .add_fonts(vec![Cow::Borrowed(FONT_REGULAR), Cow::Borrowed(FONT_BOLD), Cow::Borrowed(FONT_MONO)])
        .expect("bundled fonts load");
}

/// How to open the window.
pub struct Options {
    pub title: String,
    pub size: (f32, f32),
    /// Send `Event::Tick` this often (milliseconds); 0 for never.
    pub tick_ms: u64,
}

/// Opens a window showing the screen returned by `build`, laid out again whenever the window resizes.
/// `update` applies each `Event` to the state and says whether the screen must be redrawn.
pub fn run<S: 'static>(
    title: &str,
    size: (f32, f32),
    state: S,
    build: impl Fn(&S) -> Node + 'static,
    update: impl Fn(&mut S, &Event) -> bool + 'static,
) {
    run_with(Options { title: title.to_string(), size, tick_ms: 0 }, state, build, update)
}

/// Like [`run`], with options.
pub fn run_with<S: 'static>(
    options: Options,
    state: S,
    build: impl Fn(&S) -> Node + 'static,
    update: impl Fn(&mut S, &Event) -> bool + 'static,
) {
    let title: SharedString = options.title.clone().into();
    let (size, tick_ms) = (options.size, options.tick_ms);
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
            move |_, cx| {
                cx.new(|cx| {
                    if tick_ms > 0 {
                        cx.spawn(async move |this, cx| loop {
                            cx.background_executor().timer(std::time::Duration::from_millis(tick_ms)).await;
                            if this.update(cx, |view: &mut SceneView<S>, cx| view.send(Event::Tick, cx)).is_err() {
                                break;
                            }
                        })
                        .detach();
                    }
                    SceneView { state, cache: Vec::new(), build: Box::new(build), update: Box::new(update), measurer: Measurer::new(), focus: cx.focus_handle(), hovered: None, scrolls: vec![] }
                })
            },
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

/// Something to do to the offscreen window before the picture is taken.
#[cfg(feature = "headless")]
#[derive(Clone, Copy, Debug)]
pub enum Action<'a> {
    /// Click the element with this id.
    Click(&'a str),
    /// Press a key (e.g. "a", "enter", "backspace").
    Key(&'a str),
    /// Type text into whatever has the keyboard focus (as `Event::Key` with `text`, one per character).
    Type(&'a str),
    /// Send an `Event::Tick`, as the clock would.
    Tick,
    /// Wait this many milliseconds (lets background threads deliver news before the next action).
    Wait(u64),
}

/// Like [`screenshot`], but first clicks the elements with the given ids, in order, as a user would.
#[cfg(feature = "headless")]
pub fn screenshot_after_clicks<S: 'static>(
    state: S,
    build: impl Fn(&S) -> Node + 'static,
    update: impl Fn(&mut S, &Event) + 'static,
    clicks: &[&str],
    width: f32,
    height: f32,
) -> (u32, u32, Vec<u8>) {
    let actions: Vec<Action> = clicks.iter().map(|c| Action::Click(c)).collect();
    screenshot_after(state, build, move |s, e| { update(s, e); true }, &actions, width, height)
}

/// Like [`screenshot`], but first performs the actions in order, as a user would.
#[cfg(feature = "headless")]
pub fn screenshot_after<S: 'static>(
    state: S,
    build: impl Fn(&S) -> Node + 'static,
    update: impl Fn(&mut S, &Event) -> bool + 'static,
    actions: &[Action],
    width: f32,
    height: f32,
) -> (u32, u32, Vec<u8>) {
    use gpui_kit::test::{TestEventExt as _, TestWindowExt as _};
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
                move |_, cx| {
                    cx.new(|cx| SceneView {
                        state,
                        cache: Vec::new(),
                        build: Box::new(build),
                        update: Box::new(update),
                        measurer: Measurer::new(),
                        focus: cx.focus_handle(),
                        hovered: None,
                        scrolls: vec![],
                    })
                },
            )
        })
        .expect("open offscreen window");
    cx.update_window(handle, |root, window, cx| {
        window.render_frame(cx);
        let view = root.downcast::<SceneView<S>>().expect("root view");
        for action in actions {
            match action {
                Action::Click(id) => window.click(SharedString::from(id.to_string()), cx),
                Action::Key(key) => window.key_down(key, false, cx),
                Action::Type(text) => {
                    for ch in text.chars() {
                        let ev = Event::Key { key: ch.to_string(), text: Some(ch.to_string()), ctrl: false, shift: false, alt: false };
                        view.update(cx, |v, cx| v.send(ev, cx));
                    }
                }
                Action::Tick => view.update(cx, |v, cx| v.send(Event::Tick, cx)),
                Action::Wait(ms) => std::thread::sleep(std::time::Duration::from_millis(*ms)),
            }
            window.render_frame(cx);
        }
    })
    .expect("render frame");
    let img = cx.capture_screenshot(handle).expect("offscreen rendering must be available");
    (img.width(), img.height(), img.into_raw())
}
