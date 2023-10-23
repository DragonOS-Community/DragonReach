use super::BaseUnit;


pub struct TimerUnit {
    unit_base: BaseUnit,
    timer_part: TimerPart,
}

pub struct TimerPart {
    on_active_sec: u64,
    on_boot_sec: u64,
    on_start_up_sec: u64,
    on_unit_inactive_sec: u64,
    on_calendar: u64,
    accuarcy_sec: u64,
    randomized_delay_sec: u64,
    fixed_random_delay: bool,
    on_clock_change: bool,
    on_time_zone_change: bool,
    unit: usize,
    persistent: bool,
    wake_system: bool,
    remain_after_elapse: bool,
}

pub enum TimerUnitAttr {
    OnActiveSec,
    OnBootSec,
    OnStartUpSec,
    OnUnitInactiveSec,
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