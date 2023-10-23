use std::hash::{Hash, Hasher};
use std::{
    collections::hash_map::DefaultHasher,
    collections::vec_deque::VecDeque,
    print, println,
    process::Child,
    sync::{Arc, Mutex, RwLock},
    vec::Vec,
};

use crate::unit::Unit;
use hashbrown::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    // 对于启动后即使退出亦认为其为运行状态的特殊注册类Service,对于这类进程做一个标记
    static ref FLAG_RUNNING: RwLock<Vec<usize>> = RwLock::new(Vec::new());

    // 任务等待队列，IDLE类型的service入队等待其它任务完成再执行
    static ref IDLE_SERVIEC_DEQUE: Mutex<VecDeque<usize>> = Mutex::new(VecDeque::new());

    // id到unit的映射表，全局的Unit管理表
    pub(super) static ref ID_TO_UNIT_MAP: RwLock<HashMap<usize,Arc<Mutex<dyn Unit>>>> = RwLock::new(HashMap::new());

    // 辅助表，通过服务名映射其id
    static ref NAME_TO_UNIT_MAP: RwLock<HashMap<u64,usize>> = RwLock::new(HashMap::new());

    // 全局运行中的Unit表
    pub(super) static ref RUNNING_TABLE: RwLock<RunningTableManager> = RwLock::new(RunningTableManager { running_table: HashMap::new() });

    // CMD进程表，用于处理Unit的CMD派生进程(ExecStartPre等命令派生进程)
    pub(super) static ref CMD_PROCESS_TABLE: RwLock<HashMap<u32,Mutex<Child>>> = RwLock::new(HashMap::new());
}

pub struct RunningTableManager {
    running_table: HashMap<usize, Child>,
}

#[allow(dead_code)]
impl RunningTableManager {
    pub fn running_table(&self) -> &HashMap<usize, Child> {
        &self.running_table
    }

    pub fn mut_running_table(&mut self) -> &mut HashMap<usize, Child> {
        &mut self.running_table
    }
}

pub struct UnitManager;

unsafe impl Sync for UnitManager {}

#[allow(dead_code)]
impl UnitManager {
    /// 插入一条path到unit_id的映射
    pub fn insert_into_name_table(path: &str, unit: usize) {
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        let hash = hasher.finish();
        NAME_TO_UNIT_MAP.write().unwrap().insert(hash, unit);
    }

    // 判断当前是否已经有了对应path的Unit
    pub fn contains_name(path: &str) -> bool {
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        let hash = hasher.finish();
        NAME_TO_UNIT_MAP.read().unwrap().contains_key(&hash)
    }

    // 通过path获取到Unit
    pub fn get_unit_with_name(name: &str) -> Option<Arc<Mutex<dyn Unit>>> {
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        let hash = hasher.finish();
        let map = NAME_TO_UNIT_MAP.read().unwrap();
        let id = match map.get(&hash) {
            Some(id) => id,
            None => {
                return None;
            }
        };

        let map = ID_TO_UNIT_MAP.read().unwrap();
        let ret = map.get(id).cloned();
        ret
    }

    // 通过unit_id获取Unit
    pub fn get_unit_with_id(id: &usize) -> Option<Arc<Mutex<dyn Unit>>> {
        let map = ID_TO_UNIT_MAP.read().unwrap();
        let ret = map.get(&id).cloned();
        ret
    }

    // 通过id获取到path
    pub fn get_id_with_path(path: &str) -> Option<usize> {
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        let hash = hasher.finish();
        NAME_TO_UNIT_MAP.read().unwrap().get(&hash).cloned()
    }

    // 判断该Unit是否正在运行中
    pub fn is_running_unit(id: &usize) -> bool {
        RUNNING_TABLE.read().unwrap().running_table.contains_key(id)
            || !FLAG_RUNNING
                .read()
                .unwrap()
                .iter()
                .filter(|x| **x == *id)
                .collect::<Vec<_>>()
                .is_empty()
    }

    // 向运行表中添加运行的Unit
    pub fn push_running(unit_id: usize, p: Child) {
        RUNNING_TABLE
            .write()
            .unwrap()
            .running_table
            .insert(unit_id, p);
    }

    // 删除运行表中的Unit
    pub fn remove_running(id: usize) {
        let mut table = RUNNING_TABLE.write().unwrap();
        table.running_table.remove(&id);
    }

    // 向id到Unit映射表中插入数据
    pub fn insert_unit_with_id(id: usize, unit: Arc<Mutex<dyn Unit>>) {
        let mut map = ID_TO_UNIT_MAP.write().unwrap();
        if !map.contains_key(&id) {
            map.insert(id, unit);
        }
    }

    // 判断当前DragonReach是否拥有目标id的Unit
    pub fn contains_id(id: &usize) -> bool {
        ID_TO_UNIT_MAP.read().unwrap().contains_key(id)
    }

    // 弹出一个处于IDLE状态的Service
    pub fn pop_a_idle_service() -> Option<Arc<Mutex<dyn Unit>>> {
        let id = IDLE_SERVIEC_DEQUE.lock().unwrap().pop_front();
        match id {
            Some(id) => {
                return Self::get_unit_with_id(&id);
            }
            None => {
                return None;
            }
        }
    }

    // 添加IDLE状态的Service，将在后续调度
    pub fn push_a_idle_service(id: usize) {
        if !Self::contains_id(&id) {
            return;
        }
        IDLE_SERVIEC_DEQUE.lock().unwrap().push_back(id);
    }

    // 将该Unit标记为运行状态，并且后续不会对其进行运行检查
    pub fn push_flag_running(id: usize) {
        let mut t = FLAG_RUNNING.write().unwrap();
        if t.contains(&id) {
            return;
        }
        t.push(id);
    }

    // 当前运行的Unit数
    pub fn running_count() -> usize {
        return RUNNING_TABLE.read().unwrap().running_table.len();
    }

    // 向Cmd运行表中添加
    pub fn push_cmd_proc(proc: Child) {
        CMD_PROCESS_TABLE
            .write()
            .unwrap()
            .insert(proc.id(), Mutex::new(proc));
    }

    // 删除对应cmd的进程
    pub fn remove_cmd_proc(id: u32) {
        CMD_PROCESS_TABLE.write().unwrap().remove(&id);
    }

    // 弹出指定id的cmd进程
    pub fn pop_cmd_proc(id: u32) -> Option<Mutex<Child>> {
        CMD_PROCESS_TABLE.write().unwrap().remove(&id)
    }

    // 初始化各Unit的依赖关系，此方法只需在解析完系统Unit文件后调用一次
    pub fn init_units_dependencies() {
        let manager = ID_TO_UNIT_MAP.write().unwrap();

        // 处理before段，将before段的Unit添加此Unit为After
        for (id, unit) in manager.iter() {
            let mut unit = unit.lock().unwrap();
            let before = unit.unit_base_mut().unit_part().before();
            for rid in before {
                let req = UnitManager::get_unit_with_id(rid).unwrap();
                let mut req = req.lock().unwrap();
                req.unit_base_mut().mut_unit_part().push_after_unit(*id);
            }
        }

        for (id, unit) in manager.iter() {
            let mut unit = unit.lock().unwrap();

            // 处理binds_to段
            let binds_to = unit.unit_base_mut().unit_part().binds_to();
            for rid in binds_to {
                let req = UnitManager::get_unit_with_id(rid).unwrap();
                let mut req = req.lock().unwrap();
                req.unit_base_mut().mut_unit_part().push_be_binded_by(*id);
            }

            // 处理part_of段
            let part_of = unit.unit_base_mut().unit_part().part_of();
            for rid in part_of {
                let req = UnitManager::get_unit_with_id(rid).unwrap();
                let mut req = req.lock().unwrap();
                req.unit_base_mut().mut_unit_part().push_be_binded_by(*id);
            }
        }
    }

    /// ## 如果Unit进程正在运行则杀死Unit进程
    pub fn try_kill_running(id: usize) -> bool {
        if Self::is_running_unit(&id) {
            Self::kill_running(id);
            return true;
        }
        return false;
    }

    pub fn kill_running(id: usize) {
        let mut running_manager = RUNNING_TABLE.write().unwrap();
        let unit = running_manager.running_table.get_mut(&id).unwrap();
        let _ = unit.kill();
        println!("kill:{}", id);
        running_manager.running_table.remove(&id);
    }
}
