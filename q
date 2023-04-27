use std::{error::Error, io::{self, Read}, process::{Command, Stdio}, cell::RefCell, fs };
use hlua::{Lua, function1 };
use hlua::LuaFunction;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Text, Spans},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame, Terminal,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};


#[derive(Debug, Clone)]
struct Prompt<'a> {
    elements: Vec<Span<'a>>,
}

impl<'a> Prompt<'a>{
    fn new() -> Prompt<'a> {
        Prompt {
            elements: Vec::new(),
        }
    }
}

#[derive(Debug)]
struct App<'a> {
    search_input: String,
    cmd_input: String,
    content: Text<'a>,
    mode: AppMode,
    prompt: Prompt<'a>,
    scroll: (u16, u16),
    lua: RefCell<Lua<'a>>,
}

// struct LuaApp {
//     search_input: String,
//     cmd_input: String,
//     content: String,
//     mode: String,
//     prompt: String,
//     scroll: (u16, u16),
// }

type LuaApp = (String, String, String, String, String, i32, i32);

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            search_input: String::new(),
            cmd_input: String::new(),
            content: Text::from(""),
            mode: AppMode::Normal,
            prompt: Prompt::new(),
            scroll: (0, 0),
            lua: RefCell::new(Lua::new())
        }
    }
    fn setup_lua(&self) {
        let mut lua = self.lua.borrow_mut();
        lua.set("print", function1(|s: String| {
            println!("{}", s);
        }));
        let src = fs::read_to_string("./lua/on_tick.lua").unwrap();
        lua.execute::<()>(&src).unwrap();
    }
}


#[derive(Debug, Clone)]
enum AppMode {
    Normal,
    Searching,
    Command
}


fn main() -> Result<(), Box<dyn Error>> {

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it


    let mut app: App = App::new();

    app.prompt.elements.push(Span::styled("{", Style::default().fg(Color::Rgb(255, 255, 0))));
    app.prompt.elements.push(Span::styled("prop", Style::default().fg(Color::Rgb(255, 0, 0))));
    app.prompt.elements.push(Span::styled("}", Style::default().fg(Color::Rgb(255, 255, 0))));
    app.prompt.elements.push(Span::styled("{", Style::default().fg(Color::Rgb(255, 255, 0))));
    app.prompt.elements.push(Span::styled("~/coding/tui_test/", Style::default().fg(Color::Rgb(255, 0, 0))));
    app.prompt.elements.push(Span::styled("} ", Style::default().fg(Color::Rgb(255, 255, 0))));

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.mode.clone() {
                AppMode::Normal => {
                    match key.code {
                        KeyCode::Char(c) => {
                            app.cmd_input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.cmd_input.pop();
                        }
                        KeyCode::Enter => {
                            let mut prompt = app.prompt.elements.clone();
                            prompt.push(Span::raw(app.cmd_input.clone()));
                            app.content.extend(Text::from(Spans::from(prompt)));
                            run_command(app.cmd_input.clone(), &mut app);
                        }
                        KeyCode::Up => {
                            app.mode = AppMode::Command;
                        }
                        KeyCode::Down => {
                            app.mode = AppMode::Searching;
                        }
                        _ => {}
                    }
                }
                AppMode::Searching => {
                    match key.code {
                        KeyCode::Char(c) => {
                            app.search_input.push(c);
                        }
                        KeyCode::Esc => {
                            app.mode = AppMode::Normal;
                        }
                        KeyCode::Backspace => {
                            app.search_input.pop();
                        }
                        _ => {}
                    }
                }
                AppMode::Command => {
                    match key.code {
                        KeyCode::Esc => {
                            app.mode = AppMode::Normal;
                        }
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let mut lua = app.lua.borrow_mut();
    let mut on_tick: LuaFunction<_> = lua.get("on_tick").unwrap();
    let stdout: LuaApp = on_tick.call().unwrap();
    app.content.extend(Text::from(Span::from(stdout.3)));
    // if stdout.len() > 0 {
    //     app.content.extend(Text::from(Span::from(stdout)));
    // }

    let size = f.size();


    let block = Block::default()
        // .title("Content")
        // .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    let style = Style::default();
    let mut text = app.content.clone();

    if app.content.height() as u16 > size.height - 2 {
        app.scroll = (app.content.height() as u16 - (size.height - 1), app.scroll.1);
    }

    let mut prompt = app.prompt.elements.clone();
    prompt.push(Span::raw(app.cmd_input.clone()));
    text.extend(Text::from(Spans::from(prompt)));


    // text.extend(Text::from(Spans::from(app.prompt.elements.clone())));
    // text.extend(Text::raw(app.cmd_input.clone() + "\n"));
    text.patch_style(style);

    let main = Paragraph::new(text.clone())
        .block(block)
        .scroll(app.scroll.clone());

    f.render_widget(main, size);

    if let AppMode::Searching = app.mode.clone() {
        let block = Block::default().title("Popup").borders(Borders::ALL);
        let search_bar = Paragraph::new(app.search_input.clone()).block(block);
        let area = search_rect(60, 6, size);
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(search_bar, area);
    }
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

fn run_command(cmd: String, app: &mut App) {
    if cmd.is_empty() {
        return;
    }
    app.cmd_input.clear();
    let path = cmd.split(" ").next().unwrap().to_string();
    let mut p = Command::new(&path);
    p.stdout(Stdio::piped());

    let mut iter = cmd.split(" ");
    iter.next();
    for arg in iter {
        p.arg(arg);
    }

    let p = match p.spawn() {
        Ok(v) => v,
        Err(_) => {
            let msg = format!("`{}` command not found\n", path);
            app.content.extend(Text::from(msg));
            return;
        }
    };

    let mut s = String::new();
    match p.stdout.unwrap().read_to_string(&mut s) {
        Ok(_) => app.content.extend(Text::from(s)),
        Err(_) => {
            let msg = format!("an unexpected error ocered while running `{}`\n", path);
            app.content.extend(Text::from(msg));
        }
    };
}

