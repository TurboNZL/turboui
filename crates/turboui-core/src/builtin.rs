//! Themes that ship with TurboUI, so apps do not have to carry the files.
use crate::theme::Theme;

const PALETTES: &[(&str, &str)] = &[
    ("macos-dark", include_str!("../../../themes/palettes/macos-dark.toml")),
    ("macos", include_str!("../../../themes/palettes/macos.toml")),
    ("nord", include_str!("../../../themes/palettes/nord.toml")),
    ("dracula", include_str!("../../../themes/palettes/dracula.toml")),
];
const STYLES: &[(&str, &str)] = &[
    ("flat", include_str!("../../../themes/styles/flat.toml")),
    ("liquid", include_str!("../../../themes/styles/liquid.toml")),
];

/// Names of the built-in palettes and styles.
pub fn palette_names() -> Vec<&'static str> {
    PALETTES.iter().map(|(n, _)| *n).collect()
}
pub fn style_names() -> Vec<&'static str> {
    STYLES.iter().map(|(n, _)| *n).collect()
}

/// A built-in look by palette and style name, e.g. `theme("macos-dark", "liquid")`. `None` for an unknown name.
pub fn theme(palette: &str, style: &str) -> Option<Theme> {
    let p = PALETTES.iter().find(|(n, _)| *n == palette)?.1;
    let s = STYLES.iter().find(|(n, _)| *n == style)?.1;
    Theme::from_parts(p, s).ok()
}

/// Parses `palette/style` (a style may be left out and defaults to `flat`).
pub fn theme_from_arg(arg: &str) -> Option<Theme> {
    let (p, s) = arg.split_once('/').unwrap_or((arg, "flat"));
    theme(p, s)
}
