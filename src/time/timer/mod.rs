use std::time::{Duration, Instant};

use crate::{error::runtime_error::RuntimeError, unit::timer::TimerUnit};

/// 伪定时器，结合主循环来实现计时,在计时器触发时，会执行相应的cmd命令
/// 后续实现线程后，应使用线程实现
pub struct Timer {
    // 在创建计时器时记录当前时间
    instant: Instant,
    // 计时器触发时执行的闭包
    callback: Box<dyn FnMut() -> Result<(), RuntimeError> + Send + Sync + 'static>,
    // 计时时长
    duration: Duration,
    // 此计时器的拥有者(Unit)
    parent: usize,
}

impl Timer {
    /// ## 创建计时任务
    //要new一个unit！！，查询id命名规则
    pub fn new(
        duration: Duration,
        callback: Box<dyn FnMut() -> Result<(), RuntimeError> + Send + Sync + 'static>,
        parent: usize,
    ) -> Self {
        let _timerunit = TimerUnit::default();
        Timer {
            instant: Instant::now(),
            callback: callback,
            duration: duration,
            parent: parent,
        }
    }

    /// ## 判断当前是否到时，若执行该函数时已到时，将会执行定时任务
    ///
    /// ### return 到时返回true,否则返回false
    pub fn check(&mut self) -> bool {
        // println!("{},{}",self.instant.elapsed().as_micros(),self.duration.as_micros());
        if self.instant.elapsed().saturating_sub(self.duration) > Duration::ZERO {
            // TODO: 未进行错误处理
            if let Err(_e) = (self.callback)() {
                println!("task error");
            }
            return true;
        }
        return false;
    }

    /// ## 获取此计时器的拥有者Unit
    pub fn parent(&self) -> usize {
        self.parent
    }
}
