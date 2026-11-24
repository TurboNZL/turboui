use crate::style::{Color, Glass, Radial};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum ThemeError {
    Parse(String),
    BadColor(String, String),
}
impl std::fmt::Display for ThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeError::Parse(e) => write!(f, "theme file is not valid: {e}"),
            ThemeError::BadColor(k, v) => write!(f, "theme colour `{k}` is not understood: `{v}`"),
        }
    }
}
impl std::error::Error for ThemeError {}

#[derive(Deserialize)]
struct PaletteFile {
    name: String,
    mode: String,
    colors: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct GlassFile {
    blur: f32,
    saturate: f32,
    brightness: f32,
    fill: String,
    sheen: f32,
    rim: String,
    lens: bool,
    shadow_blur: f32,
    shadow_dy: f32,
    shadow: String,
}

#[derive(Deserialize)]
struct RadialFile {
    x: f32,
    y: f32,
    radius: f32,
    color: String,
}

#[derive(Deserialize)]
struct StyleFile {
    name: String,
    #[serde(default)]
    numbers: BTreeMap<String, f32>,
    glass_dark: Option<GlassFile>,
    glass_light: Option<GlassFile>,
    #[serde(default)]
    ambient_dark: Vec<RadialFile>,
    #[serde(default)]
    ambient_light: Vec<RadialFile>,
}

/// A look: a colour palette plus a style (shapes and materials), loaded from two TOML files.
///
/// Colour tokens use turbothemes' names: `bg surface raised line hover fg muted accent accent-fg good warn bad top`.
/// A colour written as `token/0.4` means that token at 40% opacity; `white/0.1` and `black/0.3` also work.
#[derive(Clone, Debug)]
pub struct Theme {
    pub name: String,
    pub dark: bool,
    colors: BTreeMap<String, Color>,
    numbers: BTreeMap<String, f32>,
    glass: Option<Glass>,
    ambient: Vec<Radial>,
}

impl Theme {
    /// `palette` is a file from `themes/palettes/`, `style` one from `themes/styles/`.
    pub fn from_parts(palette: &str, style: &str) -> Result<Theme, ThemeError> {
        let p: PaletteFile = toml::from_str(palette).map_err(|e| ThemeError::Parse(e.to_string()))?;
        let s: StyleFile = toml::from_str(style).map_err(|e| ThemeError::Parse(e.to_string()))?;
        let dark = p.mode == "dark";
        let mut colors = BTreeMap::new();
        for (k, v) in &p.colors {
            colors.insert(k.clone(), Color::parse(v).ok_or_else(|| ThemeError::BadColor(k.clone(), v.clone()))?);
        }
        let mut t = Theme { name: format!("{} / {}", p.name, s.name), dark, colors, numbers: s.numbers, glass: None, ambient: vec![] };
        let g = if dark { s.glass_dark } else { s.glass_light };
        if let Some(g) = g {
            t.glass = Some(Glass {
                blur: g.blur,
                saturate: g.saturate,
                brightness: g.brightness,
                fill: t.try_spec(&g.fill)?,
                sheen: g.sheen,
                rim: t.try_spec(&g.rim)?,
                lens: g.lens,
                shadow_blur: g.shadow_blur,
                shadow_dy: g.shadow_dy,
                shadow: t.try_spec(&g.shadow)?,
            });
        }
        let amb = if dark { s.ambient_dark } else { s.ambient_light };
        for r in amb {
            let color = t.try_spec(&r.color)?;
            t.ambient.push(Radial { x: r.x, y: r.y, radius: r.radius, color });
        }
        Ok(t)
    }

    fn try_spec(&self, spec: &str) -> Result<Color, ThemeError> {
        let bad = || ThemeError::BadColor("spec".into(), spec.to_string());
        if let Some((name, a)) = spec.split_once('/') {
            let alpha: f32 = a.trim().parse().map_err(|_| bad())?;
            let base = match name.trim() {
                "white" => Color::rgb(0xffffff),
                "black" => Color::rgb(0x000000),
                n => *self.colors.get(n).ok_or_else(bad)?,
            };
            return Ok(base.faded(alpha));
        }
        if let Some(c) = Color::parse(spec) {
            return Ok(c);
        }
        self.colors.get(spec).copied().ok_or_else(bad)
    }

    /// A colour token or `token/alpha`. A missing token is a bug in the screen or the theme, so this stops loudly.
    pub fn color(&self, key: &str) -> Color {
        self.try_spec(key).unwrap_or_else(|_| panic!("theme `{}` has no colour `{key}`", self.name))
    }
    /// A number token (radius, text size).
    pub fn num(&self, key: &str) -> f32 {
        *self.numbers.get(key).unwrap_or_else(|| panic!("theme `{}` has no number `{key}`", self.name))
    }
    /// The glass material, if this style has one.
    pub fn glass(&self) -> Option<&Glass> {
        self.glass.as_ref()
    }
    /// Ambient background light (empty for flat styles).
    pub fn ambient(&self) -> &[Radial] {
        &self.ambient
    }
}
