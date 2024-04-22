use std::{sync::{Arc, Mutex, RwLock}, time::Duration};

use crate::{error::runtime_error::RuntimeError, time::timer::Timer, unit::timer::TimerUnit,unit::Unit};
use hashbrown::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    // 管理全局计时器任务
    static ref TIMER_TASK_MANAGER:RwLock<TimerManager> = RwLock::new(TimerManager {
        inner_timers: Vec::new(),
        inner_timers_unit: Vec::new(),
        id_table:RwLock::new(Vec::new())//.0是TimerUnit的id,.1是父Unit的id
    });
}

pub struct TimerManager {
    inner_timers: Vec<Timer>,
    inner_timers_unit: Vec<Arc<Mutex<TimerUnit>>>,
    id_table: RwLock<Vec<(usize,usize)>>,
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
    /// 只有通过两这个方式载入的Timer或Timer_unit对象才会真正的实现计时
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

    pub fn push_timer_unit(unit:Arc<Mutex<TimerUnit>>)
    {
        let mut timemanager=TIMER_TASK_MANAGER
            .write()
            .unwrap();
        let mut unit_ =unit.lock().unwrap();
        timemanager.id_table.write().unwrap().push((unit_.unit_id(), unit_.get_parent_unit()));
        drop(unit_);
        timemanager.inner_timers_unit
        .push(unit);//加入到inner_timers_unit
    }

    /// ## 检测定时器是否到时，到时则触发
    ///
    /// 该方法在主循环中每循环一次检测一次，是伪计时器的主运行函数
    pub fn check_timer() {
        let mut writer = TIMER_TASK_MANAGER.write().unwrap();
        //此处触发定时器，若定时器被触发，则移除
        writer.inner_timers.retain_mut(|x| !x.check());
        drop(writer);
        //此处触发Timer_unit,不移除
        let reader = TIMER_TASK_MANAGER.read().unwrap();

        for timer in &reader.inner_timers_unit {
            let  mut timer_unit=timer.lock().unwrap();
            if timer_unit.check() {
                  let _ = timer_unit.run();//运行作出相应操作
                  let id =timer_unit.get_parent_unit();
                  drop(timer_unit);
                  TimerManager::adjust_timevalue(&id, true);
            }
            
        }
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

    pub fn is_timer(id:&usize)->bool{
        let id_table=&TIMER_TASK_MANAGER
        .read()
        .unwrap()
        .id_table;
        for iter in id_table.read().unwrap().iter(){
            if iter.0==*id {
                return true;
            }
        }
        false
    }
    /// unit_id:父unit的id  flag:1为exec 0为exit 
    pub fn adjust_timevalue(unit_id: &usize, flag: bool /*1为启动0为退出 */){
        let manager=TIMER_TASK_MANAGER
        .read()
        .unwrap();
        for iter in &manager.inner_timers_unit{
            let mut unit =iter.lock().unwrap();
            if unit.get_parent_unit()==*unit_id{
                unit.change_stage(flag);
            }
        }
    }

}
