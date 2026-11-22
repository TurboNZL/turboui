use fontdue::{Font as FdFont, FontSettings};

pub const FONT_REGULAR: &[u8] = include_bytes!("../fonts/DejaVuSans.ttf");
pub const FONT_BOLD: &[u8] = include_bytes!("../fonts/DejaVuSans-Bold.ttf");
pub const FONT_MONO: &[u8] = include_bytes!("../fonts/DejaVuSansMono.ttf");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    Regular,
    Bold,
    Mono,
}

/// Measures text. This is the single source of text sizes, so layout is the same on every backend.
pub struct Measurer {
    regular: FdFont,
    bold: FdFont,
    mono: FdFont,
}

pub const LINE_HEIGHT: f32 = 1.3;

impl Measurer {
    pub fn new() -> Measurer {
        let load = |b: &[u8]| FdFont::from_bytes(b, FontSettings::default()).expect("bundled font is valid");
        Measurer { regular: load(FONT_REGULAR), bold: load(FONT_BOLD), mono: load(FONT_MONO) }
    }
    pub fn font(&self, f: Font) -> &FdFont {
        match f {
            Font::Regular => &self.regular,
            Font::Bold => &self.bold,
            Font::Mono => &self.mono,
        }
    }
    /// Width of a single line of text in pixels.
    pub fn width(&self, f: Font, size: f32, s: &str) -> f32 {
        let font = self.font(f);
        s.chars().map(|c| font.metrics(c, size).advance_width).sum()
    }
    pub fn line_height(&self, size: f32) -> f32 {
        (size * LINE_HEIGHT).round()
    }
    /// Distance from the top of a line box to the text baseline.
    pub fn baseline(&self, f: Font, size: f32) -> f32 {
        let lm = self.font(f).horizontal_line_metrics(size).expect("font has line metrics");
        let content = lm.ascent - lm.descent;
        let line = self.line_height(size);
        ((line - content) / 2.0 + lm.ascent).round()
    }
}

impl Default for Measurer {
    fn default() -> Self {
        Self::new()
    }
}
