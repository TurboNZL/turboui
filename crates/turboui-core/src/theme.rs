use crate::style::Color;
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
            ThemeError::BadColor(k, v) => write!(f, "theme colour `{k}` is not #rrggbb or #rrggbbaa: `{v}`"),
        }
    }
}
impl std::error::Error for ThemeError {}

#[derive(Deserialize)]
struct Raw {
    name: String,
    colors: BTreeMap<String, String>,
    #[serde(default)]
    numbers: BTreeMap<String, f32>,
}

/// A named set of design tokens, loaded from a TOML file. The app asks for tokens by name
/// (`theme.color("accent")`) instead of writing colours into screens.
#[derive(Clone, Debug)]
pub struct Theme {
    pub name: String,
    colors: BTreeMap<String, Color>,
    numbers: BTreeMap<String, f32>,
}

impl Theme {
    pub fn from_toml(src: &str) -> Result<Theme, ThemeError> {
        let raw: Raw = toml::from_str(src).map_err(|e| ThemeError::Parse(e.to_string()))?;
        let mut colors = BTreeMap::new();
        for (k, v) in raw.colors {
            let c = Color::parse(&v).ok_or_else(|| ThemeError::BadColor(k.clone(), v.clone()))?;
            colors.insert(k, c);
        }
        Ok(Theme { name: raw.name, colors, numbers: raw.numbers })
    }
    /// A colour token. A missing token is a bug in the screen or the theme, so this stops loudly.
    pub fn color(&self, key: &str) -> Color {
        *self.colors.get(key).unwrap_or_else(|| panic!("theme `{}` has no colour token `{key}`", self.name))
    }
    /// A number token (spacing, radius, font size).
    pub fn num(&self, key: &str) -> f32 {
        *self.numbers.get(key).unwrap_or_else(|| panic!("theme `{}` has no number token `{key}`", self.name))
    }
}
