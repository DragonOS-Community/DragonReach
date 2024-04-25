pub mod ctl_manager;
pub mod timer_manager;
pub mod unit_manager;

pub use unit_manager::*;

use crate::executor::ExitStatus;

use self::timer_manager::TimerManager;

pub struct Manager;

impl Manager {
    /// ## 检查当前DragonReach运行的项目状态，并对其分发处理
    pub fn check_running_status() {
        // 检查正在运行的Unit
        let mut running_manager = RUNNING_TABLE.write().unwrap();
        let mut exited_unit: Vec<(usize, ExitStatus)> = Vec::new();
        for unit in running_manager.mut_running_table() {
            let proc = unit.1;
            match proc.try_wait() {
                //进程正常退出
                Ok(Some(status)) => {
                    exited_unit.push((
                        *unit.0,
                        ExitStatus::from_exit_code(status.code().unwrap_or(0)),
                    ));
                }
                //进程错误退出(或启动失败)
                Err(e) => {
                    eprintln!("unit error: {}", e);

                    //test
                    exited_unit.push((*unit.0, ExitStatus::from_exit_code(!0)));
                }
                //进程处于正常运行状态
                _ => {}
            }
        }
        //释放锁，以便后续删除操作能拿到锁
        drop(running_manager);

        // 处理退出的Unit
        for tmp in exited_unit {
            // 从运行表中擦除该unit
            UnitManager::remove_running(tmp.0);

            // 取消该任务的定时器任务
            TimerManager::cancel_timer(tmp.0);

            let _ = UnitManager::get_unit_with_id(&tmp.0)
                .unwrap()
                .lock()
                .unwrap()
                .exit(); //交付给相应类型的Unit类型去执行退出后的逻辑

                TimerManager::update_next_trigger(tmp.0,false); //更新所有归属于此unit的计时器
            

               // 交付处理子进程退出逻辑
            let unit = UnitManager::get_unit_with_id(&tmp.0).unwrap();
            unit.lock().unwrap().after_exit(tmp.1);
        }

        // 若无运行中任务，则取出IDLE任务运行
        if UnitManager::running_count() == 0 {
            let unit = UnitManager::pop_a_idle_service();
            match unit {
                Some(unit) => {
                    let _ = unit.lock().unwrap().run();
                }
                None => {}
            }
        }
    }

    /// ## 检查当前所有cmd进程的运行状态
    pub fn check_cmd_proc() {
        let mut exited = Vec::new();
        let mut table = CMD_PROCESS_TABLE.write().unwrap();
        for tuple in table.iter_mut() {
            let mut proc = tuple.1.lock().unwrap();
            match proc.try_wait() {
                // 正常运行
                Ok(None) => {}
                // 停止运行，从表中删除数据
                _ => {
                    // TODO: 应该添加错误处理，有一些命令执行失败会影响服务正常运行
                    // 后续应该添加机制来执行服务相关命令启动失败的回调
                    exited.push(*tuple.0);
                }
            }
        }

        for id in exited {
            table.remove(&id);
        }
    }
}
