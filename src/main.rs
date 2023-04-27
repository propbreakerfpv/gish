use app::{run_app, App, reset_terminal};
use std::{error::Error, io, panic::set_hook};
use tui::{
    backend::CrosstermBackend,
    style::{Color, Style},
    text::Span,
    Terminal,
};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

mod app;
mod lua;
mod shell;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    // custom panic hook to reset terminal
    let original_hook = std::panic::take_hook();
    set_hook(Box::new(move |panic| {
        reset_terminal().unwrap();
        original_hook(panic)
    }));

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it

    let mut app: App = App::new();

    app.prompt.elements.push(Span::styled("{", Style::default().fg(Color::Rgb(255, 255, 0)),));
    app.prompt.elements.push(Span::styled("prop",Style::default().fg(Color::Rgb(255, 0, 0)),));
    app.prompt.elements.push(Span::styled("}",Style::default().fg(Color::Rgb(255, 255, 0)),));
    app.prompt.elements.push(Span::styled("{",Style::default().fg(Color::Rgb(255, 255, 0)),));
    app.prompt.elements.push(Span::styled("~/coding/tui_test/",Style::default().fg(Color::Rgb(255, 0, 0)),));
    app.prompt.elements.push(Span::styled("} ",Style::default().fg(Color::Rgb(255, 255, 0)),));

    app.setup_lua();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
        )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
