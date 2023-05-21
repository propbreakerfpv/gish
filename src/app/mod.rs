use std::{cell::RefCell, collections::HashMap, error::Error, fs, io};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use hlua::Lua;
use tui::{
    backend::Backend,
    text::{Span, Spans, Text},
    Terminal,
};

use crate::{ansi::{test, Pos}, lua::{setup_lua, self}, shell::run_command, ui::ui};

use self::auto_comp::on_comp;

pub mod auto_comp;

fn vec_empty_char(width: u16, hight: u16) -> Vec<Vec<Pos>> {
    let mut out = Vec::new();
    for _ in 1..=hight {
        out.push(vec![Pos::Empty; width as usize]);
    }
    out
}

#[derive(Debug)]
pub struct App<'a> {
    pub search_input: String,
    pub cmd_input: String,
    pub content: Text<'a>,
    /// x, y: horazontal, vertical
    pub vc: (u16, u16),
    pub vstdout: Vec<Vec<Pos>>,
    pub mode: AppMode,
    pub prompt: String,
    pub prompt_update: bool,
    pub status_bar: Text<'a>,
    pub cmd_history: Vec<String>,
    pub cmd_history_idx: usize,
    pub alais: HashMap<String, String>,
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
            vc: (0, 0),
            vstdout: vec_empty_char(200, 50),
            mode: AppMode::Normal,
            prompt: String::new(),
            prompt_update: false,
            status_bar: Text::from("status_bar"),
            cmd_history: Vec::new(),
            cmd_history_idx: 0,
            alais: HashMap::new(),
            scroll: (0, 0),
            lua: RefCell::new(Lua::new()),
        }
    }
    pub fn setup_lua(&self) {
        setup_lua(self);
    }
    // pub fn add_content(&mut self, content: String) {
    //
    //     Text::from(content);
    // }
    pub fn println<T: ToString>(&mut self, msg: T) {
        // for char in msg.to_string().chars() {
        //     self.write_char(Char::Char(char));
        // }
        // self.vc.1 += 1;
        self.content = test(self, msg.to_string());
        // self.content.extend(Text::from(msg.to_string()));
    }
    pub fn print_span(&mut self, msg: Vec<Span<'a>>) {
        self.content.extend(Text::from(Spans::from(msg)));
    }

    // fn write_char(&mut self, char: Char) {
    //     self.vstdout[self.vc.1 as usize][self.vc.0 as usize] = char;
    //     self.vc.0 += 1;
    // }
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

    let startup = fs::read_to_string("./config/startup.sh").unwrap();
    for line in startup.lines() {
        run_command(line.trim().to_string(), &mut app)
    }

    app.prompt = lua::get_prompt(&mut app);

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.mode.clone() {
                AppMode::Normal => match key.code {
                    // todo ctr-c kill cmd
                    // KeyCode::Char('c') => {
                    //
                    //     key.modifiers.contains(other);
                    //     app.println("kill prog");
                    // }
                    KeyCode::Char(c) => {
                        app.cmd_history_idx = 0;
                        app.cmd_input.push(c);
                        app.prompt_update = true;
                        // app.write_char(Char::Char(c));
                    }
                    KeyCode::Backspace => {
                        if app.cmd_input.pop().is_some() {
                            app.println("\x1b[1D \x1b[1D");
                        }
                    }
                    KeyCode::Tab => {
                        on_comp(&app);
                    }
                    KeyCode::Enter => {
                        if app.cmd_history_idx > 0 {
                            // todo this should not be get_mut but the barrow checker is playing
                            // games with me so thats what it is for now.
                            let cmd_history_len = app.cmd_history.clone().len();
                            let a = app
                                .cmd_history
                                .get_mut(cmd_history_len - app.cmd_history_idx);
                            if let Some(v) = a {
                                app.cmd_input = v.clone();
                            }
                        }
                        let mut prompt = app.prompt.clone();
                        if !app.cmd_history.is_empty() {
                            if let Some(v) = app.cmd_history.last() {
                                if *v != app.cmd_input {
                                    app.cmd_history.push(app.cmd_input.clone());
                                }
                            }
                        } else {
                            app.cmd_history.push(app.cmd_input.clone());
                        }
                        prompt.push_str(app.cmd_input.clone().as_str());
                        // app.println(prompt);
                        app.println('\n');
                        run_command(app.cmd_input.clone(), &mut app);

                        app.println(app.prompt.clone());
                        
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
                            let a = app
                                .cmd_history
                                .get_mut(cmd_history_len - app.cmd_history_idx);
                            if let Some(v) = a {
                                app.cmd_input = v.clone();
                            }
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
                                let a = app
                                    .cmd_history
                                    .get_mut(cmd_history_len - app.cmd_history_idx);
                                if let Some(v) = a {
                                    app.cmd_input = v.clone();
                                }
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
