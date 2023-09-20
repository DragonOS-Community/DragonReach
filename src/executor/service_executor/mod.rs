#[cfg(target_os = "dragonos")]
use drstd as std;

use std::{process::Command, sync::Arc};

use crate::{
    error::runtime_error::{RuntimeError, RuntimeErrorType},
    manager::{GLOBAL_UNIT_MANAGER, RunnnigUnit},
    unit::{service::ServiceUnit, UnitState},
};

pub struct ServiceExecutor;

impl ServiceExecutor {
    pub fn exec(service: &ServiceUnit) -> Result<(), RuntimeError> {
        let manager = GLOBAL_UNIT_MANAGER.read().unwrap();
        //处理conflict
        let conflicts = service.unit_base().unit_part().conflicts();
        for u in conflicts {
            // 如果有冲突项enable的时候，该unit不能启动
            let unit = manager.get_unit_with_id(u).unwrap();
            if *unit.unit_base().state() == UnitState::Enabled {
                eprintln!("{}: Service startup failed: conflict unit", unit.unit_base().unit_part().description());
                return Err(RuntimeError::new(RuntimeErrorType::ExecFailed));
            }
        }

        //处理ExecStarts,执行在服务启动前执行的命令
        let cmds = service.service_part().exec_start_pre();
        for cmdtask in cmds {
            cmdtask.exec()?;
        }

        //服务的启动命令
        let exec_start = service.service_part().exec_start();
        let proc = Command::new(&exec_start.path).args(&exec_start.cmd).spawn();

        match proc {
            Ok(p) => {
                //修改service状态

                //启动成功后将Child加入全局管理的进程表
                let mut manager_writer = GLOBAL_UNIT_MANAGER.write().unwrap();
                manager_writer.running_table.push(RunnnigUnit::new(p,Arc::new(service.clone())));
                //执行启动后命令
                let cmds = service.service_part().exec_start_pos();
                for cmd in cmds {
                    cmd.exec()?
                }
            }
            Err(err) => {
                eprintln!("{}: Service startup failed: {}", exec_start.path, err);
                return Err(RuntimeError::new(RuntimeErrorType::ExecFailed));
            }
        }

        Ok(())
    }
}
