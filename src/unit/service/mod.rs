use super::{BaseUnit, Unit};
use crate::error::{ParseError, ParseErrorType};
use crate::parse::parse_service::ServiceParser;
use crate::parse::parse_util::UnitParseUtil;
use crate::parse::{Segment, SERVICE_UNIT_ATTR_TABLE};
use crate::task::cmdtask::CmdTask;

#[cfg(target_os = "dragonos")]
use drstd as std;

use std::rc::Rc;
use std::string::String;
use std::vec::Vec;
#[derive(Default)]
pub struct ServiceUnit {
    unit_base: BaseUnit,
    service_part: ServicePart,
}

#[derive(Debug)]
pub enum ServiceType {
    Simple,
    Forking,
    OneShot,
    Dbus,
    Notify,
    Idle,
}

impl Default for ServiceType {
    fn default() -> Self {
        ServiceType::Simple
    }
}

#[derive(Debug)]
pub enum RestartOption {
    AlwaysRestart,
    OnSuccess,
    OnFailure,
    OnAbnormal,
    OnAbort,
    OnWatchdog,
    None,
}

impl Default for RestartOption {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug)]
pub enum MountFlag {
    Shared,
    Slave,
    Private,
}

impl Default for MountFlag {
    fn default() -> Self {
        Self::Private
    }
}

#[derive(Default, Debug)]
pub struct ServicePart {
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
    restart_sec: u64,
    restart: RestartOption,
    timeout_start_sec: u64,
    timeout_stop_sec: u64,
    //上下文配置相关
    environment: Vec<String>,
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
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn from_path(path: &str) -> Result<Rc<Self>, ParseError>
    where
        Self: Sized,
    {
        return ServiceParser::parse(path);
    }

    fn set_attr(&mut self, segment: Segment, attr: &str, val: &str) -> Result<(), ParseError> {
        if segment != Segment::Service {
            return Err(ParseError::new(ParseErrorType::EINVAL, String::new(),0));
        }
        let attr_type = SERVICE_UNIT_ATTR_TABLE.get(attr).ok_or(ParseError::new(ParseErrorType::EINVAL, String::new(),0));
        return self.service_part.set_attr(attr_type.unwrap(), val);
    }

    fn set_unit_base(&mut self, base: BaseUnit) {
        self.unit_base = base;
    }

    fn unit_type(&self) -> super::UnitType {
        return self.unit_base.unit_type;
    }
}

impl ServiceUnit {
    pub fn unit_base(&self) -> &BaseUnit {
        return &self.unit_base;
    }

    pub fn service_part(&self) -> &ServicePart {
        return &self.service_part;
    }
}

pub enum ServiceUnitAttr {
    None,
    //Service段
    //定义启动时的进程行为
    Type,
    //
    RemainAfterExit,
    //启动命令
    ExecStart,
    //启动当前服务之前执行的命令
    ExecStartPre,
    //启动当前服务之后执行的命令
    ExecStartPos,
    //重启当前服务时执行的命令
    ExecReload,
    //停止当前服务时执行的命令
    ExecStop,
    //停止当其服务之后执行的命令
    ExecStopPost,
    //自动重启当前服务间隔的秒数
    RestartSec,
    //定义何种情况 Systemd 会自动重启当前服务
    Restart,
    //启动服务时等待的秒数
    TimeoutStartSec,
    //停止服务时的等待秒数，如果超过这个时间仍然没有停止，应该使用 SIGKILL 信号强行杀死服务的进程
    TimeoutStopSec,
    //为服务指定环境变量
    Environment,
    //指定加载一个包含服务所需的环境变量的列表的文件，文件中的每一行都是一个环境变量的定义
    EnvironmentFile,
    //服务的进程优先级，值越小优先级越高，默认为 0。其中 -20 为最高优先级，19 为最低优先级
    Nice,
    //指定服务的工作目录
    WorkingDirectory,
    //指定服务进程的根目录（/ 目录）。如果配置了这个参数，服务将无法访问指定目录以外的任何文件
    RootDirectory,
    //指定运行服务的用户
    User,
    //指定运行服务的用户组
    Group,
    //服务的 Mount Namespace 配置，会影响进程上下文中挂载点的信息
    MountFlags,
}

impl ServicePart {
    pub fn set_attr(&mut self, attr: &ServiceUnitAttr, val: &str) -> Result<(), ParseError> {
        match attr {
            ServiceUnitAttr::Type => match val {
                "simple" => self.service_type = ServiceType::Simple,
                "forking" => self.service_type = ServiceType::Forking,
                "oneshot" => self.service_type = ServiceType::OneShot,
                "dbus" => self.service_type = ServiceType::Dbus,
                "notify" => self.service_type = ServiceType::Notify,
                "idle" => self.service_type = ServiceType::Idle,
                _ => {
                    return Err(ParseError::new(ParseErrorType::EINVAL, String::new(),0));
                }
            },
            ServiceUnitAttr::RemainAfterExit => {
                self.remain_after_exit = UnitParseUtil::parse_boolean(val)?
            }
            ServiceUnitAttr::ExecStart => {
                self.exec_start.extend(UnitParseUtil::parse_cmd_task(val)?);
            }
            ServiceUnitAttr::ExecStartPre => {
                self.exec_start_pre
                    .extend(UnitParseUtil::parse_cmd_task(val)?);
            }
            ServiceUnitAttr::ExecStartPos => {
                self.exec_start_pos
                    .extend(UnitParseUtil::parse_cmd_task(val)?);
            }
            ServiceUnitAttr::ExecReload => {
                self.exec_reload.extend(UnitParseUtil::parse_cmd_task(val)?);
            }
            ServiceUnitAttr::ExecStopPost => {
                self.exec_stop_post
                    .extend(UnitParseUtil::parse_cmd_task(val)?);
            }
            ServiceUnitAttr::ExecStop => {
                self.exec_stop.extend(UnitParseUtil::parse_cmd_task(val)?);
            }
            ServiceUnitAttr::RestartSec => self.restart_sec = UnitParseUtil::parse_sec(val)?,
            ServiceUnitAttr::Restart => match val {
                "always" => self.restart = RestartOption::AlwaysRestart,
                "on-success" => self.restart = RestartOption::OnSuccess,
                "on-failure" => self.restart = RestartOption::OnFailure,
                "on-abnormal" => self.restart = RestartOption::OnAbnormal,
                "on-abort" => self.restart = RestartOption::OnAbort,
                "on-watchdog" => self.restart = RestartOption::OnWatchdog,
                _ => {
                    return Err(ParseError::new(ParseErrorType::EINVAL,String::new(),0));
                }
            },
            ServiceUnitAttr::TimeoutStartSec => {
                self.timeout_start_sec = UnitParseUtil::parse_sec(val)?
            }
            ServiceUnitAttr::TimeoutStopSec => {
                self.timeout_stop_sec = UnitParseUtil::parse_sec(val)?
            }
            ServiceUnitAttr::Environment => {
                self.environment.push(String::from(val));
            }
            ServiceUnitAttr::EnvironmentFile => {
                if !UnitParseUtil::is_valid_file(val) {
                    return Err(ParseError::new(ParseErrorType::EFILE,String::new(),0));
                }
                self.environment_file = String::from(val);
            }
            ServiceUnitAttr::Nice => {
                self.nice = UnitParseUtil::parse_nice(val)?;
            }
            ServiceUnitAttr::WorkingDirectory => {
                if !UnitParseUtil::is_dir(val) {
                    return Err(ParseError::new(ParseErrorType::ENODIR,String::new(),0));
                }
                self.working_directory = String::from(val);
            }
            ServiceUnitAttr::User => {
                //TODO: 检查系统是否存在这个用户
                self.user = String::from(val);
            }
            ServiceUnitAttr::Group => {
                //TODO: 检查系统是否存在该用户组
                self.group = String::from(val);
            }
            ServiceUnitAttr::MountFlags => match val {
                "shared" => self.mount_flags = MountFlag::Shared,
                "slave" => self.mount_flags = MountFlag::Slave,
                "private" => self.mount_flags = MountFlag::Private,
                _ => {
                    return Err(ParseError::new(ParseErrorType::EINVAL,String::new(),0));
                }
            },
            _ => {
                return Err(ParseError::new(ParseErrorType::EINVAL,String::new(),0));
            }
        }
        return Ok(());
    }

    // 生命周期相关
    pub fn service_type(&self) -> &ServiceType {
        &self.service_type
    }

    pub fn remain_after_exit(&self) -> bool {
        self.remain_after_exit
    }

    pub fn exec_start(&self) -> &Vec<CmdTask> {
        &self.exec_start
    }

    pub fn exec_start_pre(&self) -> &Vec<CmdTask> {
        &self.exec_start_pre
    }

    pub fn exec_start_pos(&self) -> &Vec<CmdTask> {
        &self.exec_start_pos
    }

    pub fn exec_reload(&self) -> &Vec<CmdTask> {
        &self.exec_reload
    }

    pub fn exec_stop(&self) -> &Vec<CmdTask> {
        &self.exec_stop
    }

    pub fn exec_stop_post(&self) -> &Vec<CmdTask> {
        &self.exec_stop_post
    }

    pub fn restart_sec(&self) -> u64 {
        self.restart_sec
    }

    pub fn restart(&self) -> &RestartOption {
        &self.restart
    }

    pub fn timeout_start_sec(&self) -> u64 {
        self.timeout_start_sec
    }

    pub fn timeout_stop_sec(&self) -> u64 {
        self.timeout_stop_sec
    }

    // 上下文配置相关
    pub fn environment(&self) -> &[String] {
        &self.environment
    }

    pub fn environment_file(&self) -> &str {
        &self.environment_file
    }

    pub fn nice(&self) -> i8 {
        self.nice
    }

    pub fn working_directory(&self) -> &str {
        &self.working_directory
    }

    pub fn root_directory(&self) -> &str {
        &self.root_directory
    }

    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn group(&self) -> &str {
        &self.group
    }

    pub fn mount_flags(&self) -> &MountFlag {
        &self.mount_flags
    }
}
