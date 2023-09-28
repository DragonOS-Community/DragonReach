use crate::{error::runtime_error::RuntimeError, time::timer::Timer};
use lazy_static::lazy_static;
use std::{boxed::Box, sync::RwLock, time::Duration, vec::Vec};

lazy_static! {
    // 管理全局计时器任务
    static ref TIMER_TASK_MANAGER: RwLock<TimerManager> = RwLock::new(TimerManager {
        inner_timers: Vec::new()
    });
}

pub struct TimerManager {
    inner_timers: Vec<Timer>,
}

impl<'a> IntoIterator for &'a mut TimerManager {
    type Item = &'a mut Timer;

    type IntoIter = std::slice::IterMut<'a, Timer>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner_timers.iter_mut()
    }
}

impl TimerManager {
    /// ## 添加定时器任务
    ///
    /// 只有通过这个方式创建的Timer对象才会真正的实现计时
    pub fn push_timer<F>(duration: Duration, callback: F, parent: usize)
    where
        F: FnMut() -> Result<(), RuntimeError> + Send + Sync + 'static,
    {
        TIMER_TASK_MANAGER
            .write()
            .unwrap()
            .inner_timers
            .push(Timer::new(duration, Box::new(callback), parent));
    }

    /// ## 检测定时器是否到时，到时则触发
    ///
    /// 该方法在主循环中每循环一次检测一次，是伪计时器的主运行函数
    pub fn check_timer() {
        let mut writer = TIMER_TASK_MANAGER.write().unwrap();
        //此处触发定时器，若定时器被触发，则移除
        writer.inner_timers.retain_mut(|x| !x.check());
    }

    /// ## 取消掉一个unit的所有定时任务，
    ///
    /// 一般在unit启动失败或者退出unit时进行该操作
    pub fn cancel_timer(unit_id: usize) {
        TIMER_TASK_MANAGER
            .write()
            .unwrap()
            .inner_timers
            .retain(|x| x.parent() == unit_id)
    }
}
