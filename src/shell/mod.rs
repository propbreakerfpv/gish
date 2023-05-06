use std::process::{Command, Stdio};

use hlua::{LuaTable, LuaFunction};
use tui::text::Text;

use crate::{app::{App, reset_terminal}, ansi::test};

pub fn run_if_builtin(app: &mut App, cmd: String) -> Option<()> {
    let path = cmd.split(" ").next().unwrap().to_string();
    match path.as_str() {
        "exit" => {
            reset_terminal().unwrap();
            std::process::exit(0);
        }
        "refresh" => {
            app.content.extend(Text::from("refreshing"));
            let mut lua = app.lua.borrow_mut();
            let mut internal: LuaTable<_> = lua.get("_internal").unwrap();
            let run_event: Option<LuaFunction<_>> = internal.get("run_event");

            match run_event {
                Some(mut v) => {
                    let mut table: LuaTable<_> = v.call_with_args("refresh").unwrap();
                    for (k, v) in table.iter::<String, String>().filter_map(|e| e) {
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
                None => {}
            };
            Some(())
        }
        "alais" => {
            // todo cmd.split(' ') does not work because of "" and stuff. this should be fixed when
            // we start actualy complying with bash but till then it should probubly work better
            // then it does right now.
            // also cmd should be its own type.
            let name = cmd.split(' ').nth(1)?.to_string();
            let value = cmd.split(' ').nth(2)?.to_string();
            app.alais.insert(name.clone(), value);
            Some(())
        }
        &_ => None
    }
}

pub fn run_command(mut cmd: String, app: &mut App) {
    if cmd.is_empty() {
        return;
    }

    app.cmd_input.clear();
    let path = cmd.split(" ").next().unwrap().to_string();

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

    let output = p.wait_with_output().unwrap();

    let content: String = output.stdout.iter().fold(String::new(), |mut acc, &x| {
        acc.push(x as char);
        acc
    });
    
    let new_content = test(app, content.clone());
    app.content.extend(new_content);
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
