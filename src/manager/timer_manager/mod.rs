use std::{
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

use crate::{
    error::runtime_error::RuntimeError, time::timer::Timer, unit::timer::TimerUnit, unit::Unit,
};
use hashbrown::HashMap;
use lazy_static::lazy_static;



lazy_static! {
    // 管理全局计时器任务
    static ref TIMER_TASK_MANAGER:RwLock<TimerManager> = RwLock::new(TimerManager {
        inner_timers: Vec::new(),
        timer_unit_map: RwLock::new(HashMap::new()),

        id_table:RwLock::new(Vec::new())
    });
}

pub struct TimerManager {
    inner_timers: Vec<Timer>,
    timer_unit_map: RwLock<HashMap<usize, Arc<Mutex<TimerUnit>>>>, //id->TimerUnit
    id_table: RwLock<Vec<(usize, usize)>>, //.0是TimerUnit的id,.1是父Unit的id
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

    pub fn push_timer_unit(unit: Arc<Mutex<TimerUnit>>) {
        let timemanager = TIMER_TASK_MANAGER.write().unwrap();
        let mut unit_ = unit.lock().unwrap();
        let unit_id = unit_.unit_id();
        timemanager
            .id_table
            .write()
            .unwrap()
            .push((unit_id, unit_.get_parent_unit()));
        drop(unit_);
        timemanager
            .timer_unit_map
            .write()
            .unwrap()
            .insert(unit_id, unit); //加入到inner_timers_map
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
        let timer_unit_map = reader.timer_unit_map.read().unwrap();
        let mut inactive_unit: Vec<usize> = Vec::new();
        for (_, timer_unit) in timer_unit_map.iter() {
            let mut timer_unit = timer_unit.lock().unwrap();
            if timer_unit.enter_inactive() {
                inactive_unit.push(timer_unit.unit_id());
                continue;
            }
            if timer_unit.check() {
                //println!("unit id : {} , parent id : {} ",timer_unit.unit_id(),timer_unit.get_parent_unit());
                let _ = timer_unit._run(); //运行作出相应操作
                let id = timer_unit.get_parent_unit();
                drop(timer_unit);
                TimerManager::update_next_trigger(id, true); //更新触发时间
            }
        }

        for id in inactive_unit {//处理Inactive需要退出的计时器
            //println!("Prepared to exit...");
            timer_unit_map.get(&id).unwrap().lock().unwrap().exit();

            TimerManager::remove_timer_unit(id);
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

    pub fn is_timer(id: &usize) -> bool {
        let id_table = &TIMER_TASK_MANAGER.read().unwrap().id_table;
        for iter in id_table.read().unwrap().iter() {
            if iter.0 == *id {
                return true;
            }
        }
        false
    }
    /// unit_id:父unit的id  flag:1为exec 0为exit
    fn adjust_timevalue(unit_id: &usize, flag: bool /*1为启动0为退出 */) -> Vec<usize> {
        let mut result = Vec::new();
        let manager = TIMER_TASK_MANAGER.read().unwrap();
        for (self_id, parent_id) in manager.id_table.read().unwrap().iter() {
            if unit_id == parent_id {
                let timer_unit_map = manager.timer_unit_map.read().unwrap();
                let mut timer_unit = timer_unit_map.get(self_id).unwrap().lock().unwrap();
                timer_unit.change_stage(flag);
                result.push(*self_id);
            }
        }
        result
    }

    /// 从Timer表中删除该Unit
    pub fn remove_timer_unit(unit_id: usize) {
        let manager = TIMER_TASK_MANAGER.read().unwrap();

        manager.timer_unit_map.write().unwrap().remove(&unit_id);
        let mut index: usize = 0;
        let mut id_table = manager.id_table.write().unwrap();
        for (self_id, _) in id_table.iter() {
            //因为id是递增的，后续可优化为二分查找
            if unit_id == *self_id {
                id_table.remove(index);
                println!("remove id:{}", unit_id);
                return;
            }
            index = index + 1
        }
    }

    /// 获得该id下的所有计时器
    pub fn get_timer(parent_id: usize) -> Vec<usize> {
        let mut result = Vec::new();
        let timer_manager = TIMER_TASK_MANAGER.read().unwrap();
        let reader = timer_manager.id_table.read().unwrap();
        for (timer_id, id) in reader.iter() {
            if *id == parent_id {
                result.push(*timer_id);
            }
        }
        result
    }
    ///此时传入的是parent_id
    pub fn update_next_trigger(unit_id: usize, flag: bool) {
        let timer_vec = Self::adjust_timevalue(&unit_id, flag);

        let timer_manager = TIMER_TASK_MANAGER.read().unwrap();
        let timer_unit_map = timer_manager.timer_unit_map.read().unwrap();
        timer_vec.iter().for_each(|id| {
            timer_unit_map
                .get(id)
                .unwrap()
                .lock()
                .unwrap()
                .mut_timer_part()
                .update_next_trigger();
        });
    }
}
