#[cfg(target_os = "dragonos")]
use drstd as std;

use std::{process::Command, string::String};

use crate::error::runtime_error::{RuntimeError, RuntimeErrorType};

#[derive(Debug, Clone, Default)]
pub struct CmdTask {
    pub path: String,
    pub cmd: Vec<String>,
    pub ignore: bool, //表示忽略这个命令的错误，即使它运行失败也不影响unit正常运作
}

impl CmdTask {
    pub fn exec(&self) -> Result<(), RuntimeError> {
        let result = Command::new(&self.path).args(&self.cmd).output();
        match result {
            Ok(output) => {
                if !output.status.success() && !self.ignore {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    eprintln!("{}: Command failed: {}", self.path, stderr);
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
