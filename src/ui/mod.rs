use hlua::{LuaFunction, PushGuard, LuaTable};
use tui::{backend::Backend, Frame, text::{Text, Span, Spans}, widgets::{Block, Paragraph, Borders, Clear}, style::{Style, Color}, layout::{Rect, Layout, Direction, Constraint}};
use crate::{app::{App, LuaApp, AppMode}, lua::call_internal};



pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {

        let content: LuaApp = match call_internal(&app, "on_tick") {
            Some(v) => {
                match v {
                    Ok(v) => v,
                    Err(e) => e.to_string(),
                }
            }
            None => String::new(),
        };

        if content.len() > 0 {
            app.content
                .extend(Text::from(Span::from(String::from(content))));
        }

    let size = f.size();

    let block = Block::default()
        // .title("Content")
        // .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    let style = Style::default();
    let mut text = app.content.clone();

    if app.content.height() as u16 > size.height - 2 {
        app.scroll = (
            app.content.height() as u16 - (size.height - 2),
            app.scroll.1,
        );
    }

    let mut prompt = {
        let mut lua = app.lua.borrow_mut();
        let mut internal: LuaTable<_> = lua.get("_internal").unwrap();
        let run_event: Option<LuaFunction<_>> = internal.get("run_event");

        match run_event {
            Some(mut v) => {
                let mut ret = Vec::new();//vec![Span::styled("hello", Style::default().fg(Color::Rgb(255, 255, 0)),)];
                let mut table: LuaTable<_> = v.call_with_args("prompt").unwrap();
                for (_, v) in table.iter::<i32, String>().filter_map(|e| e) {
                    let ground: u8 = v[0..1].parse().unwrap();
                    let r = u8::from_str_radix(&v[1..=2], 16).unwrap();
                    let g = u8::from_str_radix(&v[3..=4], 16).unwrap();
                    let b = u8::from_str_radix(&v[5..=6], 16).unwrap();
                    if ground == 3 {
                        let br = u8::from_str_radix(&v[6..8], 16).unwrap();
                        let bg = u8::from_str_radix(&v[8..10], 16).unwrap();
                        let bb = u8::from_str_radix(&v[10..12], 16).unwrap();
                        let value = v[13..=v.len() - 1].to_string();
                        ret.push(Span::styled(value, Style::default().bg(Color::Rgb(r, g, b)).fg(Color::Rgb(br, bg, bb))))
                    } else if ground == 1 {
                        let value = v[7..=v.len() - 1].to_string();
                        ret.push(Span::styled(value, Style::default().bg(Color::Rgb(r, g, b))))
                    } else if ground == 0 {
                        let value = v[7..=v.len() - 1].to_string();
                        ret.push(Span::styled(value, Style::default().fg(Color::Rgb(r, g, b))))
                    }
                }
                ret
            }
            None => {vec![Span::from("error in lua")]}
        }
    };

    prompt.push(Span::from(app.cmd_input.clone()));
    // let mut prompt = vec![Span::from(prompt)];
    // prompt (Span::raw(app.cmd_input.clone()));
    text.extend(Text::from(Spans::from(prompt)));

    // text.extend(Text::from(Spans::from(app.prompt.elements.clone())));
    // text.extend(Text::raw(app.cmd_input.clone() + "\n"));
    text.patch_style(style);

    let main = Paragraph::new(text.clone())
        .block(block)
        .scroll(app.scroll.clone());

    f.render_widget(main, size);

    if let AppMode::Searching = app.mode.clone() {
        let block = Block::default().title("Search").borders(Borders::ALL);
        let search_bar = Paragraph::new(app.search_input.clone()).block(block);
        let area = search_rect(60, 6, size);
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(search_bar, area);
    }


    let bar_text = match call_internal(&app, "render_status_bar") {
        Some(v) => {
            match v {
                Ok(v) => v,
                Err(e) => String::from(e.to_string())
            }
        }
        None => String::from("hello from status bar none")
    };

    let status_bar = Paragraph::new(Text::from(bar_text));
    let area = Rect::new(0, size.height - 1, size.width, 1);
    f.render_widget(Clear, area); //this clears out the background
    f.render_widget(status_bar, area);
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn search_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 8),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 8),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
