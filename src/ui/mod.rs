use tui::{backend::Backend, Frame, text::{Text, Span, Spans}, widgets::{Block, Paragraph, Borders, Clear}, style::{Style, Color}, layout::{Rect, Layout, Direction, Constraint}};
use crate::{app::{App, LuaApp, AppMode }, lua::{call_internal, run_ui_update}};



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
        .style(Style::default().bg(Color::Black));

    let style = Style::default();
    let mut text = app.content.clone();

    if app.content.height() as u16 > size.height - 2 {
        app.scroll = (
            app.content.height() as u16 - (size.height - 2),
            app.scroll.1,
            );
    }

    let mut prompt = run_ui_update(&app, "prompt");
    let prompt_len = prompt.iter().fold(0, |acc, x| acc + x.width());
    app.prompt = prompt.clone();

    prompt.push(Span::from(app.cmd_input.clone()));
    text.extend(Text::from(Spans::from(prompt)));

    text.patch_style(style);

    let main = Paragraph::new(text.clone())
        .block(block)
        .scroll(app.scroll.clone());

    f.render_widget(main, size);

    // if let AppMode::Searching = app.mode.clone() {
    match app.mode {
        AppMode::Searching => {
            let block = Block::default().title("Search").borders(Borders::ALL);
            let search_bar = Paragraph::new(app.search_input.clone()).block(block);
            let area = search_rect(60, 6, size);
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(search_bar, area);
            f.set_cursor(area.x + app.search_input.len() as u16 + 1, area.y + 1);
        }
        AppMode::Normal => {

            f.set_cursor(
                (prompt_len + app.cmd_input.len()) as u16,
                app.content.height() as u16 - app.scroll.0/*  * (size.height / 2) */
                );
        }
        AppMode::Command => {}
    }

    let bar_text = run_ui_update(&app, "render_status_bar") ;

    let status_bar = Paragraph::new(Text::from(Spans::from(bar_text)));
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
