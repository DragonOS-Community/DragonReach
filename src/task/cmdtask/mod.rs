#[cfg(target_os = "dragonos")]
use drstd as std;

use std::{
    eprint, eprintln, print, println,
    process::{Child, Command},
    string::String,
    vec::Vec,
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
}

impl CmdTask {
    pub fn spawn(&self, dir: String, envs: &[(String, String)]) -> Result<(), RuntimeError> {
        let result = Command::new(&self.path)
            .args(&self.cmd)
            .current_dir(dir)
            .envs(Vec::from(envs))
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

    pub fn no_spawn(&self, dir: String, envs: &[(String, String)]) -> Result<(), RuntimeError> {
        let result = Command::new(&self.path)
            .args(&self.cmd)
            .current_dir(dir)
            .envs(Vec::from(envs))
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
}
