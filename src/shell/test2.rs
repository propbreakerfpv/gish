use std::{os::{unix::{process::CommandExt, prelude::OsStrExt}, fd::{RawFd, FromRawFd, AsRawFd}}, ptr, io::{self, Read, Write, BufRead, BufReader}, fs::File, cell::RefCell, path::PathBuf, process::{Command, Child}, sync::mpsc::channel, thread, vec, ffi::OsStr};

use anyhow::anyhow;

//https://stackoverflow.com/questions/72150987/why-does-reading-from-an-exited-pty-process-return-input-output-error-in-rust

// fn main() {
//     let pair = open_pty(PtySize { row: 50, col: 80, px_width: 0, px_height: 0 }).unwrap();
//     let cmd = CommandBuilder::new("tty");
//
//     let mut child = pair.1.spawn_command(cmd).unwrap();
//
//     // let kept_reader = pair.1.try_clone_reader().unwrap();
//
//     drop(pair.1);
//
//     let (tx, rx) = channel();
//     let reader = pair.0.try_clone_reader().unwrap();
//
//
//     thread::spawn(move || {
//         let mut r = BufReader::with_capacity(1024 * 128, reader);
//         loop {
//             // let mut buf = Vec::new();
//             // // reader.read_to_string(&mut s);
//             // reader.read(&mut buf).unwrap();
//             // let last = buf.last().unwrap_or(&b' ');
//             // if last  == &b'\0' {
//             //     break;
//             // }
//
//             let len = {
//                 let buffer = match r.fill_buf() {
//                     Ok(v) => v,
//                     Err(_) => break,
//                 };
//                 let s: String = buffer.iter().map(|x| *x as char).collect();
//                 // let a = buffer.clone();
//                 tx.send(s).unwrap();
//                 buffer.len()
//             };
//             if len == 0 {
//                 break;
//             }
//             r.consume(len);
//
//             // println!("thread {:?}", buf);
//             // tx.send(buf).unwrap();
//         }
//         // drop(kept_reader);
//     });
//
//     {
//         let mut writer = pair.0.take_writer().unwrap();
//
//         // if you want to send data to the child youd set `to_write` to that data
//         let to_write = "";
//         if !to_write.is_empty() {
//             thread::spawn(move || {
//                 writer.write_all(to_write.as_bytes()).unwrap();
//             });
//         }
//     }
//
//     thread::spawn(move || {
//         child.wait().unwrap();
//         drop(pair.0);
//     });
//
//
//     loop {
//         let output = match rx.recv() {
//             Ok(v) => v,
//             Err(_) => break,
//         };
//         print!("{}", output)
//     }
//
//     // print!("output: ");
//     // for c in output.escape_debug() {
//     //     print!("{}", c);
//     // }
// }


pub struct CommandBuilder {
    pub args: Vec<String>,
    pub path: String,
}
impl CommandBuilder {
    pub fn new<T: ToString>(prog: T) -> CommandBuilder {
        CommandBuilder {
            args: Vec::new(),
            path: prog.to_string()
        }
    }
    pub fn arg<S: ToString>(&mut self, arg: S) {
        self.args.push(arg.to_string());
    }
    pub fn as_command(self) -> Command {
        let mut command = Command::new(self.path);
        for arg in self.args {
            command.arg(arg);
        }

        command
    }
}

pub struct PtySize {
    pub row: u16,
    pub col: u16,
    pub px_width: u16,
    pub px_height: u16,
}

pub struct MasterPty {
    pub fd: File,
    pub took_writer: RefCell<bool>,
    pub tty_name: Option<PathBuf>
}

impl MasterPty {
    pub fn try_clone_reader(&self) -> anyhow::Result<Box<dyn Read + Send>> {
        let fd = self.fd.try_clone()?;
        Ok(Box::new(fd))
    }

    pub fn take_writer(&self) -> anyhow::Result<Box<dyn Write + Send>> {
        if *self.took_writer.borrow() {
            anyhow::bail!("cannot take writer more than once");
        }
        *self.took_writer.borrow_mut() = true;
        let fd = self.fd.try_clone()?;
        Ok(Box::new(fd))
    }
}

pub struct SlavePty {
    pub fd: File,
}


impl SlavePty {
    pub fn try_clone_reader(&self) -> anyhow::Result<Box<dyn Read + Send>> {
        let fd = self.fd.try_clone()?;
        Ok(Box::new(fd))
    }
    pub fn spawn_command(&self, cmd: CommandBuilder) -> anyhow::Result<Child> {


        let mut cmd = cmd.as_command();
        unsafe {
            cmd.stdin(dup_fd(&self.fd)?)
                .stdout(dup_fd(&self.fd)?)
                .stderr(dup_fd(&self.fd)?)
                .pre_exec(move || {
                    for signo in &[
                        libc::SIGCHLD,
                        libc::SIGHUP,
                        libc::SIGINT,
                        libc::SIGQUIT,
                        libc::SIGTERM,
                        libc::SIGALRM,
                    ] {
                        libc::signal(*signo, libc::SIG_DFL);
                    }

                    let empty_set: libc::sigset_t = std::mem::zeroed();
                    libc::sigprocmask(libc::SIG_SETMASK, &empty_set, std::ptr::null_mut());

                    if libc::setsid() == -1 {
                        return Err(io::Error::last_os_error());
                    }

                    // if controlling_tty {
                    //     if libc::ioctl(0, libc::TIOCSCTTY as _, 0) == -1 {
                    //         return Err(io::Error::last_os_error());
                    //     }
                    // }

                    close_random_fds();

                    // if let Some(mask) = configured_unmastk {
                    //     libc::umask(mask);
                    // }

                    Ok(())
                })
        };

        let mut child = cmd.spawn()?;

        child.stdin.take();
        child.stdout.take();
        child.stderr.take();

        Ok(child)
    }
}

/// not sure why we need this but apparenty its nessasary??
pub fn close_random_fds() {
    if let Ok(dir) = std::fs::read_dir("/dev/fd") {
        let mut fds = vec![];
        for entry in dir {
            if let Some(num) = entry
                .ok()
                    .map(|e| e.file_name())
                    .and_then(|s| s.into_string().ok())
                    .and_then(|n| n.parse::<libc::c_int>().ok())
                    {
                        if num > 2 {
                            fds.push(num);
                        }
                    }

        }
        for fd in fds {
            unsafe {
                libc::close(fd);
            }
        }
    }
}

pub fn dup_fd(fd: &File) -> anyhow::Result<File> {
    let fd = fd.as_raw_fd();

    let duped = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 0)};

    if duped == -1 {
        let err = std::io::Error::last_os_error();
        if let Some(libc::EINVAL) = err.raw_os_error() {
            // todo suport kernal versions before 2.6.24
            panic!("your kernal version does not suport `fcntl((), F_DUPFD_CLOEXEC)` and duping fds without this is not currently suported");
        } else {
            Err(anyhow!("error duping {}", err))
        }
    } else {
        Ok(unsafe {
            File::from_raw_fd(duped)  
        })
    }

}



pub fn open_pty(size: PtySize) -> anyhow::Result<(MasterPty, SlavePty)> {
    let mut master: RawFd = -1;
    let mut slave: RawFd = -1;

    let mut size = libc::winsize {
        ws_row: size.row,
        ws_col: size.col,
        ws_xpixel: size.px_width,
        ws_ypixel: size.px_height,
    };

    let result = unsafe {
        libc::openpty(&mut master, &mut slave, ptr::null_mut(), ptr::null_mut(), &mut size)
    };

    if result != 0 {
        panic!("failed to openpty: {:?}", io::Error::last_os_error());
    }

    let tty_name = tty_name(slave);

    let master = MasterPty {
        fd: unsafe { File::from_raw_fd(master) },
        took_writer: RefCell::new(false),
        tty_name,
    };

    let slave = SlavePty {
        fd: unsafe { File::from_raw_fd(slave) },
    };

    cloexec(master.fd.as_raw_fd()).unwrap();
    cloexec(slave.fd.as_raw_fd()).unwrap();
    Ok((master, slave))
}


pub fn cloexec(fd: RawFd) -> anyhow::Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags == -1 {
        panic!(
            "fcntl to read flags failed: {:?}",
            io::Error::last_os_error()
        );
    }
    let result = unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) };
    if result == -1 {
        panic!(
            "fcntl to set CLOEXEC failed: {:?}",
            io::Error::last_os_error()
        );
    }
    Ok(())
}

pub fn tty_name(fd: RawFd) -> Option<PathBuf> {
    let mut buf = vec![0 as std::ffi::c_char; 128];

    loop {
        let res = unsafe { libc::ttyname_r(fd, buf.as_mut_ptr(), buf.len()) };

        if res == libc::ERANGE {
            buf.resize(buf.len() * 2, 0 as std::ffi::c_char);
            continue;
        }

        return if res == 0 {
            let cstr = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
            let osstr = OsStr::from_bytes(cstr.to_bytes());
            Some(PathBuf::from(osstr))
        } else {
            None
        }
    }
}
