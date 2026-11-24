//! Sample screens used to check TurboUI: rendered in tests and by the demo binary.
pub mod screen;
use turboui_core::Theme;

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

/// A built-in look, by palette and style name, e.g. `theme("macos-dark", "liquid")`.
pub fn theme(palette: &str, style: &str) -> Theme {
    let p = PALETTES.iter().find(|(n, _)| *n == palette).unwrap_or_else(|| panic!("no palette `{palette}`")).1;
    let s = STYLES.iter().find(|(n, _)| *n == style).unwrap_or_else(|| panic!("no style `{style}`")).1;
    Theme::from_parts(p, s).expect("built-in theme parses")
}

/// Parses `palette/style` (for command lines).
pub fn theme_from_arg(arg: &str) -> Theme {
    let (p, s) = arg.split_once('/').unwrap_or((arg, "flat"));
    theme(p, s)
}
