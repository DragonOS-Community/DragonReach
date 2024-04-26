use super::{BaseUnit, Unit};
use crate::error::parse_error::{ParseError, ParseErrorType};
use crate::error::runtime_error::{RuntimeError, RuntimeErrorType};
use crate::executor::Executor;
use crate::manager::timer_manager::TimerManager;
use crate::manager::UnitManager;
use crate::parse::parse_timer::TimerParser;
use crate::parse::{Segment, TIMER_UNIT_ATTR_TABLE};
use crate::time::calandar::CalendarStandard;
use crate::unit::UnitState;
use humantime::parse_duration;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct TimerUnit {
    unit_base: BaseUnit,
    timer_part: TimerPart,
}

impl Default for TimerUnit {
    fn default() -> Self {
        Self {
            unit_base: Default::default(),
            timer_part: Default::default(),
        }
    }
}

impl Unit for TimerUnit {
    fn init(&mut self) {
        // 初始化计时器单元
        // 将单元状态设置为激活中
        self.unit_base.state = UnitState::Activating;
        // 更新计时器部分的数据
        let part = &mut self.timer_part;
        part.remain_after_elapse = true;

        // 设置初始触发时间
        let part = &mut self.timer_part;
        let now = Instant::now();
        part.last_trigger = now;
        part.now_time = now;

        // 如果设置了激活时的秒数，则添加一个计时器值
        if part.on_active_sec != Default::default() {
            part.value.push(TimerVal::new(
                TimerUnitAttr::OnActiveSec,
                false,
                part.on_active_sec,
                Default::default(),
                Some(now + part.on_active_sec),
            ));
        }

        // 实现OnActiveSec的具体逻辑

        // 检查单元是否正在运行
        let unit_is_running = UnitManager::is_running_unit(&part.unit);

        if part.on_unit_active_sec != Default::default() {
            let next_trigger = if unit_is_running {
                Some(now + part.on_unit_active_sec)
            } else {
                None
            };
            part.value.push(TimerVal::new(
                TimerUnitAttr::OnUnitActiveSec,
                !unit_is_running,
                part.on_unit_active_sec,
                Default::default(),
                next_trigger,
            ));
        }

        // 实现OnUnitActiveSec的具体逻辑

        if part.on_unit_inactive_sec != Default::default() {
            part.value.push(TimerVal::new(
                TimerUnitAttr::OnUnitInactiveSec,
                true,
                part.on_unit_inactive_sec,
                Default::default(),
                None, /*无论服务是否在运行，这里都不会有值 */
            ));
        }

        // 实现OnUnitInactiveSec的具体逻辑
        part.update_next_trigger();
        self._init();
        // 将单元状态设置为激活
        self.unit_base.state = UnitState::Active;
    }

    fn set_unit_name(&mut self, name: String) {
        // 设置单元的名称
        self.unit_base_mut().unit_name = name;
    }

    fn restart(&mut self) -> Result<(), RuntimeError> {
        // 重启单元
        self.exit();
        self.init();
        Ok(())
    }

    fn from_path(path: &str) -> Result<usize, ParseError>
    where
        Self: Sized,
    {
        // 从给定的路径解析并创建计时器单元
        TimerParser::parse(path)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        // 将计时器单元转换为任何类型，用于多态调用
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        // 将计时器单元转换为任何可变类型，用于多态调用
        self
    }

    fn set_attr(&mut self, segment: Segment, attr: &str, val: &str) -> Result<(), ParseError> {
        // 设置计时器单元的属性
        if segment != Segment::Timer {
            // 如果段不是计时器段，则返回错误
            return Err(ParseError::new(ParseErrorType::EINVAL, String::new(), 0));
        }
        let attr_type = TIMER_UNIT_ATTR_TABLE.get(attr).ok_or(ParseError::new(
            ParseErrorType::EINVAL,
            String::new(),
            0,
        ));
        return self.timer_part.set_attr(attr_type.unwrap(), val);
    }

    fn set_unit_base(&mut self, unit_base: BaseUnit) {
        // 设置单元的基础信息
        self.unit_base = unit_base;
    }

    fn unit_type(&self) -> super::UnitType {
        // 返回单元的类型
        self.unit_base.unit_type
    }

    fn unit_base(&self) -> &BaseUnit {
        // 返回单元的基础信息
        &self.unit_base
    }

    fn unit_base_mut(&mut self) -> &mut BaseUnit {
        // 返回单元的基础信息的可变引用
        &mut self.unit_base
    }

    fn unit_id(&self) -> usize {
        // 返回单元的ID
        self.unit_base.unit_id
    }

    fn run(&mut self) -> Result<(), RuntimeError> {
        //真正的run在_run中
        if self.check() && UnitManager::contains_id(&self.timer_part.unit) {
            // 如果单元检查通过并且单元管理器包含该单元，则运行
            let _ = self._run(); // 运行单元
            let id = self.get_parent_unit();
            // 更新下一个触发器
            TimerManager::update_next_trigger(id, true);
        } else if !UnitManager::contains_id(&self.timer_part.unit) {
            // 如果单元管理器不包含该单元，则打印错误信息
            println!("task error,unit does not exist")
        };
        Ok(())
    }

    fn exit(&mut self) {
        UnitManager::try_kill_running(self.unit_id());
    }
}

impl TimerUnit {
    pub fn _run(&mut self) -> Result<(), RuntimeError> {
        //到这里触发计时器对应的服务
        let part = &mut self.timer_part;
        if part.value.is_empty() {
            //触发次数已尽
            self.unit_base.state = UnitState::Inactive;
        } else if matches!(
            part.value[0].attr,
            TimerUnitAttr::OnActiveSec | TimerUnitAttr::OnBootSec | TimerUnitAttr::OnStartUpSec
        ) {
            part.value.remove(0); //消耗掉此次run时的TimeValue值
        }

        if UnitManager::is_running_unit(&part.unit) {
            //如果服务已经启动，则退出
            return Ok(());
        }

        //执行相应的unit单元
        if let Ok(_) = Executor::exec(part.unit) {
            self.unit_base.state = UnitState::Active;
            part.last_trigger = Instant::now();
            return Ok(());
        } else {
            self.unit_base.state = UnitState::Failed;
            return Err(RuntimeError::new(RuntimeErrorType::ExecFailed));
        }
    }
    fn _init(&self) {
        let unit: Arc<Mutex<TimerUnit>> = Arc::new(Mutex::new(self.clone()));
        TimerManager::push_timer_unit(unit);
    }

    pub fn check(&mut self) -> bool {
        let part = &mut self.timer_part;
        //计时器不可用
        if part.value.len() == 0 {
            //不可能再触发
            self.unit_base.state = UnitState::Inactive;
        }
        if self.unit_base.state == UnitState::Inactive
        //可能是手动停止
        {
            return false;
        }
        if UnitManager::is_running_unit(&part.unit)  //在运行就不管了
        || part.next_elapse_monotonic_or_boottime==None
        //下次触发时间无限大
        {
            return false;
        }
        part.now_time = Instant::now();

        //到时间执行Timer所管理的Unit
        // println!(
        //     "Now time::{:?},next_elapse_monotonic_or_boottime::{:?}_",
        //     part.now_time,
        //     part.next_elapse_monotonic_or_boottime.unwrap()
        // );

         //！！！此处判断在DragonOs有大问题！时间跨度很大，但在linux上正常运行
        if part.now_time >= part.next_elapse_monotonic_or_boottime.unwrap() {
            //检查Timer管理的unit是否存在
            if let Some(_) = UnitManager::get_unit_with_id(&part.unit) {
                // let _ = unit.lock().unwrap().run();//运行时作出相应操作,check不负责此操作
                // println!(
                //     "Now time::{:?},next_elapse_monotonic_or_boottime::{:?}_",
                //     part.now_time,
                //     part.next_elapse_monotonic_or_boottime.unwrap()
                // );
                return true;
            }
            println!("task error,unit does not exist");
        }
        return false;
    }

    pub fn unit_base(&self) -> &BaseUnit {
        &self.unit_base
    }

    pub fn timer_part(&self) -> &TimerPart {
        &self.timer_part
    }

    pub fn mut_timer_part(&mut self) -> &mut TimerPart {
        &mut self.timer_part
    }
    pub fn timer_init(&mut self) {
        let part = &mut self.timer_part;
        part.next_elapse_monotonic_or_boottime = None;
        part.next_elapse_realtime = Instant::now();
        part.now_time = Instant::now();
        part.remain_after_elapse = true;
    }

    pub fn get_parent_unit(&mut self) -> usize {
        self.timer_part().unit
    }

    pub fn enter_inactive(&mut self) -> bool {
        //判断计时器是否失效
        if self.unit_base.state == UnitState::Inactive {
            return true;
        }
        false
    }

    ///在unit run或exit的时候改变TimerValue中OnUnitInactiveSec和OnUnitActiveSec的状态
    pub fn change_stage(&mut self, flag: bool /*1为启动0为退出 */) {
        for val in &mut self.timer_part.value {
            match val.attr {
                TimerUnitAttr::OnUnitActiveSec => {
                    val.disabled = false;
                    if flag {
                        val.next_elapse = Some(Instant::now() + val.val);
                    }
                }
                TimerUnitAttr::OnUnitInactiveSec => {
                    val.disabled = false;
                    if !flag {
                        val.next_elapse = Some(Instant::now() + val.val);
                    }
                }
                _ => {}
            }
        }
    }
}
unsafe impl Sync for TimerUnit {}

unsafe impl Send for TimerUnit {}
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TimerPart {
    //TODO! 因此时DragonReach未实现时间事件源的相关功能，目前还是循环确认Timer的情况
    ///@brief 存储触发计时器的时间集合
    value: Vec<TimerVal>,

    ///@brief 相对于该单元自身被启动的时间点
    on_active_sec: Duration,

    ///@brief 相对于机器被启动的时间点
    on_boot_sec: Duration,

    ///@brief 相对于systemd被首次启动的时间点，也就是内核启动init进程的时间点
    on_start_up_sec: Duration,

    ///@brief 相对于匹配单元最后一次被启动的时间点
    on_unit_active_sec: Duration,

    ///@brief 相对于匹配单元 最后一次被停止的时间点
    on_unit_inactive_sec: Duration,

    ///@brief 定义基于挂钟时间(wallclock)的日历定时器，值是一个日历事件表达式
    on_calendar: CalendarStandard,

    ///@brief 设置定时器的触发精度,默认1min
    accuarcy_sec: usize,

    ///@brief 随机延迟一小段时间，默认0表示不延迟
    randomized_delay_sec: usize,

    ///@brief
    fixed_random_delay: bool,

    ///@brief
    on_clock_change: bool,

    ///@brief
    on_timezone_change: bool,

    ///@brief 默认值是 与此定时器单元同名的服务单元
    unit: usize,

    ///@brief 若设为"yes"，则表示将匹配单元的上次触发时间永久保存在磁盘上，默认no
    persistent: bool,

    ///@brief 若设为"yes"， 则表示当某个定时器到达触发时间点时， 唤醒正在休眠的系统并阻止系统进入休眠状态，默认no
    wake_system: bool,

    ///@brief 若设为"yes" ，那么该定时器将不会被再次触发，也就是可以确保仅被触发一次；默认yes
    remain_after_elapse: bool, //默认yes

    ///@brief 表示计时器下次实时时间触发的时间戳
    next_elapse_realtime: Instant,

    ///@brief 表示计时器下次单调时间或引导时间触发的时间戳
    next_elapse_monotonic_or_boottime: Option<Instant>, //None表示无限大

    ///@brief 用于存储计时器最后一次触发的时间戳。
    last_trigger: Instant,

    ///@brief 用于表示当前的时间。
    now_time: Instant,
}

impl Default for TimerPart {
    fn default() -> Self {
        Self {
            value: Default::default(),
            on_active_sec: Default::default(),
            on_boot_sec: Default::default(),
            on_start_up_sec: Default::default(),
            on_unit_active_sec: Default::default(),
            on_unit_inactive_sec: Default::default(),
            on_calendar: CalendarStandard::default(),
            accuarcy_sec: 60, // 默认设置为 60 秒
            randomized_delay_sec: 0,
            fixed_random_delay: false,
            on_clock_change: false,
            on_timezone_change: false,
            unit: Default::default(),
            persistent: false,
            wake_system: false,
            remain_after_elapse: true,

            next_elapse_realtime: Instant::now(),
            next_elapse_monotonic_or_boottime: None,
            last_trigger: Instant::now(),
            now_time: Instant::now(),
        }
    }
}

impl TimerPart {
    /// 更新下一次的触发时间
    pub fn update_next_trigger(&mut self) {
        self.now_time = Instant::now();

        //let unit_is_running=UnitManager::is_running_unit(&self.unit);
        //检查并更新value
        let mut index = 0;
        while index < self.value.len() {
            let val = &mut self.value[index];
            match val.attr {
                TimerUnitAttr::OnUnitInactiveSec | TimerUnitAttr::OnUnitActiveSec => {
                    //更新OnUnitInactiveSec和OnUnitActiveSec类型的值
                    if val.disabled || val.next_elapse == None {
                        //None表示此时无法确认下次触发时间
                        index = index + 1;
                        continue;
                    } else if val.next_elapse.unwrap() < self.now_time {
                        self.next_elapse_monotonic_or_boottime = val.next_elapse;
                        val.next_elapse = None;
                        // println!("Update the time!");
                        return;
                    }
                }

                TimerUnitAttr::OnActiveSec | TimerUnitAttr::OnBootSec => {
                    if val.next_elapse.unwrap() < self.now_time {
                        self.next_elapse_monotonic_or_boottime = val.next_elapse;
                        self.value.remove(index); //在这一步准备把index从value里弹出去
                        return;
                    }
                }
                //TimerUnitAttr::OnStartUpSec => todo!(),
                //TimerUnitAttr::OnCalendar => todo!(),
                _ => todo!(), //暂未支持
            }
            index += 1;
        }
        // 对value排序，使得最早的定时器时间在最前面,且None类型在最后面
        self.value.sort_by//(|a, b| a.next_elapse.cmp(&b.next_elapse));
        (|a, b| match (a.next_elapse, b.next_elapse) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
        (Some(a), Some(b)) => a.cmp(&b),
    });
        if self.value.is_empty() || self.value[0].next_elapse == None {
            //无法得到下次触发的具体时间
            return;
        }

        // 从已排序的Vec中获取最早的定时器时间
        self.next_elapse_monotonic_or_boottime = self.value[0].next_elapse;

        return;
    }
    /// &str->attr的parse
    pub fn set_attr(&mut self, attr: &TimerUnitAttr, val: &str) -> Result<(), ParseError> {
        match attr {
            TimerUnitAttr::OnActiveSec => {
                self.on_active_sec = {
                    if let Ok(duration) = parse_duration(val) {
                        duration
                    } else {
                        return Err(ParseError::new(ParseErrorType::EINVAL, String::new(), 0));
                    }
                }
            }

            TimerUnitAttr::OnBootSec => {
                self.on_boot_sec = {
                    if let Ok(duration) = parse_duration(val) {
                        duration
                    } else {
                        return Err(ParseError::new(ParseErrorType::EINVAL, String::new(), 0));
                    }
                }
            }

            TimerUnitAttr::OnStartUpSec => {
                self.on_start_up_sec = {
                    if let Ok(duration) = parse_duration(val) {
                        duration
                    } else {
                        return Err(ParseError::new(ParseErrorType::EINVAL, String::new(), 0));
                    }
                }
            }
            TimerUnitAttr::OnUnitInactiveSec => {
                self.on_unit_inactive_sec = {
                    if let Ok(duration) = parse_duration(val) {
                        duration
                    } else {
                        return Err(ParseError::new(ParseErrorType::EINVAL, String::new(), 0));
                    }
                }
            }
            TimerUnitAttr::OnUnitActiveSec => {
                self.on_unit_active_sec = {
                    if let Ok(duration) = parse_duration(val) {
                        duration
                    } else {
                        return Err(ParseError::new(ParseErrorType::EINVAL, String::new(), 0));
                    }
                }
            }
            //  TimerUnitAttr::OnCalendar=>self.on_calendar={
            //     if let Ok(calendar) = parse_calendar(val) {
            //         calendar
            //     } else {
            //         return Err(ParseError::new(ParseErrorType::EINVAL, String::new(), 0));
            //     }
            //  },
            TimerUnitAttr::Persistent => {
                self.persistent = {
                    match val {
                        "true" => true,
                        "false" => false,
                        _ => return Err(ParseError::new(ParseErrorType::EINVAL, String::new(), 0)),
                    }
                }
            }
            TimerUnitAttr::Unit => self.unit = UnitManager::get_id_with_path(val).unwrap(),
            _ => {
                return Err(ParseError::new(ParseErrorType::EINVAL, String::new(), 0));
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy)]
pub enum TimerUnitAttr {
    //TimerBase
    // State,
    // Result,
    OnActiveSec,
    OnBootSec,
    OnStartUpSec,
    OnUnitInactiveSec,
    OnUnitActiveSec,
    OnCalendar,
    AccuarcySec,
    RandomizedDelaySec,
    FixedRandomDelay,
    OnClockChange,
    OnTimeZoneChange,
    Unit,
    Persistent,
    WakeSystem,
    RemainAfterElapse,
}
impl Default for TimerUnitAttr {
    fn default() -> Self {
        TimerUnitAttr::OnActiveSec
    }
}

#[derive(Debug, Clone)]
pub struct TimerVal {
    attr: TimerUnitAttr,
    disabled: bool,
    val: Duration,
    //calendar_standard:Vec<CalendarStandard>,//只针对calendar事件
    next_elapse: Option<Instant>,
}

impl TimerVal {
    pub fn new(
        attr: TimerUnitAttr,
        disabled: bool,
        val: Duration,
        calendar_standard: Vec<CalendarStandard>,
        next_elapse: Option<Instant>,
    ) -> TimerVal {
        TimerVal {
            attr,
            disabled,
            val,
            //calendar_standard,
            next_elapse,
        }
    }
}
