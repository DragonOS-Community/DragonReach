use std::fs::{self, File};
use std::io::Read;
use std::os::fd::FromRawFd;
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;

use crate::error::ErrorFormat;
use crate::manager::ctl_manager::CtlManager;

use super::ctl_parser::{CommandOperation, CtlParser, Pattern};
use super::{ctl_path, DRAGON_REACH_CTL_PIPE};

lazy_static! {
    static ref CTL_READER: Mutex<Arc<File>> = {
        let file = Systemctl::init_listener();
        Mutex::new(Arc::new(file))
    };
}

pub struct Command {
    pub(crate) operation: CommandOperation,
    pub(crate) args: Option<Vec<String>>,
    pub(crate) patterns: Vec<Pattern>,
}

impl Command {
    #[allow(dead_code)]
    fn new(op: CommandOperation, patterns: Vec<Pattern>, args: Option<Vec<String>>) -> Self {
        Command {
            operation: op,
            args,
            patterns: patterns,
        }
    }
}

impl Default for Command {
    fn default() -> Self {
        Command {
            operation: CommandOperation::None,
            args: None,
            patterns: Vec::new(),
        }
    }
}

pub struct Systemctl;

impl Systemctl {
    pub fn init() {
        Self::init_ctl_pipe();
    }

    pub fn init_listener() -> File {
        let fd = unsafe { libc::open(ctl_path().as_ptr(), libc::O_RDONLY | libc::O_NONBLOCK) };
        if fd < 0 {
            panic!("open ctl pipe error");
        }
        unsafe { File::from_raw_fd(fd) }
    }

    pub fn ctl_listen() {
        let mut guard = CTL_READER.lock().unwrap();
        let mut s = String::new();
        if let Ok(size) = guard.read_to_string(&mut s) {
            if size == 0 {
                return;
            }
            match CtlParser::parse_ctl(&s) {
                Ok(cmd) => {
                    let _ = CtlManager::exec_ctl(cmd);
                }
                Err(err) => {
                    eprintln!("parse tcl command error: {}", err.error_format());
                }
            }
        }
    }

    fn is_ctl_exists() -> bool {
        if let Ok(metadata) = fs::metadata(DRAGON_REACH_CTL_PIPE) {
            metadata.is_file()
        } else {
            false
        }
    }

    fn init_ctl_pipe() {
        if !Self::is_ctl_exists() {
            let path = ctl_path();
            let ret = unsafe { libc::mkfifo(path.as_ptr(), 0o666) };
            if ret != 0 {
                // 创建管道失败打日志
                panic!("create ctl pipe failed");
            }
        }
    }
}
