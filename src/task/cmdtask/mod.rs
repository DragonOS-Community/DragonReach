use std::{eprint, eprintln, print, process::Command, string::String, vec::Vec};

use crate::{
    error::runtime_error::{RuntimeError, RuntimeErrorType},
    manager::UnitManager,
};

#[derive(Debug, Clone, Default)]
pub struct CmdTask {
    pub path: String,
    pub cmd: Vec<String>,
    pub ignore: bool, //表示忽略这个命令的错误，即使它运行失败也不影响unit正常运作
    pub dir: String,
    pub envs: Vec<(String, String)>,
    pub pid: u32,
}

impl CmdTask {
    /// ## 以新建进程的方式运行这个cmd
    pub fn spawn(&self) -> Result<(), RuntimeError> {
        let result = Command::new(&self.path)
            .args(&self.cmd)
            .current_dir(self.dir.clone())
            .envs(self.envs.clone())
            .spawn();
        match result {
            Ok(proc) => {
                UnitManager::push_cmd_proc(proc);
            }
            Err(err) => {
                if !self.ignore {
                    eprintln!("{}: Command failed: {}", self.path, err);
                    return Err(RuntimeError::new(RuntimeErrorType::ExecFailed));
                }
            }
        }
        Ok(())
    }

    /// ## 阻塞式运行
    pub fn no_spawn(&self) -> Result<(), RuntimeError> {
        let result = Command::new(&self.path)
            .args(&self.cmd)
            .current_dir(self.dir.clone())
            .envs(self.envs.clone())
            .spawn();

        match result {
            Ok(mut child) => match child.wait() {
                Ok(status) => {
                    if !status.success() && !self.ignore {
                        return Err(RuntimeError::new(RuntimeErrorType::ExecFailed));
                    }
                }
                Err(_) => {
                    if !self.ignore {
                        return Err(RuntimeError::new(RuntimeErrorType::ExecFailed));
                    }
                }
            },
            Err(e) => {
                if !self.ignore {
                    eprintln!("{}: Command failed: {}", self.path, e);
                    return Err(RuntimeError::new(RuntimeErrorType::ExecFailed));
                }
            }
        }
        Ok(())
    }

    /// ## 若这个cmd任务spawn了，则kill这个cmd进程
    pub fn stop(&mut self) {
        if self.pid != 0 {
            let res = UnitManager::pop_cmd_proc(self.pid).unwrap();

            let mut proc = res.lock().unwrap();

            match proc.try_wait() {
                //进程正常退出
                Ok(Some(_status)) => {}
                //进程错误退出(或启动失败)
                _ => {
                    proc.kill().expect("Cannot kill cmd task");
                }
            };
        }
    }
}
