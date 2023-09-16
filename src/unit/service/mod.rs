use super::{Unit, UnitPart, BaseUnit};
use drstd::std as std;
use std::string::String;
use std::vec::Vec;
use crate::error::SystemError;

use crate::unit::CmdTask;

struct ServiceUnit{
    unit_base: BaseUnit,
    service_part: ServicePart
}

enum ServiceType {
    Simple,
    Forking,
    OneShot,
    Dbus,
    Notify,
    Idle
}

enum RestartOption {
    AlwaysRestart,
    OnSuccess,
    OnFailure,
    OnAbnormal,
    OnAbort,
    OnWatchdog
}

enum MountFlag {
    Shared,
    Slave,
    Private
}

struct ServicePart {
    //生命周期相关
    service_type: ServiceType,
    /// 
    remain_after_exit: bool,
    exec_start: Vec<CmdTask>,
    exec_start_pre: Vec<CmdTask>,
    exec_start_pos: Vec<CmdTask>,
    exec_reload: Vec<CmdTask>,
    exec_stop: Vec<CmdTask>,
    exec_stop_post: Vec<CmdTask>,
    restart_sec: u32,
    restart: RestartOption,
    timeout_start_sec: u32,
    timeout_stop_sec: u32,
    //上下文配置相关
    environment: String,
    environment_file: String,
    nice: i8,
    working_directory: String,
    root_directory: String,
    user: String,
    group: String,
    mount_flags: MountFlag,
    //LimitCPU / LimitSTACK / LimitNOFILE / LimitNPROC 等,后续支持再添加
}

impl Unit for ServiceUnit {
    fn parse(path: &str) -> Self {
        
    }
}