//! TurboUI core: describe a screen once, get the same laid-out scene everywhere.
//!
//! The flow is `Node` tree (what the app writes) -> `layout()` -> `Scene` (a flat list of
//! rectangles and text with final positions). A renderer backend only has to draw a `Scene`;
//! it never decides layout, so every platform shows the same thing.

mod ambient;
mod icons;
mod layout;
mod node;
mod style;
mod text;
mod theme;

pub use icons::{cube, icon};
pub use layout::{layout, DrawItem, Hit, Scene};
pub use node::{col, row, shape, spacer, text as label, Face, Kind, Node};
pub use ambient::paint_radials;
pub use style::{Align, Color, Dir, Glass, Justify, Radial, Size, Style};
pub use text::{Font, Measurer, FONT_MONO, FONT_REGULAR, FONT_BOLD};
pub use theme::{Theme, ThemeError};
