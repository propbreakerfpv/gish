use std::{cell::RefCell, collections::HashMap, error::Error, fs, io, env, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode, MouseEventKind},
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use hlua::Lua;
use tui::{
    backend::Backend,
    text::Text,
    Terminal, widgets::Paragraph, layout::Rect,
};

use crate::{ansi::Pos, lua::{setup_lua, self}, ui::ui};

use self::{auto_comp::on_comp, config::Config, pane::{PaneLocation, Pane, Panes, PaneType}};

pub mod auto_comp;
pub mod config;
pub mod pane;
pub mod new_pane;

fn vec_empty_pos(width: u16, hight: u16) -> Vec<Vec<Pos>> {
    let mut out = Vec::new();
    for _ in 1..=hight {
        out.push(vec![Pos::Empty; width as usize]);
    }
    out
}


#[derive(Debug)]
pub struct Window {
}

#[derive(Debug)]
pub struct App<'a> {
    pub search_input: String,
    pub cmd_input: String,
    /// x, y: width, hight
    pub size: (u16, u16),
    pub vtext: HashMap<String, VText<'a>>,
    pub content: Text<'a>,
    /// x, y: horazontal, vertical
    pub vc: (u16, u16),
    pub vstdout: Vec<Vec<Pos>>,
    pub scrollback: Vec<Vec<Pos>>,
    pub config: Config,
    pub mode: AppMode,
    pub prompt: String,
    pub prompt_update: bool,
    pub status_bar: Text<'a>,
    pub cmd_history: Vec<String>,
    pub cmd_history_idx: usize,
    pub alais: HashMap<String, String>,
    pub scroll: (u16, u16),
    pub max_scroll: (u16, u16),
    pub path: String,
    pub cmds: Vec<String>,
    pub lua: RefCell<Lua<'a>>,
    pub windows: Vec<Window>,
    pub panes: Panes<'a>,
    pub active_window: u16,
    pub active_pane: PaneLocation,
}

#[derive(Debug)]
pub struct VText<'a> {
    pub p: Paragraph<'a>,
    pub size: Rect, 
}

impl<'a> App<'a> {
    pub fn new() -> App<'static> {
        let (col, row) = crossterm::terminal::size().unwrap();
        // panic!("col {}, row {}", col, row);
        let mut app = App {
            search_input: String::new(),
            cmd_input: String::new(),
            size: (col, row),
            vtext: HashMap::new(),
            content: Text::from(""),
            vc: (0, 0),
            vstdout: vec_empty_pos(col, row-2),
            scrollback: Vec::new(),
            config: Config::default(),
            mode: AppMode::Normal,
            prompt: String::new(),
            prompt_update: false,
            status_bar: Text::from("status_bar"),
            cmd_history: Vec::new(),
            cmd_history_idx: 0,
            alais: HashMap::new(),
            scroll: (0, 0),
            max_scroll: (0, 0),
            path: env::var("PATH").unwrap(),
            cmds: get_cmds(),
            lua: RefCell::new(Lua::new()),
            windows: vec![],
            panes: Panes::new(),
            active_window: 1,
            active_pane: PaneLocation::default(),
        };
        let pane_ref = Pane::new(app.size);
        {
            let mut pane = pane_ref.lock().unwrap();
            pane.is_active = true;
        }
        app.panes.hash_map.insert(PaneLocation::default(), pane_ref);
        app
    }
    pub fn setup_lua(&self) {
        setup_lua(self);
    }
    // pub fn add_content(&mut self, content: String) {
    //
    //     Text::from(content);
    // }
    pub fn println<T: ToString>(&mut self, msg: T) {
        let pane_ref = self.panes.hash_map.get_mut(&self.active_pane).unwrap();
        let mut pane = pane_ref.lock().unwrap();
        pane.println(msg)
    }

    pub fn dbg<T: ToString>(&mut self, msg: T) {
        // todo make this find any debug panes and print to them. should also maybe be able to
        // specify a specific pane.
        let pane = self.panes.hash_map.get(&PaneLocation::default().x(1)).unwrap().lock().unwrap();
        if let PaneType::Debug = pane.pane_type {
            pane.cmd_tx.send(msg.to_string()).unwrap();
        }
    }
    pub fn update_app(&mut self) {
        let path = env::var("PATH").unwrap();
        if self.path != path {
            self.path = path;
            self.cmds = get_cmds();
        }
    }

    // fn add_pane(&mut self) {
    //     let new_pane_ref;
    //     let location;
    //     let active_pane = self.active_pane.clone();
    //     {
    //         let pane_ref = self.panes.hash_map.get(&self.active_pane).unwrap();
    //         let mut pane = pane_ref.lock().unwrap();
    //
    //         if pane.size.0 > pane.size.1 {
    //             pane.size.0 = pane.size.0 / 2;
    //             location = active_pane.plus_x(1);
    //         } else {
    //             pane.size.1 = pane.size.1 / 2;
    //             location = active_pane.plus_y(1);
    //         }
    //         let size = pane.size;
    //
    //         new_pane_ref = Pane::new(size);
    //         // let new_pane = new_pane_ref.lock().unwrap();
    //     }
    //     self.panes.hash_map.insert(location, new_pane_ref);
    // }
}

fn get_cmds() -> Vec<String> {
    let mut out = Vec::new();
    let paths = env::var("PATH").unwrap();
    let paths: Vec<&str> = paths.split(':').collect();
    for path in paths {
        let dir = match fs::read_dir(path) {
            Ok(v) => v,
            Err(_) => continue,
        };
        for file in dir {
            let file = file.unwrap();
            out.push(file.file_name().to_str().unwrap().to_string());
        }
    }
    out
}

/// Normal
/// Searching
/// Command
#[derive(Debug, Clone)]
pub enum AppMode {
    Normal,
    // todo figure out if searching is going to be a mode
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

    // let startup = fs::read_to_string("./config/startup.sh").unwrap();
    // for line in startup.lines() {
    //     run_command(line.trim().to_string(), &mut app)
    // }

    app.prompt = lua::get_prompt(&mut app);
    app.println(app.prompt.clone());


    let mut update = true;

    loop {
        {
            let pane_ref = app.panes.hash_map.get_mut(&app.active_pane).unwrap();
            let mut pane = pane_ref.lock().unwrap();
            if pane.scroll.0 > pane.max_scroll.0 {
                pane.max_scroll = pane.scroll;
            }
        }

        if update {
            terminal.draw(|f| ui(f, &mut app))?;
            update = false;
        }



        let event;
        if event::poll(Duration::from_millis(1))? {
            event = event::read()?;
        } else {
            for (_, pane) in app.panes.hash_map.clone() {
                if pane.lock().unwrap().update_rx.recv_timeout(Duration::from_millis(1/app.panes.hash_map.len() as u64)).is_ok() {
                    update = true;
                    break;
                }
            }
            continue;
        }
        match event {
            Event::Mouse(e) => {
                match e.kind {
                    MouseEventKind::ScrollUp => {
                        let pane_ref = app.panes.hash_map.get_mut(&app.active_pane).unwrap();
                        let mut pane = pane_ref.lock().unwrap();
                        if app.config.scroll_amount > pane.scroll.0 {
                            app.scroll.0 = 0;
                        } else {
                            pane.scroll.0 -= app.config.scroll_amount;
                        }
                    }
                    MouseEventKind::ScrollDown => {
                        let pane_ref = app.panes.hash_map.get_mut(&app.active_pane).unwrap();
                        let mut pane = pane_ref.lock().unwrap();
                        if pane.scroll.0 + app.config.scroll_amount < pane.max_scroll.0 {
                            pane.scroll.0 += app.config.scroll_amount;
                        } else if pane.scroll.0 < pane.max_scroll.0 {
                            pane.scroll.0 = pane.max_scroll.0;
                        }
                    }
                    _ => {}
                }
            }
            Event::Key(key) => {
                match app.mode.clone() {
                    AppMode::Normal => match key.code {
                        // todo ctr-c kill cmd
                        // KeyCode::Char('c') => {
                        //
                        //     key.modifiers.contains(other);
                        //     app.println("kill prog");
                        // }
                        KeyCode::Home => {
                            // todo this should not be here. just for testing lol
                            // app.add_pane();
                        }
                        KeyCode::Char(c) => {
                            let pane_ref = app.panes.hash_map.get_mut(&app.active_pane).unwrap();
                            {
                                let mut pane = pane_ref.lock().unwrap();
                                pane.cmd_history_idx = 0;
                                pane.cmd_input.push(c);
                                pane.prompt_update = true;
                                pane.scroll = (pane.content.height() as u16 - pane.vstdout.len() as u16, pane.scroll.1);
                            }
                            update = true;
                        }
                        KeyCode::Backspace => {
                            let pane_ref = app.panes.hash_map.get_mut(&app.active_pane).unwrap();
                            let mut pane = pane_ref.lock().unwrap();

                            if pane.cmd_input.pop().is_some() {
                                pane.println("\x1b[1D \x1b[1D");
                            }
                        }
                        KeyCode::Tab => {
                            let pane_ref = app.panes.hash_map.get_mut(&app.active_pane).unwrap();
                            let pane = pane_ref.lock().unwrap();
                            on_comp(pane);
                        }
                        KeyCode::Enter => {
                            let cmd = {
                                let pane_ref = app.panes.hash_map.get_mut(&app.active_pane).unwrap();
                                let mut pane = pane_ref.lock().unwrap();

                                pane.scroll = (pane.content.height() as u16 - pane.vstdout.len() as u16, pane.scroll.1);

                                if pane.cmd_history_idx > 0 {
                                    // todo this should not be get_mut but the barrow checker is playing
                                    // games with me so thats what it is for now.
                                    let cmd_history_len = pane.cmd_history.clone().len();
                                    let cmd_history_idx = pane.cmd_history_idx;
                                    let a =pane 
                                        .cmd_history
                                        .get_mut(cmd_history_len - cmd_history_idx);
                                    if let Some(v) = a {
                                        pane.cmd_input = v.clone();
                                    }
                                }
                                let mut prompt = pane.prompt.clone();
                                let cmd_input = pane.cmd_input.clone();
                                if !pane.cmd_history.is_empty() {
                                    if let Some(v) = pane.cmd_history.last() {
                                        if *v != pane.cmd_input {
                                            pane.cmd_history.push(cmd_input);
                                        }
                                    }
                                } else {
                                    pane.cmd_history.push(cmd_input);
                                }
                                prompt.push_str(pane.cmd_input.clone().as_str());
                                pane.println('\n');

                                let cmd = pane.cmd_input.clone();
                                pane.run_command(cmd.clone());
                                cmd
                            };
                            app.dbg(format!("running command {}", cmd));
                            // todo make this run some version of app.update_app()
                            // app.update_app();
                        }
                        KeyCode::End => {
                            app.mode = AppMode::Command;
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
            _ => {}
        }
    }
}
