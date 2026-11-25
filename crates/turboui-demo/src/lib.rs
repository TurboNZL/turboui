//! Sample screens used to check TurboUI: rendered in tests and by the demo binary.
pub mod screen;
use turboui_core::Theme;

/// A built-in look, by palette and style name, e.g. `theme("macos-dark", "liquid")`.
pub fn theme(palette: &str, style: &str) -> Theme {
    turboui_core::builtin::theme(palette, style).unwrap_or_else(|| panic!("no built-in look `{palette}/{style}`"))
}

/// Parses `palette/style` (for command lines).
pub fn theme_from_arg(arg: &str) -> Theme {
    turboui_core::builtin::theme_from_arg(arg).unwrap_or_else(|| panic!("no built-in look `{arg}`"))
}
