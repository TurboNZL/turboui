/// An sRGB colour with alpha.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Color = Color { r: 0, g: 0, b: 0, a: 0 };
    pub const fn rgb(hex: u32) -> Color {
        Color { r: (hex >> 16) as u8, g: (hex >> 8) as u8, b: hex as u8, a: 255 }
    }
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }
    /// Parses `#rrggbb` or `#rrggbbaa`.
    pub fn parse(s: &str) -> Option<Color> {
        let h = s.strip_prefix('#')?;
        let v = u32::from_str_radix(h, 16).ok()?;
        match h.len() {
            6 => Some(Color::rgb(v)),
            8 => Some(Color::rgba((v >> 24) as u8, (v >> 16) as u8, (v >> 8) as u8, v as u8)),
            _ => None,
        }
    }
    /// `0xRRGGBBAA`, the order GPUI's `rgba()` takes.
    pub fn to_rgba_u32(self) -> u32 {
        (self.r as u32) << 24 | (self.g as u32) << 16 | (self.b as u32) << 8 | self.a as u32
    }
    /// Mixes towards white by `f` (0.0 to 1.0).
    pub fn lighten(self, f: f32) -> Color {
        let m = |v: u8| (v as f32 + (255.0 - v as f32) * f).round() as u8;
        Color { r: m(self.r), g: m(self.g), b: m(self.b), a: self.a }
    }
    /// Mixes towards black by `f` (0.0 to 1.0).
    pub fn darken(self, f: f32) -> Color {
        let m = |v: u8| (v as f32 * (1.0 - f)).round() as u8;
        Color { r: m(self.r), g: m(self.g), b: m(self.b), a: self.a }
    }
    /// Same colour with its alpha scaled by `f` (0.0 to 1.0).
    pub fn faded(self, f: f32) -> Color {
        Color { a: (self.a as f32 * f.clamp(0.0, 1.0)).round() as u8, ..self }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Justify {
    Start,
    Center,
    End,
    SpaceBetween,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Size {
    Auto,
    Px(f32),
    Percent(f32),
}

/// A soft round colour patch (used for ambient backgrounds). `x` is a fraction of the box width, `y` of its height,
/// and `radius` a fraction of its width.
#[derive(Clone, Debug, PartialEq)]
pub struct Radial {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub color: Color,
}

/// A "liquid glass" surface: the picture behind it is blurred, bent at the rim, tinted and lit.
#[derive(Clone, Debug, PartialEq)]
pub struct Glass {
    /// Blur of what is behind, in pixels.
    pub blur: f32,
    pub saturate: f32,
    pub brightness: f32,
    /// Tint laid over the blurred picture (its alpha is the glass density).
    pub fill: Color,
    /// Strength (0 to 1) of the diagonal white sheen.
    pub sheen: f32,
    /// Rim highlight colour; its alpha is the peak brightness.
    pub rim: Color,
    /// Bend the light at the edge (the "lens"). Off gives plain frosted glass.
    pub lens: bool,
    pub shadow_blur: f32,
    pub shadow_dy: f32,
    pub shadow: Color,
}

/// Everything a box or text can look like. Plain data: no renderer types in here.
#[derive(Clone, Debug)]
pub struct Style {
    pub dir: Dir,
    pub gap: f32,
    /// Padding as `[top, right, bottom, left]`.
    pub padding: [f32; 4],
    pub width: Size,
    pub height: Size,
    pub grow: f32,
    pub align: Align,
    pub justify: Justify,
    pub bg: Option<Color>,
    pub radius: f32,
    pub border: Option<(f32, Color)>,
    pub color: Color,
    pub font_size: f32,
    pub bold: bool,
    pub mono: bool,
    /// Row boxes only: start a new line when the children do not fit.
    pub wrap: bool,
    pub radials: Vec<Radial>,
    pub glass: Option<Glass>,
    /// Children are cut off at this box's edge.
    pub clip: bool,
    /// Vertical scroll offset in pixels (children move up by this much). Set through `Node::scroll_area`.
    pub scroll: f32,
    /// Fills its parent and is drawn above its siblings (dialogs, scrims, menus).
    pub overlay: bool,
    /// Margin on the left; `auto` pushes the node to the end of a row.
    pub margin_left_auto: bool,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            dir: Dir::Column,
            gap: 0.0,
            padding: [0.0; 4],
            width: Size::Auto,
            height: Size::Auto,
            grow: 0.0,
            align: Align::Stretch,
            justify: Justify::Start,
            bg: None,
            radius: 0.0,
            border: None,
            color: Color::rgb(0xffffff),
            font_size: 14.0,
            bold: false,
            mono: false,
            wrap: false,
            radials: vec![],
            glass: None,
            clip: false,
            scroll: 0.0,
            overlay: false,
            margin_left_auto: false,
        }
    }
}
