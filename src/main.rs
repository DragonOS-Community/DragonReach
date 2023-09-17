// #![no_std]
// #![no_main]

//extern crate drstd;
extern crate hashbrown;

//use drstd as std;
use std::print;
use std::println;
use std::rc::Rc;
use unit::service::ServiceUnit;

mod contants;
mod error;
mod parse;
mod task;
mod unit;

use crate::unit::service;

use self::unit::Unit;

pub struct FileDescriptor(usize);

//#[no_mangle]
fn main() {
    let service =
        ServiceUnit::from_path("/home/heyicong/DragonReach/parse_test/test.service").unwrap();
    let service = service.as_ref();
    let cmds = &service.service_part.exec_start;
    for cmd in cmds{
        println!("{},{}",cmd.path,cmd.cmd);
    }
}
