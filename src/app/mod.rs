use std::{io, cell::RefCell, fs, error::Error, collections::{HashMap, HashSet}};

use crossterm::{event::{Event, self, KeyCode}, terminal::{disable_raw_mode, LeaveAlternateScreen}};
use hlua::{Lua, function1, LuaFunction, function0, LuaTable};
use tui::{backend::Backend, Terminal, text::{Span, Text, Spans}};

use crate::{ui::ui, shell::run_command, lua::setup_lua};

#[derive(Debug, Clone)]
pub struct Prompt<'a> {
    pub elements: Vec<Span<'a>>,
}

impl<'a> Prompt<'a> {
    fn new() -> Prompt<'a> {
        Prompt {
            elements: Vec::new(),
        }
    }
}



#[derive(Debug)]
pub struct App<'a> {
    pub search_input: String,
    pub cmd_input: String,
    pub content: Text<'a>,
    pub mode: AppMode,
    pub prompt: Prompt<'a>,
    pub status_bar: Text<'a>,
    pub scroll: (u16, u16),
    pub lua: RefCell<Lua<'a>>,
}

pub type LuaApp = String;

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        App {
            search_input: String::new(),
            cmd_input: String::new(),
            content: Text::from(""),
            mode: AppMode::Normal,
            prompt: Prompt::new(),
            status_bar: Text::from("status_bar"),
            scroll: (0, 0),
            lua: RefCell::new(Lua::new()),
        }
    }
    pub fn setup_lua(&self) {
        setup_lua(&self);
    }
}

/// Normal
/// Searching
/// Command
#[derive(Debug, Clone)]
pub enum AppMode {
    Normal,
    Searching,
    Command,
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn reset_terminal() -> Result<()> {
    disable_raw_mode()?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}


pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {


    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.mode.clone() {
                AppMode::Normal => match key.code {
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
                        {
                            let mut lua = app.lua.borrow_mut();
                            let of: Option<LuaFunction<_>> = lua.get("event_on_cmd_run");
                            match of {
                                Some(mut v) => {
                                    v.call::<()>().unwrap();
                                }
                                None => {}
                            };
                        }
                        run_command(app.cmd_input.clone(), &mut app);
                    }
                    KeyCode::Up => {
                        app.mode = AppMode::Command;
                    }
                    KeyCode::Down => {
                        app.mode = AppMode::Searching;
                    }
                    _ => {}
                },
                AppMode::Searching => match key.code {
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
                },
                AppMode::Command => match key.code {
                    KeyCode::Esc => {
                        app.mode = AppMode::Normal;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => {}
                },
            }
        }
    }
}
