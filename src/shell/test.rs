use std::{ptr, fs::{File, read, self}, os::{fd::FromRawFd, unix::prelude::FileExt}, process::Command, ffi::CString, io::{self, Read}};

use libc::{winsize, c_char, c_void};

pub fn test() {

    let (master, slave) = openpty();

    let mut cmd = Command::new("whoami");

    let mut child = cmd.spawn().unwrap();
    
    // child.
}

fn openpty() -> (File, File) {

    let mut master = -1;
    let mut slave = -1;

    let mut size = winsize {
        ws_row: 50,
        ws_col: 100,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    let res = unsafe {libc::openpty(&mut master, &mut slave, ptr::null_mut(), ptr::null_mut(), &mut size)};
    if res != 0 {
        panic!("opening pty failed");
    }

    // close slave when the program exits
    let flags = unsafe { libc::fcntl(slave, libc::F_GETFD) };
    // should maybe handle error when flags is -1
    unsafe { libc::fcntl(slave, libc::F_SETFD, flags | libc::FD_CLOEXEC) };

    // close master when the program exits
    let flags = unsafe { libc::fcntl(master, libc::F_GETFD) };
    // should maybe handle error when flags is -1
    unsafe { libc::fcntl(master, libc::F_SETFD, flags | libc::FD_CLOEXEC) };

    let master = unsafe {File::from_raw_fd(master)};
    let slave = unsafe {File::from_raw_fd(slave)};
    return (master, slave);
}


