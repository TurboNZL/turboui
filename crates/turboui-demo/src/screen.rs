//! A sample TurboMC Server screen, written only against `turboui-core`.
use turboui_core::{col, label, row, spacer, Align, Color, Node, Theme};

struct Srv(&'static str, &'static str, &'static str, bool);

fn button(t: &Theme, text: &str, fill: Color, fg: Color) -> Node {
    row().pad_xy(14.0, 8.0).radius(t.num("radius_sm")).bg(fill).child(label(text).bold().size(t.num("text_sm")).color(fg))
}

fn server_row(t: &Theme, s: &Srv, selected: bool) -> Node {
    let dot = if s.3 { t.color("ok") } else { t.color("muted") };
    let mut r = row().gap(10.0).pad(10.0).radius(t.num("radius_md")).align(Align::Center)
        .child(col().w(10.0).h(10.0).radius(5.0).bg(dot))
        .child(
            col().gap(2.0)
                .child(label(s.0).bold().size(t.num("text_md")).color(t.color("text")))
                .child(label(format!("{} {}", s.1, s.2)).size(t.num("text_sm")).color(t.color("muted"))),
        )
        .child(spacer())
        .child(label(if s.3 { "Running" } else { "Stopped" }).size(11.0).color(dot));
    if selected {
        r = r.bg(t.color("panel_hi")).border(1.0, t.color("border"));
    }
    r
}

fn log_line(t: &Theme, time: &str, level: &str, msg: &str) -> Node {
    let c = match level {
        "WARN" => t.color("warn"),
        "ERROR" => t.color("err"),
        _ => t.color("text"),
    };
    row().gap(8.0)
        .child(label(time).mono().size(12.0).color(t.color("muted")))
        .child(label(format!("[{level}]")).mono().size(12.0).color(c))
        .child(label(msg).mono().size(12.0).color(c))
}

/// The server-manager main window.
pub fn server_manager(t: &Theme) -> Node {
    let servers = [
        Srv("Survival", "Paper", "1.21.4", true),
        Srv("Creative", "Vanilla", "1.21.4", false),
        Srv("Modded", "Fabric", "1.21.1", false),
        Srv("Minigames", "Purpur", "1.21.4", false),
    ];
    let top = row().h(48.0).pad_xy(16.0, 0.0).gap(10.0).align(Align::Center).bg(t.color("panel"))
        .child(col().w(22.0).h(22.0).radius(5.0).bg(t.color("accent")))
        .child(label("TurboMC Server").bold().size(16.0).color(t.color("text")))
        .child(spacer())
        .child(label("Settings").size(t.num("text_md")).color(t.color("muted")));

    let sidebar = col().w(280.0).pad(t.num("space_md")).gap(t.num("space_sm")).bg(t.color("panel"))
        .child(label("SERVERS").bold().size(11.0).color(t.color("muted")))
        .children(servers.iter().enumerate().map(|(i, s)| server_row(t, s, i == 0)));

    let console = col().grow(1.0).pad(t.num("space_md")).gap(3.0).radius(t.num("radius_md")).bg(t.color("console_bg"))
        .child(log_line(t, "12:01:03", "INFO", "Starting minecraft server version 1.21.4"))
        .child(log_line(t, "12:01:04", "INFO", "Loading properties"))
        .child(log_line(t, "12:01:06", "WARN", "Can't keep up! Is the server overloaded?"))
        .child(log_line(t, "12:01:09", "INFO", "Done (4.812s)! For help, type \"help\""))
        .child(log_line(t, "12:03:41", "INFO", "Steve joined the game"))
        .child(log_line(t, "12:04:10", "ERROR", "Could not pass event PlayerMoveEvent"));

    let input = row().h(36.0).pad_xy(12.0, 0.0).align(Align::Center).radius(t.num("radius_sm")).bg(t.color("input_bg"))
        .border(1.0, t.color("border"))
        .child(label("Type a command...").size(t.num("text_md")).color(t.color("muted")));

    let main = col().grow(1.0).pad(t.num("space_lg")).gap(t.num("space_md"))
        .child(
            row().align(Align::Center).gap(8.0)
                .child(label("Survival").bold().size(t.num("text_lg")).color(t.color("text")))
                .child(label("Paper 1.21.4").size(t.num("text_md")).color(t.color("muted")))
                .child(spacer())
                .child(button(t, "Stop", t.color("accent"), t.color("accent_text")))
                .child(button(t, "Kill", t.color("panel_hi"), t.color("err"))),
        )
        .child(console)
        .child(input);

    col().bg(t.color("bg")).child(top).child(row().grow(1.0).child(sidebar).child(main))
}
