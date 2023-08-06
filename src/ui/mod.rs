use crate::{
    app::{App, AppMode},
    lua::{get_prompt, run_ui_update},
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Spans, Text},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();

    for (_location, pane_ref) in app.panes.clone().hash_map.into_iter() {
        let mut pane = pane_ref.lock().unwrap();
        let mut prompt = get_prompt(app);

        let text = pane.content.clone();

        let mut block = Block::default().style(Style::default().bg(Color::Black));
        if pane.x > 0 {
            block = block.borders(Borders::LEFT);
        }
        if pane.y > 0 {
            block = block.borders(Borders::TOP);
        }

        let main = Paragraph::new(text.clone())
            .block(block)
            // messes up cursore position so i think i will have to handle wraping myself?
            // usefull for debuging though. lol
            // .wrap(Wrap { trim: false })
            .scroll(pane.scroll);

        f.render_widget(main, Rect::new(pane.x, pane.y, pane.size.0, pane.size.1));

        // auto_sagest(app, &mut pane);
        for (_, vtext) in &pane.vtext {
            f.render_widget(vtext.p.clone(), vtext.size)
        }
    }

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
            let pane_ref = app.panes.hash_map.get_mut(&app.active_pane).unwrap();
            let pane = pane_ref.lock().unwrap();

            if pane.scroll.0 == pane.max_scroll.0 {
                f.set_cursor(pane.x + pane.vc.0, pane.y + pane.vc.1);
            }
        }
        AppMode::Command => {}
    }

    let bar_text = run_ui_update(app, "render_status_bar");

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
