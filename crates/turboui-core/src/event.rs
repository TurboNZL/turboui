/// Something the user did, reported by a backend to the app. The app changes its own state and the screen is rebuilt.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// Sent regularly when the app asked for it (see the backend's `Options::tick_ms`), so it can look for outside news.
    Tick,
    /// A node with this id was clicked.
    Click(String),
    /// The pointer is now over the node with this id (`None`: over nothing clickable).
    Hover(Option<String>),
    /// The wheel moved over a scroll area. `dy` is in pixels, positive scrolls the content up (reads further down).
    /// `viewport` and `content` are the heights of the area and of what it holds, to clamp the offset.
    Scroll { id: String, dy: f32, viewport: f32, content: f32 },
    /// A key was pressed. `key` is a name such as "a", "enter", "backspace", "left"; `text` is the character it types, if any.
    Key { key: String, text: Option<String>, ctrl: bool, shift: bool, alt: bool },
}
