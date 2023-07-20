use std::{fs, env, sync::MutexGuard};

use tui::{widgets::Paragraph, text::{Spans, Span, Text}, style::{Style, Color}, layout::Rect};
use crate::shell;

use super::{VText, pane::Pane, App, };

pub fn on_comp(mut pane: MutexGuard<Pane>) -> String {
    let mut sagest = get_autocomp(&pane);
    if ! sagest.is_empty() {
        sagest = sagest[pane.cmd_input.len()..sagest.len()].to_string();
    }
    pane.cmd_input += &sagest;
    pane.println(sagest);
    String::new()
}

pub fn auto_sagest(app: &mut App, pane: &mut MutexGuard<Pane>) {
    if ! pane.is_active {
        return;
    }
    if ! pane.cmd_input.contains(' ') && pane.scroll.0 == pane.max_scroll.0 {
        if app.config.only_sagest_when_typing {
        }
        let mut sagest = get_autocomp(&pane);
        if ! sagest.is_empty() && pane.cmd_input.len() <= sagest.len() {
            sagest = sagest[pane.cmd_input.len()..sagest.len()].to_string();
        }

        let text = Spans::from(Span::styled(sagest.clone(), Style::default().fg(Color::Rgb(89, 89, 89))));
        let p = Paragraph::new(text);
        let size = Rect::new(pane.vc.0 + pane.x, pane.vc.1 + pane.y, sagest.len() as u16, 1);
        pane.vtext.insert(String::from("auto_sagest"), VText { p, size });
    } else if ! pane.running_cmd {
        let p = Paragraph::new(Text::default());
        let size = Rect::new(pane.vc.0, pane.vc.1, 0, 1);
        pane.vtext.insert(String::from("auto_sagest"), VText { p, size });
    }
}

fn get_autocomp(pane: &MutexGuard<Pane>) -> String {
    // todo make this work with lua, add diferent sources each with its own priority and stuff

    let mut cmds = get_cmds(pane.cmd_input.clone(), pane);
    for cmd in get_internal_cmds(&pane.cmd_input) {
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

fn rec_find_file_with_prefix(prefix: &String, parent: String) -> Vec<String> {
    if prefix.len() < parent.len() {
        return vec![parent];
    }
    let mut out = Vec::new();
    for file in match fs::read_dir(&parent) {
        Ok(v) => v,
        Err(_) => return vec![],
    } {
        let mut name = parent.clone();
        name.push('/');
        name.push_str(file.unwrap().file_name().to_str().unwrap());
        if name.len() >= prefix.len() {
            if &name[0..prefix.len()] == prefix.as_str() {
                rec_find_file_with_prefix(&prefix, name).iter().for_each(|x| out.push(x.clone()));
            }
        }
    }
    out
}

fn get_cmds(prefix: String, pane: &MutexGuard<Pane>) -> Vec<String> {
    let mut out = Vec::new();
    if prefix.starts_with("./") {
        let pwd = env::var("PWD").unwrap();
        let mut p = pwd.clone();
        p.push_str(&prefix[2..]);
        for file in fs::read_dir(&pwd).unwrap() {
            let mut name = pwd.clone();
            name.push_str(file.unwrap().file_name().to_str().unwrap());
            if p.len() >= name.len() {
                if &p[0..name.len()] == name.as_str() {
                    rec_find_file_with_prefix(&p, name).iter().for_each(|x| out.push(x.clone()));
                }
            } else if name.starts_with(&p) {
                out.push(name);
            }
        }
        out = out.iter().map(|x| x[pwd.len() - 2..x.len()].to_string()).collect();
    } else if prefix.starts_with('/') {
        for file in fs::read_dir("/").unwrap() {
            let mut name = String::from("/");
            name.push_str(file.unwrap().file_name().to_str().unwrap());
            if prefix.len() >= name.len() {
                if &prefix[0..name.len()] == name.as_str() {
                    rec_find_file_with_prefix(&prefix, name).iter().for_each(|x| out.push(x.clone()));
                }
            } else if name.starts_with(&prefix) {
                out.push(name);
            }
        }
    } else {
        for path in pane.cmds.clone() {
            if path.starts_with(&prefix) {
                out.push(path);
            }
        }
    }
    out
}
