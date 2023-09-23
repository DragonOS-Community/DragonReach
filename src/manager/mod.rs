#[cfg(target_os = "dragonos")]
use drstd as std;
use std::fs::File;
use std::{eprint, eprintln, os::fd::AsFd, print, println, process::Child, vec::Vec};

pub mod unit_manager;
use std::io::Read;
use std::os::unix::io::{AsRawFd, FromRawFd};
pub use unit_manager::*;

use crate::executor::ExitStatus;

pub struct Manager;

impl Manager {
    /// ## 检查处于运行状态的unit状态
    pub fn check_running_status() {
        let mut running_manager = RUNNING_TABLE.write().unwrap();
        let mut dead_unit: Vec<usize> = Vec::new();
        let mut exited_unit: Vec<(usize, ExitStatus)> = Vec::new();
        for unit in running_manager.into_iter() {
            let proc = unit.child();
            match proc.try_wait() {
                //进程正常退出
                Ok(Some(status)) => {
                    //TODO:交付给相应类型的Unit类型去执行退出后的逻辑
                    println!("Service exited success");

                    exited_unit.push((
                        *unit.id(),
                        ExitStatus::from_exit_code(status.code().unwrap()),
                    ));

                    //退出后从表中去除该任务
                    dead_unit.push(*unit.id());
                }
                //进程错误退出(或启动失败)
                Err(e) => {
                    eprintln!("unit error: {}", e);

                    //test
                    exited_unit.push((
                        *unit.id(),
                        ExitStatus::from_exit_code(!0),
                    ));

                    //从表中去除该任务
                    dead_unit.push(*unit.id());
                }
                //进程处于正常运行状态
                _ => {}
            }
        }
        //释放锁，以便后续删除操作能拿到锁
        drop(running_manager);

        //从表中清除数据
        for id in dead_unit {
            UnitManager::remove_running(id);
        }

        if UnitManager::running_count() == 0 {
            let unit = UnitManager::pop_a_idle_service();
            match unit {
                Some(unit) => {
                    let _ = unit.lock().unwrap().run();
                }
                None => {}
            }
        }

        // 交付处理子进程退出逻辑
        for tmp in exited_unit {
            let unit = UnitManager::get_unit_with_id(&tmp.0).unwrap();
            unit.lock().unwrap().after_exit(tmp.1);
        }
    }
}
