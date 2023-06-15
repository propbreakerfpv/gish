use std::{collections::HashMap, sync::{mpsc::{channel, Sender, Receiver}, Arc, Mutex}, thread, env};

use tui::text::Text;

use crate::{ansi::{Pos, render_text}, shell::new_run_command};

use super::{VText, get_cmds, vec_empty_pos};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PaneLocation {
    window: u16,
    x: u16,
    y: u16,
}
impl Default for PaneLocation {
    fn default() -> Self {
        PaneLocation {
            window: 1,
            x: 0,
            y: 0,
        }
    }
}

pub type PaneRef<'a> = Arc<Mutex<Pane<'a>>>;

#[derive(Debug)]
pub enum PaneType {
    Normal,
    // specal type for debug printing within gish, could also be used for errors in ANSI codes and
    // stuff in futur.
    // Debug,
}


#[derive(Debug)]
pub enum PaneMode {
    Normal,
}

#[derive(Debug)]
pub struct Pane<'a> {
    pub pane_type: PaneType,
    pub cmd_input: String,
    /// x, y: width, hight
    pub size: (u16, u16),
    pub x: u16,
    pub y: u16,
    pub vtext: HashMap<String, VText<'a>>,
    pub content: Text<'a>,
    /// x, y: horazontal, vertical
    pub vc: (u16, u16),
    pub vstdout: Vec<Vec<Pos>>,
    pub scrollback: Vec<Vec<Pos>>,
    pub mode: PaneMode,
    pub prompt: String,
    pub prompt_update: bool,
    pub cmd_history: Vec<String>,
    pub cmd_history_idx: usize,
    pub scroll: (u16, u16),
    pub max_scroll: (u16, u16),
    pub path: String,
    pub cmds: Vec<String>,
    pub alais: HashMap<String, String>,
    pub cmd_tx: Sender<String>,
    pub update_tx: Sender<bool>,
    pub update_rx: Receiver<bool>,
}



impl<'a> Pane<'a> {
    pub fn new() -> PaneRef<'static> {
        let (col, row) = crossterm::terminal::size().unwrap();
        let (cmd_tx, cmd_rx) = channel();
        let (update_tx, update_rx) = channel();
        let pane = Pane {
            pane_type: PaneType::Normal,
            cmd_input: String::new(),
            size: (200, 50),
            x: 0,
            y: 0,
            vtext: HashMap::new(),
            content: Text::from(""),
            vc: (0, 0),
            vstdout: vec_empty_pos(col, row-2),
            scrollback: Vec::new(),
            prompt: String::new(),
            prompt_update: false,
            cmd_history: Vec::new(),
            cmd_history_idx: 0,
            scroll: (0, 0),
            max_scroll: (0, 0),
            path: env::var("PATH").unwrap(),
            cmds: get_cmds(),
            mode: PaneMode::Normal,
            alais: HashMap::new(),
            cmd_tx,
            update_tx,
            update_rx,
        };
        let pane = Arc::new(Mutex::new(pane));
        let i_pane = Arc::clone(&pane);

        thread::spawn(move || {
            loop {
                // wait for a command to be sent then run it. if recv failes(main thread panics)
                // then we break and exit this thread.
                
                let cmd = match cmd_rx.recv() {
                    Ok(v) => v,
                    Err(_) => {
                        // todo somehow kill the pane when we break. ie, make it so the user sees it
                        // disappear
                        break;
                    }
                };

                new_run_command(cmd, Arc::clone(&i_pane));
            }
        });
        pane
    }

    pub fn println<T: ToString>(&mut self, msg: T) {
        self.content = render_text(self, msg.to_string());
        self.update_tx.send(true).unwrap();
    }

    pub fn run_command(&mut self, cmd: String) {
        self.cmd_tx.send(cmd).unwrap();
    }
}

