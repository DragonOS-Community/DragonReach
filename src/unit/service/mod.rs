use super::{BaseUnit, Unit};
use crate::error::runtime_error::RuntimeError;
use crate::error::{parse_error::ParseError, parse_error::ParseErrorType};
use crate::executor::service_executor::ServiceExecutor;
use crate::executor::ExitStatus;

use crate::manager::timer_manager::TimerManager;
use crate::parse::parse_service::ServiceParser;
use crate::parse::parse_util::UnitParseUtil;
use crate::parse::{Segment, SERVICE_UNIT_ATTR_TABLE};
use crate::task::cmdtask::CmdTask;

#[derive(Clone, Debug)]
pub struct ServiceUnit {
    unit_base: BaseUnit,
    service_part: ServicePart,
}

impl Default for ServiceUnit {
    fn default() -> Self {
        let mut sp = ServicePart::default();
        sp.working_directory = String::from("/");
        Self {
            unit_base: BaseUnit::default(),
            service_part: sp,
        }
    }
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RestartOption {
    //ServiceRestart
    AlwaysRestart, //总是重启
    OnSuccess,     //在该服务正常退出时
    OnFailure,     //在该服务启动失败时
    OnAbnormal,    //在该服务以非0错误码退出时
    OnAbort,       //在该服务显示退出时(通过DragonReach手动退出)
    OnWatchdog,    //定时观测进程无响应时(当前未实现)
    None,          //不重启
}

impl Default for RestartOption {
    fn default() -> Self {
        Self::None
    }
}

impl RestartOption {
    pub fn is_restart(&self, exit_status: &ExitStatus) -> bool {
        if *self == Self::AlwaysRestart {
            return true;
        }

        match (*self, *exit_status) {
            (Self::OnSuccess, ExitStatus::Success) => {
                return true;
            }
            (Self::OnAbnormal, ExitStatus::Abnormal) => {
                return true;
            }
            (Self::OnAbort, ExitStatus::Abort) => {
                return true;
            }
            (Self::OnFailure, ExitStatus::Failure) => {
                return true;
            }
            (Self::OnWatchdog, ExitStatus::Watchdog) => {
                return true;
            }
            _ => {
                return false;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
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

#[allow(dead_code)]
#[derive(Default, Debug, Clone)]
pub struct ServicePart {
    //生命周期相关
    service_type: ServiceType,
    ///
    remain_after_exit: bool,
    exec_start: CmdTask,
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
    environment: Vec<(String, String)>,
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

    fn from_path(path: &str) -> Result<usize, ParseError>
    where
        Self: Sized,
    {
        return ServiceParser::parse(path);
    }

    fn set_attr(&mut self, segment: Segment, attr: &str, val: &str) -> Result<(), ParseError> {
        if segment != Segment::Service {
            return Err(ParseError::new(ParseErrorType::EINVAL, String::new(), 0));
        }
        let attr_type = SERVICE_UNIT_ATTR_TABLE.get(attr).ok_or(ParseError::new(
            ParseErrorType::EINVAL,
            String::new(),
            0,
        ));
        return self.service_part.set_attr(attr_type.unwrap(), val);
    }

    fn set_unit_base(&mut self, base: BaseUnit) {
        self.unit_base = base;
    }

    fn unit_type(&self) -> super::UnitType {
        return self.unit_base.unit_type;
    }

    fn unit_base(&self) -> &BaseUnit {
        return &self.unit_base;
    }

    fn unit_id(&self) -> usize {
        return self.unit_base.unit_id;
    }

    fn run(&mut self) -> Result<(), RuntimeError> {
        self.exec()
    }

    fn unit_base_mut(&mut self) -> &mut BaseUnit {
        return &mut self.unit_base;
    }

    fn after_exit(&mut self, exit_status: ExitStatus) {
        ServiceExecutor::after_exit(self, exit_status);
    }

    fn init(&mut self) {
        let part = &mut self.service_part;
        for cmd in part.exec_reload.iter_mut() {
            cmd.dir = part.working_directory.to_string();
            cmd.envs = part.environment.clone();
        }
        part.exec_start.dir = part.working_directory.to_string();
        part.exec_start.envs = part.environment.clone();
        for cmd in part.exec_start_pos.iter_mut() {
            cmd.dir = part.working_directory.to_string();
            cmd.envs = part.environment.clone();
        }
        for cmd in part.exec_start_pre.iter_mut() {
            cmd.dir = part.working_directory.to_string();
            cmd.envs = part.environment.clone();
        }
        for cmd in part.exec_stop.iter_mut() {
            cmd.dir = part.working_directory.to_string();
            cmd.envs = part.environment.clone();
        }
        for cmd in part.exec_stop_post.iter_mut() {
            cmd.dir = part.working_directory.to_string();
            cmd.envs = part.environment.clone();
        }
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn exit(&mut self) {
        ServiceExecutor::exit(self);
        //改变计时器内部状态
    }

    fn restart(&mut self) -> Result<(), RuntimeError> {
        return ServiceExecutor::restart(self);
    }
}

impl ServiceUnit {
    pub fn unit_base(&self) -> &BaseUnit {
        return &self.unit_base;
    }

    pub fn service_part(&self) -> &ServicePart {
        return &self.service_part;
    }

    pub fn mut_service_part(&mut self) -> &mut ServicePart {
        return &mut self.service_part;
    }

    fn exec(&mut self) -> Result<(), RuntimeError> {
        let _ = ServiceExecutor::exec(self);
        Ok(())
    }
}

unsafe impl Sync for ServiceUnit {}

unsafe impl Send for ServiceUnit {}

pub enum ServiceUnitAttr {
    //ServiceExecCommand+
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

#[allow(dead_code)]
impl ServicePart {
    pub fn set_attr(&'_ mut self, attr: &ServiceUnitAttr, val: &str) -> Result<(), ParseError> {
        match attr {
            ServiceUnitAttr::Type => match val {
                "simple" => self.service_type = ServiceType::Simple,
                "forking" => self.service_type = ServiceType::Forking,
                "oneshot" => self.service_type = ServiceType::OneShot,
                "dbus" => self.service_type = ServiceType::Dbus,
                "notify" => self.service_type = ServiceType::Notify,
                "idle" => self.service_type = ServiceType::Idle,
                _ => {
                    return Err(ParseError::new(ParseErrorType::EINVAL, String::new(), 0));
                }
            },
            ServiceUnitAttr::RemainAfterExit => {
                self.remain_after_exit = UnitParseUtil::parse_boolean(val)?
            }
            ServiceUnitAttr::ExecStart => {
                self.exec_start = UnitParseUtil::parse_cmd_task(val)?[0].clone();
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
                    return Err(ParseError::new(ParseErrorType::EINVAL, String::new(), 0));
                }
            },
            ServiceUnitAttr::TimeoutStartSec => {
                self.timeout_start_sec = UnitParseUtil::parse_sec(val)?
            }
            ServiceUnitAttr::TimeoutStopSec => {
                self.timeout_stop_sec = UnitParseUtil::parse_sec(val)?
            }
            ServiceUnitAttr::Environment => {
                self.environment.push(UnitParseUtil::parse_env(val)?);
            }
            ServiceUnitAttr::EnvironmentFile => {
                if !UnitParseUtil::is_valid_file(val) {
                    return Err(ParseError::new(ParseErrorType::EFILE, String::new(), 0));
                }
                self.environment
                    .extend(UnitParseUtil::parse_environment_file(val)?);
            }
            ServiceUnitAttr::Nice => {
                self.nice = UnitParseUtil::parse_nice(val)?;
            }
            ServiceUnitAttr::WorkingDirectory => {
                if !UnitParseUtil::is_dir(val) {
                    return Err(ParseError::new(ParseErrorType::ENODIR, String::new(), 0));
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
                    return Err(ParseError::new(ParseErrorType::EINVAL, String::new(), 0));
                }
            },
            _ => {
                return Err(ParseError::new(ParseErrorType::EINVAL, String::new(), 0));
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

    pub fn exec_start(&self) -> &CmdTask {
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

    pub fn exec_stop_post(&mut self) -> &mut Vec<CmdTask> {
        &mut self.exec_stop_post
    }

    pub fn mut_exec_start_pre(&mut self) -> &mut Vec<CmdTask> {
        &mut self.exec_start_pre
    }

    pub fn mut_exec_start_pos(&mut self) -> &mut Vec<CmdTask> {
        &mut self.exec_start_pos
    }

    pub fn mut_exec_reload(&mut self) -> &mut Vec<CmdTask> {
        &mut self.exec_reload
    }

    pub fn mut_exec_stop(&mut self) -> &mut Vec<CmdTask> {
        &mut self.exec_stop
    }

    pub fn mut_exec_stop_post(&mut self) -> &mut Vec<CmdTask> {
        &mut self.exec_stop_post
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
    pub fn environment(&self) -> &[(String, String)] {
        &self.environment
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
