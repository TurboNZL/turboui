use crate::node::{Kind, Node};
use crate::style::{Align, Color, Dir, Justify, Size};
use crate::text::{Font, Measurer};
use taffy::prelude as t;

/// One thing to draw, with its final position in window pixels. Items are in paint order.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawItem {
    Rect { x: f32, y: f32, w: f32, h: f32, bg: Option<Color>, radius: f32, border: Option<(f32, Color)> },
    Text { x: f32, y: f32, w: f32, h: f32, text: String, size: f32, color: Color, font: Font, baseline: f32, line_height: f32 },
}

/// The result of laying a screen out: a flat list of items. This is all a backend gets to see.
#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    pub width: f32,
    pub height: f32,
    pub items: Vec<DrawItem>,
}

fn dim(s: Size) -> t::Dimension {
    match s {
        Size::Auto => t::auto(),
        Size::Px(v) => t::length(v),
        Size::Percent(p) => t::percent(p),
    }
}

fn build(tree: &mut t::TaffyTree<()>, n: &Node, m: &Measurer) -> t::NodeId {
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
        Kind::Box => {
            let kids: Vec<_> = n.children.iter().map(|c| build(tree, c, m)).collect();
            let [pt, pr, pb, pl] = s.padding;
            let style = t::Style {
                display: t::Display::Flex,
                flex_direction: if s.dir == Dir::Row { t::FlexDirection::Row } else { t::FlexDirection::Column },
                gap: t::Size { width: t::length(s.gap), height: t::length(s.gap) },
                padding: t::Rect { left: t::length(pl), right: t::length(pr), top: t::length(pt), bottom: t::length(pb) },
                size: t::Size { width: dim(s.width), height: dim(s.height) },
                flex_grow: s.grow,
                flex_shrink: if s.width == Size::Auto && s.height == Size::Auto { 1.0 } else { 0.0 },
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

fn collect(tree: &t::TaffyTree<()>, id: t::NodeId, n: &Node, ox: f32, oy: f32, m: &Measurer, out: &mut Vec<DrawItem>) {
    let l = tree.layout(id).unwrap();
    let (x, y, w, h) = (ox + l.location.x, oy + l.location.y, l.size.width, l.size.height);
    let s = &n.style;
    match &n.kind {
        Kind::Text(txt) => {
            let font = if s.mono { Font::Mono } else if s.bold { Font::Bold } else { Font::Regular };
            out.push(DrawItem::Text {
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
            if s.bg.is_some() || s.border.is_some() {
                out.push(DrawItem::Rect { x, y, w, h, bg: s.bg, radius: s.radius, border: s.border });
            }
            let kids = tree.children(id).unwrap();
            for (k, kid) in n.children.iter().zip(kids) {
                collect(tree, kid, k, x, y, m, out);
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
    let id = build(&mut tree, &root, m);
    tree.compute_layout(id, t::Size { width: t::AvailableSpace::Definite(width), height: t::AvailableSpace::Definite(height) })
        .unwrap();
    let mut items = vec![];
    collect(&tree, id, &root, 0.0, 0.0, m, &mut items);
    Scene { width, height, items }
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
