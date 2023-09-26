#[cfg(target_os = "dragonos")]
use drstd as std;

use std::{
    eprint, eprintln, print, println,
    process::{Child, Command},
    string::String,
    vec::Vec, os::unix::process::CommandExt,
};

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
    pub envs: Vec<(String,String)>,
    pub pid: u32
}

impl CmdTask {
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

    pub fn no_spawn(&self) -> Result<(), RuntimeError> {
        let result = Command::new(&self.path)
            .args(&self.cmd)
            .current_dir(self.dir.clone())
            .envs(self.envs.clone())
            .output();
        match result {
            Ok(output) => {
                print!("{}", String::from_utf8_lossy(&output.stdout));
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
                if !output.status.success() {
                    return Err(RuntimeError::new(RuntimeErrorType::ExecFailed));
                }
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

    pub fn stop(&mut self){
        if self.pid != 0 {
            let res = UnitManager::pop_cmd_proc(self.pid).unwrap();

            let mut proc = res.lock().unwrap();

            match proc.try_wait() {
                //进程正常退出
                Ok(Some(status)) => {}
                //进程错误退出(或启动失败)
                _ => {
                    proc.kill().expect("Cannot kill cmd task");
                }
            };
        }
    }
}
