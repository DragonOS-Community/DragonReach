#![no_std]
#![no_main]
#![feature(slice_pattern)]
#![feature(fn_traits)]

#[cfg(target_os = "dragonos")]
extern crate drstd as std;

extern crate hashbrown;

mod contants;
mod error;
mod executor;
mod manager;
mod parse;
mod systemctl;
mod task;
mod time;
mod unit;

use std::{eprint, eprintln};

use std::string::ToString;
use std::vec::Vec;

use error::ErrorFormat;
use executor::Executor;
use manager::timer_manager::TimerManager;
use manager::Manager;
use parse::UnitParser;
use systemctl::listener::Systemctl;

pub struct FileDescriptor(usize);

const DRAGON_REACH_UNIT_DIR: &'static str = "/etc/reach/system/";

#[no_mangle]
fn main() {
    // 初始化
    Systemctl::init();

    let mut units_file_name = Vec::new();
    //读取目录里面的unit文件
    if let Ok(entries) = std::fs::read_dir(DRAGON_REACH_UNIT_DIR) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let filename = entry.file_name();
                        let filename = filename.to_str().unwrap();
                        units_file_name.push(filename.to_string());
                    }
                }
            }
        }
    }

    //启动服务
    for path in units_file_name {
        let id = match UnitParser::from_path(&path) {
            Ok(id) => id,
            Err(e) => {
                eprintln!("Err:{}", e.error_format());
                0
            }
        };

        if id != 0 {
            if let Err(e) = Executor::exec(id) {
                eprintln!("Err:{}", e.error_format());
            }
        }
    }

    // 启动完服务后进入主循环
    loop {
        // 检查各服务运行状态
        Manager::check_running_status();
        // 检查cmd进程状态
        Manager::check_cmd_proc();
        // 检查计时器任务
        TimerManager::check_timer();
        // 监听systemctl
        Systemctl::ctl_listen();
    }
}
