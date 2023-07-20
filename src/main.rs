// posix spec.
// https://pubs.opengroup.org/onlinepubs/9699919799/


use app::{run_app, App, reset_terminal, pane::{PaneLocation, Pane}};
use std::{error::Error, io, panic::set_hook, process};
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
        original_hook(panic);
        process::exit(1);
    }));

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app: App<'static> = App::new();
    let pane = app.panes.hash_map.get_mut(&app.active_pane).unwrap();
    let size = pane.lock().unwrap().size;
    pane.lock().unwrap().size = (size.0 / 2, size.1);

    // inserting dbg pane
    let dbg = Pane::debug();
    {
        let mut tmp_dbg = dbg.lock().unwrap();
        tmp_dbg.size = (size.0 / 2, size.1);
        tmp_dbg.x = size.0 / 2;
    }
    app.panes.hash_map.insert(PaneLocation::default().x(1), dbg);

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
