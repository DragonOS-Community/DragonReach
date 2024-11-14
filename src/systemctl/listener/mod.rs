use super::ctl_parser::{CommandOperation, CtlParser, Pattern};
use super::{ctl_path, DRAGON_REACH_CTL_PIPE};
use crate::error::ErrorFormat;
use crate::manager::ctl_manager::CtlManager;
use lazy_static::lazy_static;
use std::fs::{self, File};
use std::io::Read;
use std::os::fd::FromRawFd;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

lazy_static! {
    static ref CTL_READER: Mutex<Arc<File>> = {
        let file = Systemctl::init_listener();
        Mutex::new(Arc::new(file))
    };
}
#[derive(Debug)]
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
    /// # 初始化系统服务控制 - 初始化系统服务控制管道
    pub fn init() {
        Self::init_ctl_pipe();
    }
    /// # 初始化监听器 - 初始化系统服务控制命令监听器
    ///
    /// 打开系统服务控制命令的管道文件描述符，并设置为非阻塞模式。
    ///
    pub fn init_listener() -> File {
        let fd = unsafe { libc::open(ctl_path().as_ptr(), libc::O_RDONLY | libc::O_NONBLOCK) };
        if fd < 0 {
            panic!("open ctl pipe error");
        }
        unsafe { File::from_raw_fd(fd) }
    }
    /// # 监听控制命令 - 监听系统服务控制命令
    ///
    /// 持续从系统服务控制管道中读取命令。
    ///
    pub fn ctl_listen() {
        println!("ctl listen");
        let mut guard = CTL_READER.lock().unwrap();
        let mut s = String::new();
        loop {
            s.clear();
            match guard.read_to_string(&mut s) {
                Ok(size) if size > 0 => match CtlParser::parse_ctl(&s) {
                    Ok(cmd) => {
                        let _ = CtlManager::exec_ctl(cmd);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse command: {}", e.error_format());
                    }
                },
                Ok(_) => {
                    // 如果读取到的大小为0，说明没有数据可读，适当休眠
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    eprintln!("Failed to read from pipe: {}", e);
                    break;
                }
            }
        }
    }

    /// # 检查控制管道是否存在 - 检查系统服务控制管道文件是否存在
    ///
    /// 返回管道文件是否存在。
    ///
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
                panic!("create ctl pipe failed, err: {ret}");
            }
        }
    }
}
