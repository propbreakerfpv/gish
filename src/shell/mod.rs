// resources
// pty and other usefull info https://stackoverflow.com/questions/65175134/what-can-you-do-with-a-pty
// pty example in c++. i cant get it to compile but its interesting to look at https://gist.github.com/zmwangx/2bac2af9195cad47069419ccd9ee98d8

use std::{io::{Write, BufReader, BufRead}, env, sync::{mpsc::channel, Arc}, thread};

// use hlua::{LuaTable, LuaFunction, Lua};


use crate::app::{reset_terminal, pane::PaneRef};

use self::test2::{open_pty, PtySize, CommandBuilder};

pub mod test;
pub mod test2;

pub fn get_internal_cmds() -> Vec<String> {
    vec![
        String::from("exit"),
        String::from("refresh"),
        String::from("alais"),
    ]
}

pub fn run_if_builtin(pane: PaneRef, cmd: String) -> Option<()> {
    let mut pane = pane.lock().unwrap();
    let path = cmd.split(' ').next().unwrap().to_string();
    match path.as_str() {
        "exit" => {
            reset_terminal().unwrap();
            std::process::exit(0);
        }
        "refresh" => {
            pane.println("refreshing\n");
            // todo make this call lua code and stuff. will probubly require makeing a lua thread
            // that cann be called to with a lock or channel or smth


            // let mut internal: LuaTable<_> = lua.get("_internal").unwrap();
            // let run_event: Option<LuaFunction<_>> = internal.get("run_event");

            // if let Some(mut v) = run_event {
            //     let mut table: LuaTable<_> = v.call_with_args("refresh").unwrap();
            //     for (k, v) in table.iter::<String, String>().flatten() {
            //         match k.as_str() {
            //             "alais" => {
            //                 for alais in v.split('\n') {
            //                     let (k, a) = match alais.split_once(':') {
            //                         Some(v) => v,
            //                         None => break,
            //                     };
            //                     pane.alais.insert(k.to_string(), a.to_string());
            //                 }
            //             }
            //             _ => {}
            //         }
            //     }
            // }
            Some(())
        }
        "alais" => {
            // todo cmd.split(' ') does not work because of "" and stuff. this should be fixed when
            // we start actualy complying with bash but till then it should probubly work better
            // then it does right now.
            // also cmd should be its own type.
            let name = cmd.split(' ').nth(1)?.to_string();
            let value = cmd.split(' ').nth(2)?.to_string();
            pane.alais.insert(name, value);
            Some(())
        }
        "cd" => {
            // todo
            env::set_var("PWD", "/home/prop/");
            Some(())
        }
        &_ => None
    }
}

pub fn after_run_command(mut cmd: String, pane_ref: PaneRef) {
    let mut pane = pane_ref.lock().unwrap();
    let prompt = pane.prompt.clone();
    pane.println(prompt);
    pane.running_cmd = false;
}

pub fn new_run_command(mut cmd: String, pane_ref: PaneRef) {
    let path = cmd.split(' ').next().unwrap().to_string();

    {
        let pane = Arc::clone(&pane_ref);
        let mut pane = pane.lock().unwrap();

        if cmd.is_empty() {
            // drop pane to prevent dedlock when we try and look pane_ref in after_run_command
            drop(pane);
            after_run_command(cmd, pane_ref);
            return;
        }

        pane.cmd_input.clear();

        if let Some(alais) = pane.alais.get(&path) {
            let trimed_cmd = cmd.trim_start_matches(&path);
            let mut alais = alais.clone();
            alais.push_str(trimed_cmd);
            cmd = alais.trim().to_string();
        }
    }

    if run_if_builtin(Arc::clone(&pane_ref), cmd.clone()).is_some() {
        after_run_command(cmd, pane_ref);
        return;
    }


    let pair = open_pty(PtySize { row: 50, col: 80, px_width: 0, px_height: 0 }).unwrap();
    let mut c = CommandBuilder::new(path);
    let mut iter = cmd.split(' ');
    if iter.next().is_some() {
        iter.for_each(|x| c.arg(x.to_string()));
    }


    let mut child = pair.1.spawn_command(c).unwrap();


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
        let to_write = "";
        if !to_write.is_empty() {
            thread::spawn(move || {
                writer.write_all(to_write.as_bytes()).unwrap();
            });
        }
    }

    thread::spawn(move || {
        child.wait().unwrap();
        drop(pair.0);
    });

    loop {
        let output = match rx.recv() {
            Ok(v) => v,
            Err(_) => break,
        };
        {
            pane_ref.lock().unwrap().println(output);
        }
    }
    after_run_command(cmd, pane_ref)
}

