mod contants;
mod error;
mod executor;
mod manager;
mod parse;
mod systemctl;
mod task;
mod time;
mod unit;

use error::ErrorFormat;
use manager::{timer_manager::TimerManager, Manager};
use parse::UnitParser;
use systemctl::listener::Systemctl;

use crate::{executor::Executor, time::timer::Timer};

pub struct FileDescriptor(usize);

//const DRAGON_REACH_UNIT_DIR: &'static str = "/etc/reach/system/";
const DRAGON_REACH_UNIT_DIR: &'static str = "/home/fz/testSystemd/";

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
                        let filename = entry.file_name().to_str().unwrap().to_string();
                        units_file_name.push(filename);
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
        if id != 0 && TimerManager::is_timer(&id){
            if let Err(e) = Executor::exec(id) {
                eprintln!("Err:{}", e.error_format());
            }
        }
        println!("Parse {} success!", path);
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
