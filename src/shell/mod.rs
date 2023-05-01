use std::process::{Command, Stdio};

use tui::text::Text;

use crate::{app::{App, reset_terminal}, ansi::test};

pub fn run_if_builtin(cmd: String) -> bool {
    let path = cmd.split(" ").next().unwrap().to_string();
    match path.as_str() {
        "exit" => {
            reset_terminal().unwrap();
            std::process::exit(0);
        }
        &_ => false
    }
}

pub fn run_command(cmd: String, app: &mut App) {
    if cmd.is_empty() {
        return;
    }
    app.cmd_input.clear();
    let path = cmd.split(" ").next().unwrap().to_string();
    if run_if_builtin(cmd.clone()) {
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
    test(content.clone());
    app.content.extend(Text::from(content));

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
