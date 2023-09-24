#![no_std]
#![no_main]
#![feature(slice_pattern)]

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_os = "dragonos")]{
        extern crate drstd;
        use drstd as std;
    }
}

extern crate hashbrown;

mod contants;
mod error;
mod executor;
mod manager;
mod parse;
mod task;
mod unit;

use crate::unit::service;
use std::eprint;
use std::eprintln;
use std::fs;
use std::print;
use std::println;
use std::string::{String, ToString};
use std::vec::Vec;
use unit::service::ServiceUnit;

use self::unit::Unit;
use error::ErrorFormat;

pub struct FileDescriptor(usize);

const DRAGON_REACH_UNIT_DIR: &'static str = "/etc/reach/system/";

#[cfg(target_os = "dragonos")]
#[no_mangle]
fn main() {
    use parse::UnitParser;

    use crate::{
        executor::Executor,
        manager::{Manager, UnitManager},
        parse::parse_util::UnitParseUtil,
    };

    let mut units_file_name = Vec::new();
    //读取目录里面的unit文件
    if let Ok(entries) = fs::read_dir(DRAGON_REACH_UNIT_DIR) {
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
            let unit = UnitManager::get_unit_with_id(&id).unwrap();
            if let Err(e) = Executor::exec(&unit) {
                eprintln!("Err:{}", e.error_format());
            } else {
                println!("Service {} startup success...", id);
            }
        }
    }

    // 启动完服务后进入主循环
    loop {
        // 检查各服务运行状态
        Manager::check_running_status();
    }
}

#[cfg(not(target_os = "dragonos"))]
fn main() {
    use std::time::Instant;

    use parse::UnitParser;

    use crate::{
        executor::Executor,
        manager::{Manager, UnitManager},
        parse::parse_util::UnitParseUtil,
    };

    let mut units_file_name = Vec::new();
    //读取目录里面的unit文件
    if let Ok(entries) = fs::read_dir("/bin") {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let filename = entry.file_name();
                        let filename = filename.to_str().unwrap();
                        //units_file_name.push(filename.to_string());
                    }
                }
            }
        }
    }
  
    units_file_name.push("/home/heyicong/DragonReach/parse_test/test.service".to_string());

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
            let unit = UnitManager::get_unit_with_id(&id).unwrap();
            if let Err(e) = Executor::exec(&unit) {
                eprintln!("Err:{}", e.error_format());
            }
        }
    }

    // 启动完服务后进入主循环
    loop {
        let time = Instant::now();

        // 检查各服务运行状态
        Manager::check_running_status();
        //println!(".");

        let t = time.elapsed().as_micros();
        //println!("{}",t);
    }
}
