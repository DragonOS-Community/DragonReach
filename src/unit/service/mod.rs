use super::{BaseUnit, Unit};
use crate::error::ParseError;
use crate::parse::parse_service::ServiceParser;
use crate::parse::{AttrParse, Segment};
use crate::task::cmdtask::CmdTask;
//use drstd as std;
use std::rc::Rc;
use std::string::String;
use std::vec::Vec;
#[derive(Default)]
pub struct ServiceUnit {
    pub unit_base: BaseUnit,
    pub service_part: ServicePart,
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
    pub service_type: ServiceType,
    ///
    pub remain_after_exit: bool,
    pub exec_start: Vec<CmdTask>,
    pub exec_start_pre: Vec<CmdTask>,
    pub exec_start_pos: Vec<CmdTask>,
    pub exec_reload: Vec<CmdTask>,
    pub exec_stop: Vec<CmdTask>,
    pub exec_stop_post: Vec<CmdTask>,
    pub restart_sec: u64,
    pub restart: RestartOption,
    pub timeout_start_sec: u64,
    pub timeout_stop_sec: u64,
    //上下文配置相关
    pub environment: String,
    pub environment_file: String,
    pub nice: i8,
    pub working_directory: String,
    pub root_directory: String,
    pub user: String,
    pub group: String,
    pub mount_flags: MountFlag,
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
            return Err(ParseError::EINVAL);
        }
        return ServiceParser::parse_and_set_attribute(self, attr, val);
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
