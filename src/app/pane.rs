use std::{
    collections::HashMap,
    env,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread, io::{self, BufReader, BufRead},
};

use tui::text::Text;

use crate::{
    ansi::{render_text, Pos},
    shell::test2::{open_pty, PtySize, MasterPty, CommandBuilder},
};

use super::{get_cmds, vec_empty_pos, VText};

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

impl PaneLocation {
    pub fn plus_x(mut self, x: u16) -> PaneLocation {
        self.x += x;
        self
    }
    pub fn plus_y(mut self, y: u16) -> PaneLocation {
        self.y += y;
        self
    }
    pub fn plus_window(mut self, window: u16) -> PaneLocation {
        self.window += window;
        self
    }
    pub fn x(mut self, x: u16) -> PaneLocation {
        self.x = x;
        self
    }
    pub fn y(mut self, y: u16) -> PaneLocation {
        self.y = y;
        self
    }
    pub fn window(mut self, window: u16) -> PaneLocation {
        self.window = window;
        self
    }
}

pub type PaneRef<'a> = Arc<Mutex<Pane<'a>>>;

// pub type Panes<'a> = HashMap<PaneLocation, PaneRef<'a>>;

#[derive(Clone, Debug)]
pub struct Panes<'a> {
    pub hash_map: HashMap<PaneLocation, PaneRef<'a>>,
}

pub struct PanesIter<'a> {
    content: Vec<(&'a PaneLocation, &'a PaneRef<'a>)>,
    idx: usize,
}

// impl<'a> PanesIter<'a> {
//     pub fn in_window(&'a self, location: PaneLocation) -> PanesIter<'a> {
//         let mut out = Vec::new();
//         for v in self.content {
//             if v.0.window == location.window {
//                 out.push(v);
//             }
//         }
//         PanesIter {
//             content: out,
//             idx: 0,
//         }
//     }
//     pub fn x_in_window(&'a self, location: PaneLocation) -> PanesIter<'a> {
//         let mut out = Vec::new();
//         for v in self.content.clone() {
//             if v.0.window == location.window && v.0.x == location.x {
//                 out.push(v);
//             }
//         }
//         PanesIter {
//             content: out,
//             idx: 0,
//         }
//     }
//     pub fn y_in_window(&'a self, location: PaneLocation) -> PanesIter<'a> {
//         self.filter(|x| {
//             x.0.window == location.window && x.0.y == location.y
//         }).collect()
//     }
// }

impl<'a> FromIterator<(&'a PaneLocation, &'a PaneRef<'a>)> for PanesIter<'a> {
    fn from_iter<T: IntoIterator<Item = (&'a PaneLocation, &'a PaneRef<'a>)>>(iter: T) -> Self {
        let mut out = PanesIter {
            content: Vec::new(),
            idx: 0,
        };
        iter.into_iter().for_each(|x| out.content.push(x));
        out
    }
}

impl PaneLocation {
    pub fn new(window: u16, x: u16, y: u16) -> PaneLocation {
        PaneLocation { window, x, y }
    }
}

impl<'a> Panes<'a> {
    pub fn new() -> Panes<'a> {
        Panes {
            hash_map: HashMap::new(),
        }
    }
    pub fn in_window(&'a self, location: PaneLocation) -> PanesIter<'a> {
        let mut out = Vec::new();
        for v in self.hash_map.iter() {
            if v.0.window == location.window {
                out.push(v);
            }
        }
        PanesIter {
            content: out,
            idx: 0,
        }
    }
    pub fn x_in_window(&'a self, location: PaneLocation) -> PanesIter<'a> {
        let mut out = Vec::new();
        for v in self.hash_map.iter() {
            if v.0.window == location.window && v.0.x == location.x {
                out.push(v);
            }
        }
        PanesIter {
            content: out,
            idx: 0,
        }
    }
    pub fn y_in_window(&'a self, location: PaneLocation) -> PanesIter<'a> {
        let mut out = Vec::new();
        for v in self.hash_map.iter() {
            if v.0.window == location.window && v.0.y == location.y {
                out.push(v);
            }
        }
        PanesIter {
            content: out,
            idx: 0,
        }
    }
}

impl<'a> Iterator for PanesIter<'a> {
    type Item = (&'a PaneLocation, &'a PaneRef<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.content.len() >= self.idx {
            Some(self.content[self.idx])
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum PaneType {
    Normal,
    /// specal type for debug printing within gish, could also be used for errors in ANSI codes and
    /// stuff in futur.
    Debug,
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
    pub scroll: (u16, u16),
    pub max_scroll: (u16, u16),
    pub path: String,
    pub cmds: Vec<String>,
    pub alais: HashMap<String, String>,
    pub cmd_tx: Sender<String>,
    pub update_tx: Sender<bool>,
    pub update_rx: Receiver<bool>,
    pub running_cmd: bool,
    pub is_active: bool,
}

impl<'a> Pane<'a> {
    pub fn debug() -> PaneRef<'static> {
        let (col, row) = crossterm::terminal::size().unwrap();
        let (cmd_tx, cmd_rx) = channel();
        let (update_tx, update_rx) = channel();
        let pair = open_pty(PtySize { row: 50, col: 80, px_width: 0, px_height: 0 }).unwrap();
        let pane = Pane {
            pane_type: PaneType::Debug,
            cmd_input: String::new(),
            size: (200, 50),
            x: 0,
            y: 0,
            vtext: HashMap::new(),
            content: Text::from(""),
            vc: (0, 0),
            vstdout: vec_empty_pos(col, row - 2),
            scrollback: Vec::new(),
            scroll: (0, 0),
            max_scroll: (0, 0),
            path: env::var("PATH").unwrap(),
            cmds: get_cmds(),
            mode: PaneMode::Normal,
            alais: HashMap::new(),
            cmd_tx,
            update_tx,
            update_rx,
            running_cmd: false,
            is_active: false,
        };

        let pane = Arc::new(Mutex::new(pane));
        let i_pane = Arc::clone(&pane);

        thread::spawn(move || {
            loop {
                let msg = match cmd_rx.recv() {
                    Ok(v) => v,
                    Err(_) => {
                        // todo somehow kill the pane when we break. ie, make it so the user sees it
                        // disappear
                        break;
                    }
                };
                i_pane.lock().unwrap().println(format!("{}\n", msg));
            }
        });
        pane
    }

    pub fn new(size: (u16, u16)) -> PaneRef<'static> {
        let (col, row) = crossterm::terminal::size().unwrap();
        let (cmd_tx, cmd_rx) = channel();
        let (update_tx, update_rx) = channel();
        // let pty = open_pty();
        let pane = Pane {
            pane_type: PaneType::Normal,
            cmd_input: String::new(),
            size,
            x: 0,
            y: 0,
            vtext: HashMap::new(),
            content: Text::from(""),
            vc: (0, 0),
            vstdout: vec_empty_pos(col, row - 2),
            scrollback: Vec::new(),
            scroll: (0, 0),
            max_scroll: (0, 0),
            path: env::var("PATH").unwrap(),
            cmds: get_cmds(),
            mode: PaneMode::Normal,
            alais: HashMap::new(),
            cmd_tx,
            update_tx,
            update_rx,
            running_cmd: false,
            is_active: false,
        };

        let pane = Arc::new(Mutex::new(pane));
        let i_pane = Arc::clone(&pane);

        // we need to create a pty and insted of running the commands in this program start some
        // form of shell(zsh bash etc) and have that handle running each command. the only reson we
        // cant do it in the program is we have to drop the master and slave pty every time we run
        // a program and we dont want to create a new pty for every command that is run.
        thread::spawn(move || {
            let pair = open_pty(PtySize { row: 50, col: 80, px_width: 0, px_height: 0 }).unwrap();
            // let pair = (&pair.0, &pair.1);

            let c = CommandBuilder::new("/bin/bash");
            // let mut iter = cmd.split(' ');
            // if iter.next().is_some() {
            //     iter.for_each(|x| c.arg(x.to_string()));
            // }

            let mut child = pair.1.spawn_command(c).unwrap();

            // drop slave
            drop(pair.1);

            let (tx, rx) = channel();
            let reader = pair.0.try_clone_reader().unwrap();


            thread::spawn(move || {
                let mut r = BufReader::with_capacity(1024 * 128, reader);
                loop {
                    let len = {
                        let buffer = match r.fill_buf() {
                            Ok(v) => v,
                            Err(_) => break,
                        };
                        let s: String = buffer.iter().map(|x| *x as char).collect();
                        tx.send(s).unwrap();
                        buffer.len()
                    };
                    if len == 0 {
                        break;
                    }
                    r.consume(len);

                }
            });

            {
                let mut writer = pair.0.take_writer().unwrap();

                // if you want to send data to the child youd set `to_write` to that data
                // let to_write = "";
                // if !to_write.is_empty() {
                thread::spawn(move || {
                    // let mut stdin = io::stdin();
                    // let a = cmd_rx.recv().unwrap();
                    loop {
                        let mut cmd = cmd_rx.recv().unwrap();
                        // panic!("{}", cmd);
                        // cmd.push('\n');
                        writer.write(cmd.as_bytes()).unwrap();
                    }
                });
                // }
            }

            thread::spawn(move || {
                child.wait().unwrap();
                panic!("bash just exited");
                drop(pair.0);
            });

            loop {
                let output = match rx.recv() {
                    Ok(v) => v,
                    Err(_) => break,
                };
                {
                    i_pane.lock().unwrap().println(output);
                }
            }

            // loop {
            //     // wait for a command to be sent then run it. if recv failes(main thread panics)
            //     // then we break and exit this thread.
            //
            //     let cmd = match cmd_rx.recv() {
            //         Ok(v) => v,
            //         Err(_) => {
            //             // todo somehow kill the pane when we break. ie, make it so the user sees it
            //             // disappear
            //             break;
            //         }
            //     };
            //
            //     // run_command(cmd, Arc::clone(&i_pane), pair);
            // }
        });
        pane
    }

    pub fn println<T: ToString>(&mut self, msg: T) {
        self.content = render_text(self, msg.to_string());
        self.update_tx.send(true).unwrap();
    }

    pub fn dbg<T: ToString>(&mut self, msg: T) {
        if let PaneType::Debug = self.pane_type {
            self.content = render_text(self, msg.to_string());
            self.update_tx.send(true).unwrap();
        }
    }

    pub fn run_command(&mut self, cmd: String) {
        self.running_cmd = true;
        self.cmd_tx.send(cmd).unwrap();
        self.cmd_input.clear();
    }
}
