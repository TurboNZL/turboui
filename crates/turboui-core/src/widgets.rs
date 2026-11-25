//! Ready-made controls built from the basic blocks, drawn with the current theme. They have no behaviour of their own:
//! each takes the state to show (hovered, focused, checked ...) and an id, and the app reacts to `Event`s.
use crate::node::{col, row, spacer, text as label, Node};
use crate::style::{Align, Justify};
use crate::theme::Theme;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ButtonKind {
    Primary,
    Normal,
    Danger,
}

/// A surface in the theme's material: glass in a glass style, otherwise a flat fill.
pub fn surface(t: &Theme, n: Node, radius: f32) -> Node {
    match t.glass() {
        Some(g) => n.glass(g.clone()).radius(radius),
        None => n.bg(t.role("window")).radius(radius),
    }
}

pub fn button(t: &Theme, id: &str, text: &str, kind: ButtonKind, hovered: bool, enabled: bool) -> Node {
    let (mut bg, mut fg) = match kind {
        ButtonKind::Primary => (t.role("accent"), t.role("accent_text")),
        ButtonKind::Normal => (t.role("raised"), t.role("text")),
        ButtonKind::Danger => (t.role("raised"), t.role("err")),
    };
    if hovered && enabled {
        bg = bg.lighten(0.12);
    }
    if !enabled {
        bg = bg.faded(0.5);
        fg = t.role("disabled");
    }
    let n = row().pad_xy(16.0, 7.0).align(Align::Center).justify(Justify::Center).radius(t.num("radius_ctl")).bg(bg)
        .child(label(text).size(t.num("text_md")).color(fg));
    if enabled {
        n.id(id)
    } else {
        n
    }
}

/// A one-line text field. `focused` shows a caret after the text.
pub fn text_input(t: &Theme, id: &str, value: &str, placeholder: &str, focused: bool) -> Node {
    let shown = if value.is_empty() {
        label(placeholder).size(t.num("text_md")).color(t.role("muted"))
    } else {
        label(value).size(t.num("text_md")).color(t.role("text"))
    };
    let mut r = row().id(id).h(32.0).pad_xy(12.0, 0.0).gap(1.0).align(Align::Center).clip().bg(t.role("input_bg"))
        .radius(t.num("radius_ctl").min(16.0))
        .border(1.0, if focused { t.role("accent") } else { t.role("border") });
    if focused && !value.is_empty() {
        r = r.child(shown).child(col().w(1.0).h(16.0).bg(t.role("accent")));
    } else if focused {
        r = r.child(col().w(1.0).h(16.0).bg(t.role("accent"))).child(shown);
    } else {
        r = r.child(shown);
    }
    r
}

pub fn checkbox(t: &Theme, id: &str, checked: bool, text: &str) -> Node {
    let boxed = if checked {
        col().w(16.0).h(16.0).radius(4.0).bg(t.role("accent")).align(Align::Center).justify(Justify::Center)
            .child(crate::icons::icon("check", 11.0, t.role("accent_text")))
    } else {
        col().w(16.0).h(16.0).radius(4.0).border(1.0, t.role("muted")).bg(t.role("input_bg"))
    };
    row().id(id).gap(8.0).align(Align::Center).child(boxed).child(label(text).size(t.num("text_md")).color(t.role("text")))
}

pub fn switch(t: &Theme, id: &str, on: bool) -> Node {
    let knob = col().w(18.0).h(18.0).radius(9.0).bg(crate::style::Color::rgb(0xffffff));
    let n = row().id(id).w(38.0).h(22.0).pad(2.0).radius(11.0).align(Align::Center);
    if on {
        n.bg(t.role("accent")).justify(Justify::End).child(knob)
    } else {
        n.bg(t.role("raised")).justify(Justify::Start).child(knob)
    }
}

/// A progress bar of `fraction` (0 to 1).
pub fn progress(t: &Theme, fraction: f32, width: f32) -> Node {
    let f = fraction.clamp(0.0, 1.0);
    row().w(width).h(8.0).radius(4.0).bg(t.role("raised")).clip().child(col().w((width * f).max(0.0)).h(8.0).bg(t.role("accent")))
}

/// A modal dialog over a dimmed scrim, centred in its parent. Put the result as the LAST child of a box that fills the window.
pub fn dialog(t: &Theme, title: &str, body: Node, buttons: Vec<Node>, width: f32) -> Node {
    let card = surface(
        t,
        col().w(width).pad(18.0).gap(14.0)
            .child(label(title).bold().size(t.num("text_lg")).color(t.role("text")))
            .child(body)
            .child(row().gap(8.0).child(spacer()).children(buttons)),
        t.num("radius_card").min(20.0),
    );
    col().overlay().id("scrim").bg(crate::style::Color::rgba(0, 0, 0, 120)).align(Align::Center).justify(Justify::Center).child(card)
}
