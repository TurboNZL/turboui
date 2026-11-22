use crate::style::*;

#[derive(Clone, Debug)]
pub enum Kind {
    Box,
    Text(String),
}

/// One element of a screen. Build with [`col`], [`row`], [`label`] and [`spacer`].
#[derive(Clone, Debug)]
pub struct Node {
    pub kind: Kind,
    pub style: Style,
    pub children: Vec<Node>,
}

fn boxed(dir: Dir) -> Node {
    Node { kind: Kind::Box, style: Style { dir, ..Style::default() }, children: vec![] }
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
    Node { kind: Kind::Text(s.into()), style: Style::default(), children: vec![] }
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
    pub fn mono(mut self) -> Self {
        self.style.mono = true;
        self
    }
}
