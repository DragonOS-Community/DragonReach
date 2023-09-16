use drstd::std as std;
use std::string::String;
use std::vec::Vec;
use std::boxed::Box;
use crate::error::SystemError;

mod service;
mod target;

use self::target::TargetUnit;

//所有可解析的Unit都应该实现该trait
pub trait Unit {
    fn parse(path: &str) -> Self where Self: Sized;
}

enum UnitState {
    Enabled,
    Disabled,
    Static,
    Masked
}

enum UnitType {
    Automount,
    Device,
    Mount,
    Path,
    Scope,
    Service,
    Slice,
    Snapshot,
    Socket,
    Swap,
    Target,
    Timer,
}

//记录unit文件基本信息
struct BaseUnit {
    unit_part: UnitPart,
    install_part: InstallPart,
    state: UnitState,
    unit_type: UnitType
}

// impl Default for BaseUnit {
//     fn default() -> Self {
//         // BaseUnit { 
//         //     unit_part: UnitPart::default(), 
//         //     install_part: InstallPart::default(), 
//         //     state: , unit_type: () }
//     }
// }

struct Url {
    protocol: String,
    host: String,
    port: u16,
    path: String,
    query: Option<String>,
    fragment: Option<String>,
}

impl Url {
    fn new(url_string: &str) -> Result<Self, SystemError> {
        // 解析 URL 字符串并初始化结构体字段
        // ...

        // 验证 URL 合法性
        // ...

        // 返回 URL 结构体实例
        // ...
        Err(SystemError::EINVAL)
    }

    fn build(&self) -> String {
        // 构建 URL 字符串
        // ...
        String::new()
    }
}

struct UnitPart {
    description: String,
    documentation: Vec<Url>,
    requires: Vec<Box<dyn Unit>>,
    wants: Vec<Box<dyn Unit>>,
    after: Vec<Box<dyn Unit>>,
    before: Vec<Box<dyn Unit>>,
    binds_to: Vec<Box<dyn Unit>>,
    part_of: Vec<Box<dyn Unit>>,
    on_failure: Vec<Box<dyn Unit>>,
    conflicts: Vec<Box<dyn Unit>>
}

impl Default for UnitPart {
    fn default() -> Self {
        UnitPart { 
            description: String::new(), 
            documentation: Vec::new(), 
            requires: Vec::new(), 
            wants: Vec::new(), 
            after: Vec::new(), 
            before: Vec::new(), 
            binds_to: Vec::new(), 
            part_of: Vec::new(), 
            on_failure: Vec::new(), 
            conflicts: Vec::new() 
        }
    }
}

struct InstallPart {
    wanted_by: Vec<TargetUnit>,
    requires_by: Vec<TargetUnit>,
    also: Vec<Box<dyn Unit>>,
    alias: String
}

impl Default for InstallPart {
    fn default() -> Self {
        InstallPart { 
            wanted_by: Vec::new(), 
            requires_by: Vec::new(), 
            also: Vec::new(), 
            alias: String::new() 
        }
    }
}

struct CmdTask{
    path: String,
    cmd: String
}