#[cfg(target_os = "dragonos")]
use drstd as std;
use std::hash::{Hash, Hasher};
use std::{
    collections::hash_map::DefaultHasher,
    collections::vec_deque::VecDeque,
    process::Child,
    sync::{Arc, Mutex, RwLock},
    vec::Vec,
};

use crate::unit::service::ServiceUnit;
use crate::unit::{service, Unit};
use hashbrown::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    // 对于启动后即使退出亦认为其为运行状态的特殊注册类Service,对于这类进程做一个标记
    static ref FLAG_RUNNING: RwLock<Vec<usize>> = RwLock::new(Vec::new());

    // 任务等待队列，IDLE类型的service入队等待其它任务完成再执行
    static ref IDLE_SERVIEC_DEQUE: Mutex<VecDeque<usize>> = Mutex::new(VecDeque::new());

    // id到unit的映射表，全局的Unit管理表
    static ref ID_TO_UNIT_MAP: RwLock<HashMap<usize,Arc<Mutex<dyn Unit>>>> = RwLock::new(HashMap::new());
    
    // 辅助表，通过服务名映射其id
    static ref PATH_TO_UNIT_MAP: RwLock<HashMap<u64,usize>> = RwLock::new(HashMap::new());

    // 全局运行中的Unit表
    pub(super) static ref RUNNING_TABLE: RwLock<RunningTableManager> = RwLock::new(RunningTableManager { running_table: Vec::new() });
}

pub struct RunningTableManager {
    running_table: Vec<RunningUnit>,
}

impl<'a> IntoIterator for &'a mut RunningTableManager {
    type Item = &'a mut RunningUnit;
    type IntoIter = std::slice::IterMut<'a, RunningUnit>;

    fn into_iter(self) -> Self::IntoIter {
        self.running_table.iter_mut()
    }
}

pub struct RunningUnit {
    process: Child,
    unit_id: usize,
}
impl RunningUnit {
    pub fn new(p: Child, unit: usize) -> Self {
        RunningUnit {
            process: p,
            unit_id: unit,
        }
    }

    pub fn child(&mut self) -> &mut Child {
        &mut self.process
    }

    pub fn id(&self) -> &usize {
        &self.unit_id
    }
}

pub struct UnitManager;

unsafe impl Sync for UnitManager {}

impl UnitManager {
    pub fn insert_into_path_table(path: &str, unit: usize) {
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        let hash = hasher.finish();
        PATH_TO_UNIT_MAP.write().unwrap().insert(hash, unit);
    }

    pub fn contains_path(path: &str) -> bool {
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        let hash = hasher.finish();
        PATH_TO_UNIT_MAP.read().unwrap().contains_key(&hash)
    }

    pub fn get_unit_with_path(path: &str) -> Option<Arc<Mutex<dyn Unit>>> {
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        let hash = hasher.finish();
        let map = PATH_TO_UNIT_MAP.read().unwrap();
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

    pub fn get_unit_with_id(id: &usize) -> Option<Arc<Mutex<dyn Unit>>> {
        let map = ID_TO_UNIT_MAP.read().unwrap();
        let ret = map.get(&id).cloned();
        ret
    }

    pub fn get_id_with_path(path: &str) -> Option<usize> {
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        let hash = hasher.finish();
        PATH_TO_UNIT_MAP.read().unwrap().get(&hash).cloned()
    }

    pub fn is_running_unit(id: &usize) -> bool {
        !RUNNING_TABLE
            .read()
            .unwrap()
            .running_table
            .iter()
            .filter(|x| x.unit_id == *id)
            .collect::<Vec<_>>()
            .is_empty()
    }

    pub fn push_running(unit: RunningUnit) {
        RUNNING_TABLE.write().unwrap().running_table.push(unit);
    }

    pub fn remove_running(id: usize) {
        let mut table = RUNNING_TABLE.write().unwrap();
        match table.running_table.iter().position(|x| x.unit_id == id) {
            Some(idx) => {
                table.running_table.remove(idx);
            }
            _ => (),
        }
    }

    pub fn insert_unit_with_id(id: usize, unit: Arc<Mutex<dyn Unit>>) {
        let mut map = ID_TO_UNIT_MAP.write().unwrap();
        if !map.contains_key(&id) {
            map.insert(id, unit);
        }
    }

    pub fn contains_id(id: &usize) -> bool{
        ID_TO_UNIT_MAP.read().unwrap().contains_key(id)
    }

    pub fn pop_a_idle_service() -> Option<Arc<Mutex<dyn Unit>>>{
        let id = IDLE_SERVIEC_DEQUE.lock().unwrap().pop_front();
        match id {
            Some(id) => {
                return Self::get_unit_with_id(&id);
            }
            None =>{
                return None;
            }
        }
    }

    pub fn push_a_idle_service(id: usize){
        if !Self::contains_id(&id) {
            return;
        }
        IDLE_SERVIEC_DEQUE.lock().unwrap().push_back(id);
    }

    pub fn push_flag_running(id: usize){
        let mut t = FLAG_RUNNING.write().unwrap();
        if t.contains(&id){
            return;
        }
        t.push(id);
    }

    pub fn running_count() -> usize{
        return RUNNING_TABLE.read().unwrap().running_table.len();
    }
}
