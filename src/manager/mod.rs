pub mod ctl_manager;
pub mod timer_manager;
pub mod unit_manager;

pub use unit_manager::*;

use crate::executor::ExitStatus;

use self::timer_manager::TimerManager;
use crate::unit::signal::SIGCHILD_SIGNAL_RECEIVED;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::sync::atomic::Ordering;
pub struct Manager;

impl Manager {
    /// ## 检查当前 DragonReach 运行的项目状态，并对其分发处理
    pub fn check_running_status() {
        // 检查是否收到 SIGCHLD 信号
        if SIGCHILD_SIGNAL_RECEIVED.load(Ordering::SeqCst) {
            println!("SIGNAL_RECEIVED");
            SIGCHILD_SIGNAL_RECEIVED.store(false, Ordering::SeqCst);

            let mut exited_unit: Vec<(usize, ExitStatus)> = Vec::new();
            let mut running_manager = RUNNING_TABLE.write().unwrap();
            // 检查所有运行中的 Unit
            for unit in running_manager.mut_running_table() {
                let pid = Pid::from_raw(unit.1.id() as i32);
                // 检查 Unit 的运行状态
                match waitpid(Some(pid), Some(WaitPidFlag::WNOHANG)) {
                    // 若 Unit 为正常退出，则将其加入退出列表
                    Ok(WaitStatus::Exited(_, status)) => {
                        exited_unit.push((*unit.0, ExitStatus::from_exit_code(status)));
                    }
                    // 若 Unit 为被信号终止，则将其加入退出列表，并输出日志
                    Ok(WaitStatus::Signaled(_, signal, _)) => {
                        eprintln!("unit terminated by signal: {}", signal);
                        exited_unit.push((*unit.0, ExitStatus::from_exit_code(!0)));
                    }
                    // 其他错误情况
                    Err(_) => {
                        eprintln!("unit waitpid error");
                    }
                    // 若 Unit 正常运行，则不做处理
                    Ok(_) => {}
                }
            }

            drop(running_manager);

            // 处理退出的 Unit
            for tmp in exited_unit {
                // 将该任务从运行表中移除
                UnitManager::remove_running(tmp.0);

                // 取消该任务的定时器任务
                TimerManager::cancel_timer(tmp.0);

                // 交付处理子进程退出逻辑
                let _ = UnitManager::get_unit_with_id(&tmp.0)
                    .unwrap()
                    .lock()
                    .unwrap()
                    .exit();

                // 更新属于该 Unit 的定时器任务
                TimerManager::update_next_trigger(tmp.0, false);

                // 交付处理子进程退出后逻辑
                let unit = UnitManager::get_unit_with_id(&tmp.0).unwrap();
                unit.lock().unwrap().after_exit(tmp.1);
            }
            // 若无运行中任务，则取出 IDLE 任务运行
            if UnitManager::running_count() == 0 {
                if let Some(unit) = UnitManager::pop_a_idle_service() {
                    let _ = unit.lock().unwrap().run();
                }
            }
        }
    }

    /// ## 检查当前所有cmd进程的运行状态
    pub fn check_cmd_proc() {
        if SIGCHILD_SIGNAL_RECEIVED.load(Ordering::SeqCst) {
            SIGCHILD_SIGNAL_RECEIVED.store(false, Ordering::SeqCst);

            let mut exited = Vec::new();
            let mut table = CMD_PROCESS_TABLE.write().unwrap();

            for tuple in table.iter_mut() {
                let pid = Pid::from_raw(tuple.1.lock().unwrap().id() as i32);
                match waitpid(Some(pid), Some(WaitPidFlag::WNOHANG)) {
                    // 若 cmd 停止运行，则将其加入退出列表
                    Ok(WaitStatus::Exited(_, _)) | Ok(WaitStatus::Signaled(_, _, _)) => {
                        eprintln!("cmd exited");
                        exited.push(*tuple.0);
                    }
                    Ok(_) => {}
                    Err(_) => {
                        // TODO: 应该添加错误处理，有一些命令执行失败会影响服务正常运行
                        // 后续应该添加机制来执行服务相关命令启动失败的回调
                        eprintln!("cmd waitpid error");
                    }
                }
            }

            for id in exited {
                table.remove(&id);
            }
        }
    }
}
