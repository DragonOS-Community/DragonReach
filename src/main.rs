#![no_std]
#![no_main]
use cfg_if::cfg_if;

cfg_if!{
    if #[cfg(target_os = "dragonos")]{
        extern crate drstd;
        use drstd as std;
        use std::print;
        use std::println;
        use std::rc::Rc;
        use unit::service::ServiceUnit;
    }
}

extern crate hashbrown;

mod contants;
mod error;
mod parse;
mod task;
mod unit;

use crate::unit::service;

use self::unit::Unit;

pub struct FileDescriptor(usize);

#[cfg(target_os = "dragonos")]
#[no_mangle]
fn main() {
    use unit::service::ServiceUnit;

    let service = match ServiceUnit::from_path("/bin/test.service"){
        Ok(service) => service,
        Err(e) => {
            println!("Error:{}",e.error_format());
            return;
        }
    };
    let service = service.as_ref();
    println!("parse_result:");
    println!("Description:{:?}", service.unit_base().unit_part().description());
    println!("Documentation:{:?}",service.unit_base().unit_part().documentation());
    println!("ServiceType:{:?}",service.service_part().service_type());
    println!("ExecStrat:{:?}",service.service_part().exec_start());
    println!("WorkingDirectory:{:?}",service.service_part().working_directory());
    println!("Environment:{:?}",service.service_part().environment());
    println!("Restart:{:?}",service.service_part().restart());
    println!("RestartSec:{:?}",service.service_part().restart_sec());
    println!("User:{:?}",service.service_part().user());
    println!("Group:{:?}",service.service_part().group());
    println!("TimeoutStartSec:{:?}",service.service_part().timeout_start_sec());
    println!("TimeoutStopSec:{:?}",service.service_part().timeout_stop_sec());
}

#[cfg(not(target_os = "dragonos"))]
fn main() {
    use unit::service::ServiceUnit;

    let service = match ServiceUnit::from_path("/home/heyicong/DragonReach/parse_test/test.service"){
        Ok(service) => service,
        Err(e) => {
            println!("Error:{}",e.error_format());
            return;
        }
    };

    
    let service = service.as_ref();
    println!("parse_result:");
    println!("Description:{:?}", service.unit_base().unit_part().description());
    println!("Documentation:{:?}",service.unit_base().unit_part().documentation());
    println!("ServiceType:{:?}",service.service_part().service_type());
    println!("ExecStrat:{:?}",service.service_part().exec_start());
    println!("WorkingDirectory:{:?}",service.service_part().working_directory());
    println!("Environment:{:?}",service.service_part().environment());
    println!("Restart:{:?}",service.service_part().restart());
    println!("RestartSec:{:?}",service.service_part().restart_sec());
    println!("User:{:?}",service.service_part().user());
    println!("Group:{:?}",service.service_part().group());
    println!("TimeoutStartSec:{:?}",service.service_part().timeout_start_sec());
    println!("TimeoutStopSec:{:?}",service.service_part().timeout_stop_sec());
}
