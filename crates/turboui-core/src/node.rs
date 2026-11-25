use crate::style::*;

/// One filled polygon of a [`Kind::Shape`], with points in 0..1 of the shape's box.
#[derive(Clone, Debug)]
pub struct Face {
    pub points: Vec<(f32, f32)>,
    pub color: Color,
}

#[derive(Clone, Debug)]
pub enum Kind {
    Box,
    Text(String),
    /// Filled polygons (icons, logos). Needs a fixed width and height.
    Shape(Vec<Face>),
}

/// One element of a screen. Build with [`col`], [`row`], [`label`] and [`spacer`].
#[derive(Clone, Debug)]
pub struct Node {
    pub kind: Kind,
    pub style: Style,
    pub children: Vec<Node>,
    /// Makes the node clickable: backends report this id when it is clicked.
    pub id: Option<String>,
    /// Makes the node a scrollable area (wheel events are reported under this id).
    pub scroll_id: Option<String>,
}

fn boxed(dir: Dir) -> Node {
    Node { kind: Kind::Box, style: Style { dir, ..Style::default() }, children: vec![], id: None, scroll_id: None }
}
/// A box that stacks its children top to bottom.
pub fn col() -> Node {
    boxed(Dir::Column)
}
/// A box that lays its children out left to right.
pub fn row() -> Node {
    boxed(Dir::Row)
}
/// Single-line text.
pub fn text(s: impl Into<String>) -> Node {
    Node { kind: Kind::Text(s.into()), style: Style::default(), children: vec![], id: None, scroll_id: None }
}
/// A `w` x `h` picture made of filled polygons.
pub fn shape(w: f32, h: f32, faces: Vec<Face>) -> Node {
    Node { kind: Kind::Shape(faces), style: Style { width: Size::Px(w), height: Size::Px(h), ..Style::default() }, children: vec![], id: None, scroll_id: None }
}
/// An empty box that takes up all spare room.
pub fn spacer() -> Node {
    col().grow(1.0)
}

impl Node {
    pub fn child(mut self, n: Node) -> Self {
        self.children.push(n);
        self
    }
    pub fn children(mut self, it: impl IntoIterator<Item = Node>) -> Self {
        self.children.extend(it);
        self
    }
    pub fn gap(mut self, v: f32) -> Self {
        self.style.gap = v;
        self
    }
    /// Same padding on all four sides.
    pub fn pad(mut self, v: f32) -> Self {
        self.style.padding = [v; 4];
        self
    }
    /// Horizontal then vertical padding.
    pub fn pad_xy(mut self, x: f32, y: f32) -> Self {
        self.style.padding = [y, x, y, x];
        self
    }
    pub fn w(mut self, v: f32) -> Self {
        self.style.width = Size::Px(v);
        self
    }
    pub fn h(mut self, v: f32) -> Self {
        self.style.height = Size::Px(v);
        self
    }
    pub fn w_full(mut self) -> Self {
        self.style.width = Size::Percent(1.0);
        self
    }
    pub fn h_full(mut self) -> Self {
        self.style.height = Size::Percent(1.0);
        self
    }
    pub fn grow(mut self, v: f32) -> Self {
        self.style.grow = v;
        self
    }
    pub fn align(mut self, a: Align) -> Self {
        self.style.align = a;
        self
    }
    pub fn justify(mut self, j: Justify) -> Self {
        self.style.justify = j;
        self
    }
    pub fn bg(mut self, c: Color) -> Self {
        self.style.bg = Some(c);
        self
    }
    pub fn radius(mut self, r: f32) -> Self {
        self.style.radius = r;
        self
    }
    pub fn border(mut self, width: f32, c: Color) -> Self {
        self.style.border = Some((width, c));
        self
    }
    pub fn color(mut self, c: Color) -> Self {
        self.style.color = c;
        self
    }
    pub fn size(mut self, px: f32) -> Self {
        self.style.font_size = px;
        self
    }
    pub fn bold(mut self) -> Self {
        self.style.bold = true;
        self
    }
    pub fn radials(mut self, r: Vec<Radial>) -> Self {
        self.style.radials = r;
        self
    }
    pub fn glass(mut self, g: Glass) -> Self {
        self.style.glass = Some(g);
        self
    }
    /// Cuts children off at this box's edge.
    pub fn clip(mut self) -> Self {
        self.style.clip = true;
        self
    }
    /// A scrollable area: clips its children and shifts them up by `offset`. Wheel events over it arrive as
    /// `Event::Scroll` with this id.
    pub fn scroll_area(mut self, id: impl Into<String>, offset: f32) -> Self {
        self.scroll_id = Some(id.into());
        self.style.clip = true;
        self.style.scroll = offset;
        self
    }
    /// Fills its parent and sits above its siblings (for dialogs, scrims and menus). Put it inside a box.
    pub fn overlay(mut self) -> Self {
        self.style.overlay = true;
        self
    }
    pub fn wrap(mut self) -> Self {
        self.style.wrap = true;
        self
    }
    /// Marks the node clickable under this id.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
    pub fn mono(mut self) -> Self {
        self.style.mono = true;
        self
    }
}
