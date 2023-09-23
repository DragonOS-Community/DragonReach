#[cfg(target_os = "dragonos")]
use drstd as std;

use crate::{
    error::{
        runtime_error::{RuntimeError, RuntimeErrorType},
        ErrorFormat,
    },
    manager::{RunningUnit, UnitManager},
    parse::{parse_util::UnitParseUtil, Segment, UnitParser},
    task::cmdtask::CmdTask,
    unit::{
        service::RestartOption,
        service::{ServiceType, ServiceUnit},
        Unit, UnitState, UnitType,
    },
};
use std::os::unix::process::CommandExt;
use std::sync::Mutex;
use std::vec::Vec;
use std::{
    borrow::BorrowMut,
    cell::RefCell,
    eprint, eprintln,
    io::{Error, ErrorKind},
    print, println,
};
use std::{io::BufRead, process::Command, sync::Arc};
use std::{process::Stdio, string::ToString};

use super::ExitStatus;

pub struct ServiceExecutor;

impl ServiceExecutor {
    pub fn exec(service: &mut ServiceUnit) -> Result<(), RuntimeError> {
        match *service.service_part().service_type() {
            ServiceType::Simple => {
                return Self::exec_simple(service);
            }
            ServiceType::Forking => {
                return Self::exec_forking(service);
            }
            ServiceType::Dbus => {
                return Self::exec_dbus(service);
            }
            ServiceType::Notify => {
                return Self::exec_notify(service);
            }
            ServiceType::Idle => {
                return Self::exec_idle(service);
            }
            ServiceType::OneShot => {
                return Self::exec_one_shot(service);
            }
        }
    }
    pub fn exec_simple(service: &mut ServiceUnit) -> Result<(), RuntimeError> {
        //处理conflict
        let conflicts = service.unit_base().unit_part().conflicts();
        for u in conflicts {
            // 如果有冲突项enable的时候，该unit不能启动
            let mutex = UnitManager::get_unit_with_id(u).unwrap();
            let unit = mutex.lock().unwrap();
            if *unit.unit_base().state() == UnitState::Enabled {
                eprintln!(
                    "{}: Service startup failed: conflict unit",
                    unit.unit_base().unit_part().description()
                );
                return Err(RuntimeError::new(RuntimeErrorType::ExecFailed));
            }
        }

        //获取环境变量
        //先获取指定的环境变量
        let mut envs = Vec::from(service.service_part().environment());

        //若指定了环境变量文件，则解析环境变量文件
        let env_file = service.service_part().environment_file();
        if env_file.len() > 0 {
            let env_reader = match UnitParser::get_reader(env_file, UnitType::Unknown) {
                Ok(reader) => reader,
                Err(_) => {
                    return Err(RuntimeError::new(RuntimeErrorType::Custom(
                        "Incorrect environment variable configuration file".to_string(),
                    )));
                }
            };
            for line in env_reader.lines() {
                if let Ok(line) = line {
                    let x = match UnitParseUtil::parse_env(line.as_str()) {
                        Ok(v) => v,
                        Err(_) => {
                            return Err(RuntimeError::new(RuntimeErrorType::Custom(
                                "Failed to parse environment variable configuration file"
                                    .to_string(),
                            )));
                        }
                    };
                    envs.push(x);
                }
            }
        }

        //服务配置环境变量，配置工作目录
        //获取工作目录
        let mut dir = service.service_part().working_directory();
        if dir.is_empty() {
            dir = "/";
        }
        //获取启动命令
        let exec_start = service.service_part().exec_start();
        println!("exec:{}", exec_start.path);
        //处理ExecStartsPre,准备在服务启动前执行的命令
        //TODO:设置uid与gid
        let cmds = service.service_part().exec_start_pre().clone();
        let proc = unsafe {
            Command::new(&exec_start.path)
                .args(&exec_start.cmd)
                .current_dir(dir)
                .envs(envs)
                .stderr(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stdin(Stdio::inherit())
                .pre_exec(move || {
                    for cmdtask in cmds.clone() {
                        match cmdtask.exec() {
                            Ok(_) => (),
                            Err(e) => {
                                eprintln!("{}", e.error_format());
                                return Err(Error::new(
                                    ErrorKind::Interrupted,
                                    "ExecStartPreFailed",
                                ));
                            }
                        };
                    }
                    Ok(())
                })
                .spawn()
        };

        match proc {
            Ok(p) => {
                println!("Service running...");
                //修改service状态
                service.mut_unit_base().set_state(UnitState::Enabled);
                //启动成功后将Child加入全局管理的进程表
                UnitManager::push_running(RunningUnit::new(p, service.unit_id()));
                //执行启动后命令
                Self::exec_start_pos(service)?;
            }
            Err(err) => {
                eprintln!("{}: Service startup failed: {}", exec_start.path, err);
                return Err(RuntimeError::new(RuntimeErrorType::ExecFailed));
            }
        }
        Ok(())
    }

    fn exec_dbus(service: &ServiceUnit) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn exec_forking(service: &ServiceUnit) -> Result<(), RuntimeError> {
        Ok(())
    }

    // 此方法会改变service的启动模式为simple
    fn exec_idle(service: &mut ServiceUnit) -> Result<(), RuntimeError> {
        // 将该service加入等待运行队列
        let _ = service.set_attr(Segment::Service, "Type", "simple");
        UnitManager::push_a_idle_service(service.unit_id());
        Ok(())
    }

    fn exec_notify(service: &ServiceUnit) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn exec_one_shot(service: &ServiceUnit) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn exec_start_pos(service: &ServiceUnit) -> Result<(), RuntimeError> {
        let cmds = service.service_part().exec_start_pos();
        for cmd in cmds {
            cmd.exec()?;
        }
        Ok(())
    }

    //显式停止时执行的命令
    fn exec_stop(service: &mut ServiceUnit) -> Result<(), RuntimeError> {
        let cmds = service.service_part().exec_stop();
        for cmd in cmds {
            cmd.exec()?;
        }
        Ok(())
    }

    //停止后执行的命令
    fn exec_stop_post(service: &mut ServiceUnit) -> Result<(), RuntimeError> {
        let cmds = service.service_part().exec_stop_post();
        for cmd in cmds {
            cmd.exec()?;
        }
        Ok(())
    }

    //服务退出执行的逻辑(包括自然退出及显式退出)
    pub fn after_exit(service: &mut ServiceUnit, exit_status: ExitStatus) {
        //TODO: 需要考虑是否需要在此处执行退出后代码，还是只需要显式退出时才执行
        let _ = Self::exec_stop_post(service);

        //判断是否需要restart，需要则再次启动服务
        if service.service_part().restart().is_restart(&exit_status) {
            let _ = service.run();
            return;
        }

        //如果该进程标记了RemainAfterExit，则将其加入特殊标记表
        if service.service_part().remain_after_exit() {
            UnitManager::push_flag_running(service.unit_id());
        }
    }
}
