#[cfg(target_os = "dragonos")]
use drstd as std;

use std::time::{Duration, Instant};
use std::boxed::Box;
use std::{print,println};

use crate::{error::runtime_error::RuntimeError, task::cmdtask::CmdTask};

/// 伪定时器，结合主循环来实现计时,在计时器触发时，会执行相应的cmd命令
/// 后续实现线程后，应使用线程实现
pub struct Timer {
    instant: Instant,
    callback: Box<dyn FnMut() -> Result<(), RuntimeError> + Send + Sync + 'static>,
    duration: Duration,
    parent: usize,
}

impl Timer {
    pub fn new(
        duration: Duration,
        callback: Box<dyn FnMut() -> Result<(), RuntimeError> + Send + Sync + 'static>,
        parent: usize
    ) -> Self {
        Timer {
            instant: Instant::now(),
            callback: callback,
            duration: duration,
            parent: parent
        }
    }

    /// ## 判断当前是否到时，若执行该函数时已到时，将会执行定时任务
    ///
    /// ### return 到时返回true,否则返回false
    pub fn check(&mut self) -> bool {
        //println!("{},{}",self.instant.elapsed().as_micros(),self.duration.as_micros());
        if self.instant.elapsed().saturating_sub(self.duration) > Duration::ZERO {
            // TODO: 未进行错误处理
            if let Err(e) = (self.callback)() {
                println!("task error");
            }
            return true;
        }
        return false;
    }

    pub fn parent(&self) -> usize{
        self.parent
    }
}
