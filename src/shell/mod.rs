// resources
// pty and other usefull info https://stackoverflow.com/questions/65175134/what-can-you-do-with-a-pty
// pty example in c++. i cant get it to compile but its interesting to look at https://gist.github.com/zmwangx/2bac2af9195cad47069419ccd9ee98d8

use std::{process::{Command, Stdio}, io::{Read, Write}, os::fd::AsRawFd};

use hlua::{LuaTable, LuaFunction};
use openpty::openpty;
use pty::prelude::Fork;


use crate::{app::{App, reset_terminal}, ansi::test};

pub mod test;

pub fn get_internal_cmds() -> Vec<String> {
    vec![
        String::from("exit"),
        String::from("refresh"),
        String::from("alais"),
    ]
}

pub fn run_if_builtin(app: &mut App, cmd: String) -> Option<()> {
    let path = cmd.split(' ').next().unwrap().to_string();
    match path.as_str() {
        "exit" => {
            reset_terminal().unwrap();
            std::process::exit(0);
        }
        "refresh" => {
            app.println("refreshing\n");
            let mut lua = app.lua.borrow_mut();
            let mut internal: LuaTable<_> = lua.get("_internal").unwrap();
            let run_event: Option<LuaFunction<_>> = internal.get("run_event");

            if let Some(mut v) = run_event {
                let mut table: LuaTable<_> = v.call_with_args("refresh").unwrap();
                for (k, v) in table.iter::<String, String>().flatten() {
                    match k.as_str() {
                        "alais" => {
                            for alais in v.split('\n') {
                                let (k, a) = match alais.split_once(':') {
                                    Some(v) => v,
                                    None => break,
                                };
                                app.alais.insert(k.to_string(), a.to_string());
                            }
                        }
                        _ => {}
                    }
                }
            }
            Some(())
        }
        "alais" => {
            // todo cmd.split(' ') does not work because of "" and stuff. this should be fixed when
            // we start actualy complying with bash but till then it should probubly work better
            // then it does right now.
            // also cmd should be its own type.
            let name = cmd.split(' ').nth(1)?.to_string();
            let value = cmd.split(' ').nth(2)?.to_string();
            app.alais.insert(name, value);
            Some(())
        }
        &_ => None
    }
}
pub fn new_run_command(mut cmd: String, app: &mut App) {
    if cmd.is_empty() {
        return;
    }

    app.cmd_input.clear();
    let path = cmd.split(' ').next().unwrap().to_string();

    if let Some(alais) = app.alais.get(&path) {
        let trimed_cmd = cmd.trim_start_matches(&path);
        let mut alais = alais.clone();
        alais.push_str(trimed_cmd);
        cmd = alais.trim().to_string();
    }

    if run_if_builtin(app, cmd.clone()).is_some() {
        return;
    }


    let (mut master, mut slave, _) = openpty(None, None, None).expect("creating pty failed");


    // slave.write_all(b"hello world").unwrap();




    // let mut out = String::new();
    // master.read_to_string(&mut out).unwrap_err();
    // app.println(out);

    let mut p = Command::new(&path);


    // close slave when the program exits
    let flags = unsafe { libc::fcntl(slave.as_raw_fd(), libc::F_GETFD) };
    // should maybe handle error when flags is -1
    unsafe { libc::fcntl(slave.as_raw_fd(), libc::F_SETFD, flags | libc::FD_CLOEXEC) };

    // close master when the program exits
    let flags = unsafe { libc::fcntl(master.as_raw_fd(), libc::F_GETFD) };
    // should maybe handle error when flags is -1
    unsafe { libc::fcntl(master.as_raw_fd(), libc::F_SETFD, flags | libc::FD_CLOEXEC) };

    p.stdout(slave);

    let mut iter = cmd.split(' ');
    iter.next();
    for arg in iter {
        p.arg(arg);
    }

    let mut p = match p.spawn() {
        Ok(v) => v,
        Err(_) => {
            let msg = format!("`{}` command not found\n", path);
            app.println(msg);
            return;
        }
    };

    p.stdout.take();
    p.stdin.take();
    p.stderr.take();
    p.wait().unwrap();

    // let content: String = output.stdout.iter().fold(String::new(), |mut acc, &x| {
    //     acc.push(x as char);
    //     acc
    // });
    
    let mut content = String::new();
    master.read_to_string(&mut content).unwrap();

    let new_content = test(app, content);
    app.content = new_content;
}

pub fn run_command(mut cmd: String, app: &mut App) {
    if cmd.is_empty() {
        return;
    }

    app.cmd_input.clear();
    let path = cmd.split(' ').next().unwrap().to_string();

    if let Some(alais) = app.alais.get(&path) {
        let trimed_cmd = cmd.trim_start_matches(&path);
        let mut alais = alais.clone();
        alais.push_str(trimed_cmd);
        cmd = alais.trim().to_string();
    }

    if run_if_builtin(app, cmd.clone()).is_some() {
        return;
    }




    let mut p = Command::new(&path);
    p.stdout(Stdio::piped());

    let mut iter = cmd.split(' ');
    iter.next();
    for arg in iter {
        p.arg(arg);
    }

    let p = match p.spawn() {
        Ok(v) => v,
        Err(_) => {
            let msg = format!("`{}` command not found\n", path);
            app.println(msg);
            return;
        }
    };

    let output = p.wait_with_output().unwrap();

    let content: String = output.stdout.iter().fold(String::new(), |mut acc, &x| {
        acc.push(x as char);
        acc
    });
    
    let new_content = test(app, content);
    app.content = new_content;
    // app.content.extend(Text::from(content));

    // let stdout = p.stdout.unwrap();
    // let mut bufr = BufReader::new(stdout);
    //
    // let mut buf = vec![];
    // loop {
    //     if bufr.(&mut buf).unwrap() == 0 {
    //         break;
    //     }
    //     let line: String = buf.iter().fold(String::new(), |mut acc, &x| {
    //         acc.push(x as char);
    //         acc
    //     });
    //     app.content.extend(Text::from(line));
    //     buf.clear();
    // }
    //
    // while let Ok(n_bytes) = stdout.read(&mut buf) {
    //     if n_bytes == 0 {
    //         break
    //     }
    //     let line: String = buf.iter().fold(String::new(), |mut acc, &x| {
    //         acc.push(x as char);
    //         acc
    //     });
    //     app.content.extend(Text::from(line));
    //     buf.clear();
    // }
}
