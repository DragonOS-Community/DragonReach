#![no_std]
#![no_main]

#[cfg(target_os = "dragonos")]
extern crate drstd as std;

use std::fs::File;
use std::libc::c_str::CString;
use std::os::fd::FromRawFd;
use std::{env, libc};
use std::{string::String, vec::Vec};

const REACH_PIPE_PATH: &str = "/etc/reach/ipc/ctl";

#[cfg(target_os = "dragonos")]
#[no_mangle]
fn main() {
    use std::{eprint, eprintln, format, io::Write};

    let mut args: Vec<String> = env::args().collect();

    if args.len() <= 2 {
        args = Vec::new();
        args.push(String::from("list-units"));
    } else {
        args.remove(0);
        args.remove(0);
    }

    let mut msg = String::new();
    for arg in args {
        msg = format!("{} {}", msg, arg);
    }

    if let Err(err) = get_writer().write_all(msg.as_bytes()) {
        eprintln!("write error {}", err);
    }
}

#[cfg(not(target_os = "dragonos"))]
fn main() {}

fn get_writer() -> File {
    let fd = unsafe { libc::open(ctl_path().as_ptr(), libc::O_WRONLY) };

    unsafe { File::from_raw_fd(fd) }
}

#[allow(dead_code)]
fn get_reader() -> File {
    let fd = unsafe { libc::open(ctl_path().as_ptr(), libc::O_RDONLY) };

    unsafe { File::from_raw_fd(fd) }
}

fn ctl_path() -> CString {
    CString::new(REACH_PIPE_PATH).expect("create pipe path CString error")
}
