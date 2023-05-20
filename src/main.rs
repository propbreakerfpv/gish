

// posix spec.
// https://pubs.opengroup.org/onlinepubs/9699919799/


use app::{run_app, App, reset_terminal};
use shell::run_command;
use std::{error::Error, io, panic::set_hook};
use tui::{
    backend::CrosstermBackend,
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
mod ansi;

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

    app.setup_lua();

    run_command("refresh".to_string(), &mut app);

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
