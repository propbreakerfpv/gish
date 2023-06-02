use tui::{widgets::Paragraph, text::{Spans, Span, Text}, style::{Style, Color}, layout::Rect};


use crate::shell;

use super::{App, VText};

pub fn on_comp(app: &mut App) -> String {
    let mut sagest = get_autocomp(app);
    if ! sagest.is_empty() {
        sagest = sagest[app.cmd_input.len()..sagest.len()].to_string();
    }
    app.cmd_input += &sagest;
    app.println(sagest);
    String::new()
}

pub fn auto_sagest(app: &mut App) {
    if ! app.cmd_input.contains(' ') {
        let mut sagest = get_autocomp(app);
        if ! sagest.is_empty() && app.cmd_input.len() <= sagest.len() {
            sagest = sagest[app.cmd_input.len()..sagest.len()].to_string();
        }

        let text = Spans::from(Span::styled(sagest.clone(), Style::default().fg(Color::Rgb(89, 89, 89))));
        let p = Paragraph::new(text);
        let size = Rect::new(app.vc.0, app.vc.1, sagest.len() as u16, 1);
        app.vtext.insert(String::from("auto_sagest"), VText { p, size });
    } else {
        let p = Paragraph::new(Text::default());
        let size = Rect::new(app.vc.0, app.vc.1, 0, 1);
        app.vtext.insert(String::from("auto_sagest"), VText { p, size });
    }
}

fn get_autocomp(app: &mut App) -> String {
    // todo make this work with lua, add diferent sources each with its own priority and stuff

    let mut cmds = get_cmds(app.cmd_input.clone(), app);
    for cmd in get_internal_cmds(&app.cmd_input) {
        cmds.push(cmd);
    }
    let mut sagest = String::new();
    if cmds.len() > 0 {
        sagest = cmds[0].clone();
    }
    sagest
}
fn get_internal_cmds(prefix: &String) -> Vec<String> {
    shell::get_internal_cmds().into_iter().filter(|x| x.starts_with(prefix)).collect()
}

fn get_cmds(prefix: String, app: &mut App) -> Vec<String> {
    if prefix.starts_with("./") || prefix.starts_with('/') {
        return Vec::new()
    }
    let mut out = Vec::new();
    for path in app.cmds.clone() {
        if path.starts_with(&prefix) {
            out.push(path);
        }
    }
    out
}
