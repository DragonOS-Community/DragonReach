use std::{
    process::{Command, Stdio},
    time::Duration,
};

use crate::{
    error::runtime_error::{RuntimeError, RuntimeErrorType},
    manager::{timer_manager::TimerManager, UnitManager},
    parse::Segment,
    unit::{
        service::{ServiceType, ServiceUnit},
        Unit, UnitState,
    },
};

use super::{Executor, ExitStatus};

pub struct ServiceExecutor;

impl ServiceExecutor {
    /// ## Service执行器
    pub fn exec(service: &mut ServiceUnit) -> Result<(), RuntimeError> {
        // 通过服务启动类型分发
        match *service.service_part().service_type() {
            ServiceType::Simple => return Self::exec_simple(service),
            ServiceType::Forking => return Self::exec_forking(service),
            ServiceType::Dbus => return Self::exec_dbus(service),
            ServiceType::Notify => return Self::exec_notify(service),
            ServiceType::Idle => return Self::exec_idle(service),
            ServiceType::OneShot => return Self::exec_one_shot(service),
        };
    }

    pub fn exec_simple(service: &mut ServiceUnit) -> Result<(), RuntimeError> {
        //处理conflict
        let conflicts = service.unit_base().unit_part().conflicts();
        for u in conflicts {
            // 如果有冲突项enable的时候，该unit不能启动
            let mutex = UnitManager::get_unit_with_id(u).unwrap();
            let unit = mutex.lock().unwrap();
            if *unit.unit_base().state() == UnitState::Active {
                eprintln!(
                    "{}: Service startup failed: conflict unit",
                    unit.unit_base().unit_part().description()
                );
                return Err(RuntimeError::new(RuntimeErrorType::ExecFailed));
            }
        }

        //获取启动命令
        let exec_start = service.service_part().exec_start();

        //TODO:设置uid与gid

        //处理ExecStartsPre,准备在服务启动前执行的命令
        Self::exec_start_pre(service)?;

        //创建服务进程
        //服务配置环境变量，配置工作目录
        let proc = Command::new(&exec_start.path)
            .args(&exec_start.cmd)
            .current_dir(service.service_part().working_directory())
            .envs(Vec::from(service.service_part().environment()))
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stdin(Stdio::inherit())
            .spawn();

        match proc {
            Ok(p) => {
                // TODO: 打日志
                //修改service状态
                service.unit_base_mut().set_state(UnitState::Active);
                //启动成功后将Child加入全局管理的进程表
                UnitManager::push_running(service.unit_id(), p);
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

    fn exec_dbus(_service: &ServiceUnit) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn exec_forking(_service: &ServiceUnit) -> Result<(), RuntimeError> {
        Ok(())
    }

    // 此方法会改变service的启动模式为simple
    fn exec_idle(service: &mut ServiceUnit) -> Result<(), RuntimeError> {
        // 将该service加入等待运行队列
        let _ = service.set_attr(Segment::Service, "Type", "simple");
        UnitManager::push_a_idle_service(service.unit_id());
        Ok(())
    }

    fn exec_notify(_service: &ServiceUnit) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn exec_one_shot(_service: &ServiceUnit) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn exec_start_pos(service: &ServiceUnit) -> Result<(), RuntimeError> {
        let cmds = service.service_part().exec_start_pos();
        for cmd in cmds {
            cmd.spawn()?;
        }
        Ok(())
    }

    fn exec_start_pre(service: &ServiceUnit) -> Result<(), RuntimeError> {
        let cmds = service.service_part().exec_start_pre();
        for cmd in cmds {
            cmd.no_spawn()?;
        }
        Ok(())
    }

    //显式停止时执行的命令
    fn exec_stop(service: &mut ServiceUnit) -> Result<(), RuntimeError> {
        let cmds = service.service_part().exec_stop();
        for cmd in cmds {
            cmd.no_spawn()?;
        }
        Ok(())
    }

    //停止后执行的命令
    fn exec_stop_post(service: &mut ServiceUnit) -> Result<(), RuntimeError> {
        let cmds = service.mut_service_part().mut_exec_stop_post();
        for cmd in cmds {
            cmd.no_spawn()?;
        }
        Ok(())
    }

    fn exec_reload(service: &mut ServiceUnit) -> Result<(), RuntimeError> {
        let cmds = service.service_part().exec_reload();
        for cmd in cmds {
            cmd.no_spawn()?;
        }
        Ok(())
    }

    /// ## 服务退出执行的逻辑(包括自然退出及显式退出)
    pub fn after_exit(service: &mut ServiceUnit, exit_status: ExitStatus) {
        //TODO: 需要考虑是否需要在此处执行退出后代码，还是只需要显式退出时才执行
        let _ = Self::exec_stop_post(service);

        // 停止被spawn的命令
        let s_part = service.mut_service_part();
        for cmd in s_part.mut_exec_start_pos() {
            cmd.stop()
        }
        for cmd in s_part.mut_exec_start_pre() {
            cmd.stop()
        }

        // 取消未进行的定时器任务
        TimerManager::cancel_timer(service.unit_id());

        // 关闭和此服务绑定的项目
        for bind in service.unit_base().unit_part().be_binded_by() {
            UnitManager::try_kill_running(*bind);
        }

        //判断是否需要restart，需要则再次启动服务
        if service.service_part().restart().is_restart(&exit_status) {
            let _ = Self::restart(service);
            return;
        }

        //如果该进程标记了RemainAfterExit，则将其加入特殊标记表
        if service.service_part().remain_after_exit() {
            UnitManager::push_flag_running(service.unit_id());
            return;
        }

        //停止服务后设置Unit状态
        service.unit_base_mut().set_state(UnitState::Inactive);
    }

    /// ## 重启Service
    pub fn restart(service: &mut ServiceUnit) -> Result<(), RuntimeError> {
        let ns = service.service_part().restart_sec();
        let binds = service.unit_base().unit_part().be_binded_by();
        let binds = Vec::from(binds);
        let id = service.unit_id();
        if ns > 0 {
            let cmds = service.service_part().exec_reload().clone();
            TimerManager::push_timer(
                Duration::from_nanos(ns),
                move || {
                    for cmd in &cmds {
                        cmd.no_spawn()?;
                    }
                    Executor::exec(id)?;
                    for bind in &binds {
                        Executor::restart(*bind)?
                    }
                    Ok(())
                },
                service.unit_id(),
            )
        } else {
            UnitManager::try_kill_running(id);
            Self::exec_reload(service)?;
            eprintln!("restart");
            Self::exec(service)?;
            for bind in &binds {
                Executor::restart(*bind)?;
            }
        }
        Ok(())
    }

    /// ## 显示退出Service
    pub fn exit(service: &mut ServiceUnit) {
        // TODO: 打印日志
        let _ = Self::exec_stop(service);

        let ns = service.service_part().timeout_stop_sec();
        let id = service.unit_id();
        if ns != 0 {
            // 计时器触发后若服务还未停止，则kill掉进程
            TimerManager::push_timer(
                Duration::from_nanos(ns),
                move || {
                    UnitManager::try_kill_running(id);
                    Ok(())
                },
                service.unit_id(),
            )
        } else {
            UnitManager::try_kill_running(id);
        }
    }
}
