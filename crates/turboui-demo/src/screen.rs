//! Sample TurboMC Server screens, written only against `turboui-core`.
//! The layout follows Prism Launcher's main window: a toolbar on top, an icon grid with group
//! headings, an action list down the right, and a status bar.
use turboui_core::{col, cube, icon, label, row, spacer, Align, Color, Node, Theme};

/// The screens' own colour names, mapped onto turbothemes' tokens.
fn tok(t: &Theme, name: &str) -> Color {
    match name {
        "window" => t.color("surface"),
        "view" => t.color("bg"),
        "border" => t.color("line"),
        "text" => t.color("fg"),
        "disabled" => t.color("muted/0.55"),
        "accent_text" => t.color("accent-fg"),
        "selection" => t.color("accent/0.3"),
        "ok" => t.color("good"),
        "err" => t.color("bad"),
        "console_bg" => t.color("bg").darken(0.3),
        "input_bg" => t.color("bg"),
        other => t.color(other),
    }
}

/// A surface: glass in a glass style, otherwise a flat fill.
fn surface(t: &Theme, n: Node, radius: f32) -> Node {
    match t.glass() {
        Some(g) => n.glass(g.clone()).radius(radius),
        None => n.bg(tok(t, "window")),
    }
}


pub struct Server {
    pub name: &'static str,
    pub software: &'static str,
    pub version: &'static str,
    pub running: bool,
}

/// Everything the screens read. The app owns it; backends only report clicks by id.
pub struct State {
    pub selected: Option<usize>,
    pub servers: Vec<Server>,
}

impl State {
    pub fn sample() -> State {
        State {
            selected: Some(0),
            servers: vec![
                Server { name: "Survival", software: "Paper", version: "1.21.4", running: true },
                Server { name: "Creative", software: "Vanilla", version: "1.21.4", running: false },
                Server { name: "Modded", software: "Fabric", version: "1.21.1", running: false },
                Server { name: "Minigames", software: "Purpur", version: "1.21.4", running: false },
                Server { name: "Friends SMP", software: "Paper", version: "1.20.6", running: false },
            ],
        }
    }
    /// Applies a click. Ids look like `srv:2`.
    pub fn click(&mut self, id: &str) {
        if let Some(n) = id.strip_prefix("srv:").and_then(|n| n.parse::<usize>().ok()) {
            if n < self.servers.len() {
                self.selected = Some(n);
            }
        }
    }
}

fn hline(t: &Theme) -> Node {
    col().h(1.0).bg(tok(t, "border"))
}
fn vline(t: &Theme) -> Node {
    col().w(1.0).bg(tok(t, "border"))
}

/// A toolbar button: icon, then text.
fn tool_button(t: &Theme, ico: &str, text: &str, enabled: bool) -> Node {
    let c = if enabled { tok(t, "text") } else { tok(t, "disabled") };
    row().gap(7.0).pad_xy(10.0, 6.0).align(Align::Center).radius(t.num("radius_ctl"))
        .child(icon(ico, 16.0, c))
        .child(label(text).size(t.num("text_md")).color(c))
}

fn top_toolbar(t: &Theme) -> Node {
    let n = row().pad(4.0).gap(2.0).align(Align::Center)
        .child(tool_button(t, "plus", "Add Server", true))
        .child(col().w(1.0).h(22.0).bg(tok(t, "border")))
        .child(tool_button(t, "folder", "Folders", true))
        .child(tool_button(t, "settings", "Settings", true))
        .child(tool_button(t, "help", "Help", true));
    if t.glass().is_some() {
        surface(t, n.pad_xy(10.0, 4.0), 9999.0)
    } else {
        n
    }
}

fn action_list(t: &Theme, has_selection: bool, running: bool) -> Node {
    let on = has_selection;
    let n = col().w(176.0).pad(6.0).gap(1.0)
        .child(tool_button(t, "play", "Start", on && !running))
        .child(tool_button(t, "kill", "Kill", on && running))
        .child(col().h(1.0).bg(tok(t, "border")))
        .child(tool_button(t, "edit", "Edit", on))
        .child(tool_button(t, "tag", "Change Group", on))
        .child(tool_button(t, "folder", "Folder", on))
        .child(tool_button(t, "export", "Export", on))
        .child(tool_button(t, "copy", "Copy", on))
        .child(tool_button(t, "delete", "Delete", on));
    if t.glass().is_some() {
        surface(t, n.pad(12.0), t.num("radius_bar"))
    } else {
        n
    }
}

fn tile(t: &Theme, i: usize, s: &Server, selected: bool) -> Node {
    let dot = if s.running { tok(t, "ok") } else { tok(t, "disabled") };
    let mut n = col().id(format!("srv:{i}")).w(116.0).h(104.0).pad(8.0).gap(6.0).align(Align::Center).radius(t.num("radius_card").min(16.0))
        .child(cube(56.0, tok(t, "accent")))
        .child(
            row().gap(5.0).align(Align::Center)
                .child(icon("dot", 8.0, dot))
                .child(label(s.name).size(t.num("text_md")).color(tok(t, "text"))),
        );
    if selected {
        n = n.bg(tok(t, "selection")).border(1.0, tok(t, "accent"));
    }
    n
}

fn group(t: &Theme, title: &str, state: &State) -> Node {
    col().gap(8.0)
        .child(
            row().gap(8.0).align(Align::Center)
                .child(label(title).size(t.num("text_md")).color(tok(t, "muted")))
                .child(hline(t).grow(1.0)),
        )
        .child(
            row().wrap().gap(6.0)
                .children(state.servers.iter().enumerate().map(|(i, s)| tile(t, i, s, state.selected == Some(i)))),
        )
}

/// The main window: the list of servers.
pub fn server_manager(t: &Theme, state: &State) -> Node {
    let sel = state.selected.and_then(|i| state.servers.get(i));
    let status = match sel {
        Some(s) => format!("{} {}  -  {}", s.software, s.version, if s.running { "running" } else { "stopped" }),
        None => format!("{} servers", state.servers.len()),
    };
    let status_bar = row().h(26.0).pad_xy(10.0, 0.0).align(Align::Center)
        .child(label(status).size(t.num("text_sm")).color(tok(t, "muted")))
        .child(spacer())
        .child(label("TurboMC Server").size(t.num("text_sm")).color(tok(t, "muted")));
    let actions = action_list(t, sel.is_some(), sel.map_or(false, |s| s.running));

    if t.glass().is_some() {
        // Liquid Glass: the content sits on ambient light; the toolbar and actions float above it as glass.
        let f = t.num("float");
        col().bg(tok(t, "view")).radials(t.ambient().to_vec()).pad(f).gap(f)
            .child(top_toolbar(t))
            .child(
                row().grow(1.0).gap(f)
                    .child(col().grow(1.0).pad(10.0).child(group(t, "Servers", state)))
                    .child(actions),
            )
            .child(status_bar)
    } else {
        col().bg(tok(t, "window"))
            .child(top_toolbar(t))
            .child(hline(t))
            .child(
                row().grow(1.0)
                    .child(col().grow(1.0).bg(tok(t, "view")).pad(10.0).child(group(t, "Servers", state)))
                    .child(vline(t))
                    .child(actions),
            )
            .child(hline(t))
            .child(status_bar)
    }
}

fn log_line(t: &Theme, time: &str, level: &str, msg: &str) -> Node {
    let c = match level {
        "WARN" => tok(t, "warn"),
        "ERROR" => tok(t, "err"),
        _ => tok(t, "text"),
    };
    row().gap(8.0)
        .child(label(time).mono().size(12.0).color(tok(t, "muted")))
        .child(label(format!("[{level}]")).mono().size(12.0).color(c))
        .child(label(msg).mono().size(12.0).color(c))
}

fn page_entry(t: &Theme, ico: &str, text: &str, selected: bool) -> Node {
    let mut n = row().gap(8.0).pad_xy(10.0, 7.0).align(Align::Center).radius(t.num("radius_ctl"))
        .child(icon(ico, 16.0, tok(t, "text")))
        .child(label(text).size(t.num("text_md")).color(tok(t, "text")));
    if selected {
        n = n.bg(tok(t, "selection"));
    }
    n
}

fn dialog_button(t: &Theme, text: &str, primary: bool) -> Node {
    let (bg, fg) = if primary { (tok(t, "accent"), tok(t, "accent_text")) } else { (tok(t, "raised"), tok(t, "text")) };
    row().pad_xy(16.0, 7.0).radius(t.num("radius_ctl")).bg(bg).child(label(text).size(t.num("text_md")).color(fg))
}

/// A server's own window: page list on the left, the console on the right.
pub fn server_window(t: &Theme) -> Node {
    let pages = col().w(190.0).pad(6.0).gap(1.0)
        .child(page_entry(t, "play", "Console", true))
        .child(page_entry(t, "settings", "Server version", false))
        .child(page_entry(t, "folder", "Plugins", false))
        .child(page_entry(t, "edit", "Notes", false))
        .child(page_entry(t, "folder", "Worlds", false))
        .child(page_entry(t, "edit", "Properties", false))
        .child(page_entry(t, "export", "Backups", false))
        .child(page_entry(t, "settings", "Settings", false));
    let console = col().grow(1.0).pad(10.0).gap(3.0).bg(tok(t, "console_bg")).border(1.0, tok(t, "border")).radius(t.num("radius_card").min(12.0))
        .child(log_line(t, "12:01:03", "INFO", "Starting minecraft server version 1.21.4"))
        .child(log_line(t, "12:01:04", "INFO", "Loading properties"))
        .child(log_line(t, "12:01:06", "WARN", "Can't keep up! Is the server overloaded?"))
        .child(log_line(t, "12:01:09", "INFO", "Done (4.812s)! For help, type \"help\""))
        .child(log_line(t, "12:03:41", "INFO", "Steve joined the game"))
        .child(log_line(t, "12:04:10", "ERROR", "Could not pass event PlayerMoveEvent"));
    let input = row().h(32.0).pad_xy(12.0, 0.0).align(Align::Center).bg(tok(t, "input_bg")).border(1.0, tok(t, "border")).radius(t.num("radius_ctl").min(16.0))
        .child(label("Type a command...").size(t.num("text_md")).color(tok(t, "muted")));
    let content = col().grow(1.0).pad(8.0).gap(8.0)
        .child(console)
        .child(input)
        .child(
            row().gap(8.0)
                .child(spacer())
                .child(dialog_button(t, "Stop", true))
                .child(dialog_button(t, "Kill", false))
                .child(dialog_button(t, "Close", false)),
        );
    if t.glass().is_some() {
        let f = t.num("float");
        col().bg(tok(t, "view")).radials(t.ambient().to_vec()).pad(f).gap(f)
            .child(row().grow(1.0).gap(f).child(surface(t, pages.pad(12.0), t.num("radius_bar"))).child(content))
    } else {
        col().bg(tok(t, "window")).child(row().grow(1.0).child(pages).child(vline(t)).child(content))
    }
}
