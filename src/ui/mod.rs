use tui::{backend::Backend, Frame, text::{Text, Span, Spans}, widgets::{Block, Paragraph, Borders, Clear}, style::{Style, Color}, layout::{Rect, Layout, Direction, Constraint}};
use crate::{app::{App, LuaApp, AppMode }, lua::{call_internal, run_ui_update, get_prompt}};



pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {

    // let content: LuaApp = match call_internal(app, "on_tick") {
    //     Some(v) => {
    //         match v {
    //             Ok(v) => v,
    //             Err(e) => e.to_string(),
    //         }
    //     }
    //     None => String::new(),
    // };
    //
    // if !content.is_empty() {
    //     app.println(content);
    // }

    let size = f.size();

    let block = Block::default().style(Style::default().bg(Color::Black));


    if app.content.height() as u16 > size.height - 2 {
        app.scroll = (
            app.content.height() as u16 - (size.height - 2),
            app.scroll.1,
            );
    }

    let mut prompt = get_prompt(app);
    // panic!("{}", prompt);
    app.prompt = prompt.clone();

    prompt.push_str(app.cmd_input.clone().as_str());
    if let Some(char) = app.cmd_input.chars().last() {
        if app.prompt_update {
            app.println(char);
            app.prompt_update = false;
        }
    }

    let text = app.content.clone();

    let main = Paragraph::new(text.clone())
        .block(block)
        // messes up cursore position so i think i will have to handle wraping myself?
        // usefull for debuging though. lol
        // .wrap(Wrap { trim: false })
        .scroll(app.scroll);
        

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
            app.vc = (area.x + app.search_input.len() as u16 + 1, area.y + 1)
        }
        AppMode::Normal => {
            // let cursor_pos_x = (app.prompt_len + app.cmd_input.len()) as u16;
            // let cursor_pos_y = app.content.height() as u16 - app.scroll.0;

            // f.set_cursor(cursor_pos_x, cursor_pos_y);
            f.set_cursor(app.vc.0, app.vc.1);
            // app.vc = (cursor_pos_x, cursor_pos_y);
        }
        AppMode::Command => {}
    }

    let bar_text = run_ui_update(app, "render_status_bar") ;

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
