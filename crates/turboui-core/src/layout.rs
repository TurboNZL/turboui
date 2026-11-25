use crate::node::{Kind, Node};
use crate::style::{Align, Color, Dir, Justify, Size};
use crate::text::{Font, Measurer};
use taffy::prelude as t;

/// One thing to draw, with its final position in window pixels. Items are in paint order.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawItem {
    Rect { x: f32, y: f32, w: f32, h: f32, bg: Option<Color>, radius: f32, border: Option<(f32, Color)> },
    Text { x: f32, y: f32, w: f32, h: f32, text: String, size: f32, color: Color, font: Font, baseline: f32, line_height: f32 },
    /// Soft round colour patches inside a box (ambient background light).
    Radials { x: f32, y: f32, w: f32, h: f32, list: Vec<crate::style::Radial> },
    /// A glass surface (see [`crate::Glass`]); replaces the box's plain fill.
    Glass { x: f32, y: f32, w: f32, h: f32, radius: f32, glass: crate::style::Glass },
    /// Everything until the matching `EndClip` is cut off at this box.
    Clip { x: f32, y: f32, w: f32, h: f32 },
    EndClip,
    /// A filled polygon with points in window pixels.
    Poly { points: Vec<(f32, f32)>, color: Color },
}

/// A scrollable area and how much it holds.
#[derive(Clone, Debug, PartialEq)]
pub struct ScrollArea {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub content_h: f32,
}

/// A clickable area, reported to the app by id.
#[derive(Clone, Debug, PartialEq)]
pub struct Hit {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// The result of laying a screen out: a flat list of items. This is all a backend gets to see.
#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    pub width: f32,
    pub height: f32,
    pub items: Vec<DrawItem>,
    /// Clickable areas, topmost last.
    pub hits: Vec<Hit>,
    /// Scrollable areas, topmost last.
    pub scrolls: Vec<ScrollArea>,
}

fn dim(s: Size) -> t::Dimension {
    match s {
        Size::Auto => t::auto(),
        Size::Px(v) => t::length(v),
        Size::Percent(p) => t::percent(p),
    }
}

fn build(tree: &mut t::TaffyTree<()>, n: &Node, m: &Measurer, no_shrink: bool) -> t::NodeId {
    let s = &n.style;
    match &n.kind {
        Kind::Text(txt) => {
            let font = if s.mono { Font::Mono } else if s.bold { Font::Bold } else { Font::Regular };
            let style = t::Style {
                size: t::Size { width: t::length(m.width(font, s.font_size, txt).ceil()), height: t::length(m.line_height(s.font_size)) },
                flex_shrink: 0.0,
                ..Default::default()
            };
            tree.new_leaf(style).unwrap()
        }
        Kind::Shape(_) => {
            let style = t::Style { size: t::Size { width: dim(s.width), height: dim(s.height) }, flex_shrink: 0.0, ..Default::default() };
            tree.new_leaf(style).unwrap()
        }
        Kind::Box => {
            let scrolls = n.scroll_id.is_some();
            let kids: Vec<_> = n.children.iter().map(|c| build(tree, c, m, scrolls)).collect();
            let [pt, pr, pb, pl] = s.padding;
            let style = t::Style {
                display: t::Display::Flex,
                flex_direction: if s.dir == Dir::Row { t::FlexDirection::Row } else { t::FlexDirection::Column },
                flex_wrap: if s.wrap { t::FlexWrap::Wrap } else { t::FlexWrap::NoWrap },
                gap: t::Size { width: t::length(s.gap), height: t::length(s.gap) },
                padding: t::Rect { left: t::length(pl), right: t::length(pr), top: t::length(pt), bottom: t::length(pb) },
                size: t::Size { width: dim(s.width), height: dim(s.height) },
                flex_grow: s.grow,
                // like CSS `flex: 1`: a growing box starts from nothing, so a long list inside it scrolls instead of squashing its neighbours
                flex_basis: if s.grow > 0.0 { t::length(0.0_f32) } else { t::auto() },
                flex_shrink: if !no_shrink && s.width == Size::Auto && s.height == Size::Auto { 1.0 } else { 0.0 },
                position: if s.overlay { t::Position::Absolute } else { t::Position::Relative },
                inset: if s.overlay { t::Rect { left: t::length(0.0_f32), right: t::length(0.0_f32), top: t::length(0.0_f32), bottom: t::length(0.0_f32) } } else { t::Rect { left: t::auto(), right: t::auto(), top: t::auto(), bottom: t::auto() } },
                min_size: t::Size { width: t::length(0.0_f32), height: t::length(0.0_f32) },
                align_items: Some(match s.align {
                    Align::Start => t::AlignItems::FlexStart,
                    Align::Center => t::AlignItems::Center,
                    Align::End => t::AlignItems::FlexEnd,
                    Align::Stretch => t::AlignItems::Stretch,
                }),
                justify_content: Some(match s.justify {
                    Justify::Start => t::JustifyContent::FlexStart,
                    Justify::Center => t::JustifyContent::Center,
                    Justify::End => t::JustifyContent::FlexEnd,
                    Justify::SpaceBetween => t::JustifyContent::SpaceBetween,
                }),
                ..Default::default()
            };
            tree.new_with_children(style, &kids).unwrap()
        }
    }
}

struct Collector<'a> {
    m: &'a Measurer,
    out: Vec<DrawItem>,
    hits: Vec<Hit>,
    scrolls: Vec<ScrollArea>,
    clips: Vec<(f32, f32, f32, f32)>,
}

fn intersect(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> Option<(f32, f32, f32, f32)> {
    let (x0, y0) = (a.0.max(b.0), a.1.max(b.1));
    let (x1, y1) = ((a.0 + a.2).min(b.0 + b.2), (a.1 + a.3).min(b.1 + b.3));
    if x1 > x0 && y1 > y0 {
        Some((x0, y0, x1 - x0, y1 - y0))
    } else {
        None
    }
}

fn collect(tree: &t::TaffyTree<()>, id: t::NodeId, n: &Node, ox: f32, oy: f32, c: &mut Collector) {
    let m = c.m;
    let l = tree.layout(id).unwrap();
    let (x, y, w, h) = (ox + l.location.x, oy + l.location.y, l.size.width, l.size.height);
    let s = &n.style;
    if let Some(hid) = &n.id {
        // a clickable area only counts where it is visible
        let visible = c.clips.last().map_or(Some((x, y, w, h)), |clip| intersect((x, y, w, h), *clip));
        if let Some((hx, hy, hw, hh)) = visible {
            c.hits.push(Hit { id: hid.clone(), x: hx, y: hy, w: hw, h: hh });
        }
    }
    match &n.kind {
        Kind::Shape(faces) => {
            for f in faces {
                c.out.push(DrawItem::Poly { points: f.points.iter().map(|(px, py)| (x + px * w, y + py * h)).collect(), color: f.color });
            }
        }
        Kind::Text(txt) => {
            let font = if s.mono { Font::Mono } else if s.bold { Font::Bold } else { Font::Regular };
            c.out.push(DrawItem::Text {
                x, y, w, h,
                text: txt.clone(),
                size: s.font_size,
                color: s.color,
                font,
                baseline: m.baseline(font, s.font_size),
                line_height: m.line_height(s.font_size),
            });
        }
        Kind::Box => {
            if let Some(g) = &s.glass {
                c.out.push(DrawItem::Glass { x, y, w, h, radius: s.radius, glass: g.clone() });
            } else if s.bg.is_some() || s.border.is_some() {
                c.out.push(DrawItem::Rect { x, y, w, h, bg: s.bg, radius: s.radius, border: s.border });
            }
            if !s.radials.is_empty() {
                c.out.push(DrawItem::Radials { x, y, w, h, list: s.radials.clone() });
            }
            let mut child_y = y;
            if s.clip {
                let rect = c.clips.last().map_or(Some((x, y, w, h)), |clip| intersect((x, y, w, h), *clip));
                c.out.push(DrawItem::Clip { x, y, w, h });
                // an empty clip still has to be balanced, so keep a zero-size rectangle on the stack
                c.clips.push(rect.unwrap_or((x, y, 0.0, 0.0)));
                // the offset is kept inside the content, so `f32::MAX` means "scrolled to the end"
                let max_scroll = (l.content_size.height - h).max(0.0);
                child_y = y - s.scroll.clamp(0.0, max_scroll);
                if let Some(sid) = &n.scroll_id {
                    c.scrolls.push(ScrollArea { id: sid.clone(), x, y, w, h, content_h: l.content_size.height.max(h) });
                }
            }
            let kids = tree.children(id).unwrap();
            for (k, kid) in n.children.iter().zip(kids) {
                collect(tree, kid, k, x, child_y, c);
            }
            if s.clip {
                c.clips.pop();
                c.out.push(DrawItem::EndClip);
            }
        }
    }
}

/// Lays `root` out to fill a `width` x `height` window.
pub fn layout(root: &Node, width: f32, height: f32, m: &Measurer) -> Scene {
    let mut tree: t::TaffyTree<()> = t::TaffyTree::new();
    let mut root = root.clone();
    root.style.width = Size::Px(width);
    root.style.height = Size::Px(height);
    let id = build(&mut tree, &root, m, false);
    tree.compute_layout(id, t::Size { width: t::AvailableSpace::Definite(width), height: t::AvailableSpace::Definite(height) })
        .unwrap();
    let mut c = Collector { m, out: vec![], hits: vec![], scrolls: vec![], clips: vec![] };
    collect(&tree, id, &root, 0.0, 0.0, &mut c);
    Scene { width, height, items: c.out, hits: c.hits, scrolls: c.scrolls }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{col, row, spacer, text as label};

    fn rects(s: &Scene) -> Vec<(f32, f32, f32, f32)> {
        s.items
            .iter()
            .filter_map(|i| match i {
                DrawItem::Rect { x, y, w, h, .. } => Some((*x, *y, *w, *h)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn row_places_children_left_to_right_with_gap_and_padding() {
        let c = Color::rgb(0xff0000);
        let root = row().pad(10.0).gap(5.0).child(col().w(20.0).h(20.0).bg(c)).child(col().w(30.0).h(20.0).bg(c));
        let s = layout(&root, 200.0, 100.0, &Measurer::new());
        assert_eq!(rects(&s), vec![(10.0, 10.0, 20.0, 20.0), (35.0, 10.0, 30.0, 20.0)]);
    }

    #[test]
    fn spacer_pushes_the_last_child_to_the_end() {
        let c = Color::rgb(0x00ff00);
        let root = row().child(col().w(10.0).h(10.0).bg(c)).child(spacer()).child(col().w(10.0).h(10.0).bg(c));
        let s = layout(&root, 100.0, 10.0, &Measurer::new());
        assert_eq!(rects(&s), vec![(0.0, 0.0, 10.0, 10.0), (90.0, 0.0, 10.0, 10.0)]);
    }

    #[test]
    fn text_is_measured_the_same_every_time() {
        let m = Measurer::new();
        let a = layout(&col().child(label("Hello").size(16.0)), 300.0, 100.0, &m);
        let b = layout(&col().child(label("Hello").size(16.0)), 300.0, 100.0, &m);
        assert_eq!(a, b);
        match &a.items[0] {
            DrawItem::Text { w, h, .. } => assert!(*w > 20.0 && *h >= 16.0),
            other => panic!("expected text, got {other:?}"),
        }
    }
}
