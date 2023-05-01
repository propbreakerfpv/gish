use std::{io, cell::RefCell, error::Error };

use crossterm::{event::{Event, self, KeyCode}, terminal::{disable_raw_mode, LeaveAlternateScreen}};
use hlua::{Lua, LuaFunction };
use tui::{backend::Backend, Terminal, text::{Span, Text, Spans}};

use crate::{ui::ui, shell::run_command, lua::setup_lua};





#[derive(Debug)]
pub struct App<'a> {
    pub search_input: String,
    pub cmd_input: String,
    pub content: Text<'a>,
    pub mode: AppMode,
    pub prompt: Vec<Span<'a>>,
    pub status_bar: Text<'a>,
    pub cmd_history: Vec<String>,
    pub cmd_history_idx: usize,
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
            prompt: Vec::new(),
            status_bar: Text::from("status_bar"),
            cmd_history: Vec::new(),
            cmd_history_idx: 0,
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
                        app.cmd_history_idx = 0;
                        app.cmd_input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.cmd_input.pop();
                    }
                    KeyCode::Enter => {
                        if app.cmd_history_idx > 0 {
                            // todo this should not be get_mut but the barrow checker is playing
                            // games with me so thats what it is for now.
                            let cmd_history_len = app.cmd_history.clone().len();
                            let a = app.cmd_history.get_mut(cmd_history_len - app.cmd_history_idx);
                            match a {
                                Some(v) => app.cmd_input = v.clone(),
                                None => {}
                            };
                        }
                        let mut prompt = app.prompt.clone();
                        if app.cmd_history.len() > 0 {
                            if let Some(v) = app.cmd_history.get(app.cmd_history.len() - 1) {
                                if *v != app.cmd_input {
                                    app.cmd_history.push(app.cmd_input.clone());
                                }
                            }
                        } else {
                            app.cmd_history.push(app.cmd_input.clone());
                        }
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
                    KeyCode::End => {
                        app.mode = AppMode::Command;
                    }
                    KeyCode::Home => {
                        app.mode = AppMode::Searching;
                    }
                    KeyCode::Up => {
                        if app.cmd_history_idx < app.cmd_history.len() {
                            app.cmd_history_idx += 1;
                            // todo this should not be get_mut but the barrow checker is playing
                            // games with me so thats what it is for now.
                            let cmd_history_len = app.cmd_history.clone().len();
                            let a = app.cmd_history.get_mut(cmd_history_len - app.cmd_history_idx);
                            match a {
                                Some(v) => app.cmd_input = v.clone(),
                                None => {}
                            };
                        }
                    }
                    KeyCode::Down => {
                        if app.cmd_history_idx > 0 {
                            app.cmd_history_idx -= 1;
                            if app.cmd_history_idx == 0 {
                                app.cmd_input = String::new();
                            } else {
                                // todo this should not be get_mut but the barrow checker is playing
                                // games with me so thats what it is for now.
                                let cmd_history_len = app.cmd_history.clone().len();
                                let a = app.cmd_history.get_mut(cmd_history_len - app.cmd_history_idx);
                                match a {
                                    Some(v) => app.cmd_input = v.clone(),
                                    None => {}
                                };
                            }
                        }
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
