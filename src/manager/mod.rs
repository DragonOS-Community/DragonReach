#[cfg(target_os = "dragonos")]
use drstd as std;
use std::{
    sync::{Arc, RwLock}, collections::hash_map::DefaultHasher, process::Child,
};
use std::hash::{Hash,Hasher};

use crate::unit::Unit;
use hashbrown::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref GLOBAL_UNIT_MANAGER: RwLock<UnitManager> = {
        RwLock::new(UnitManager {
            id_to_unit: HashMap::new(),
            path_to_unit: HashMap::new(),
            running_table: Vec::new(),
        })
    };
}

pub struct RunnnigUnit(Child,Arc<dyn Unit>);
impl RunnnigUnit {
    pub fn new(p:Child,unit: Arc<dyn Unit>) -> Self {
        RunnnigUnit(p,unit)
    }
}

pub struct UnitManager {
    // 通过unit_id映射unit
    pub id_to_unit: HashMap<usize,Arc<dyn Unit>>,

    // 通过path的hash值来映射Unit
    pub path_to_unit: HashMap<u64,usize>,

    pub running_table: Vec<RunnnigUnit>
}

unsafe impl Sync for UnitManager {}

impl UnitManager {
    pub fn insert_into_path_table(&mut self,path: &str,unit: usize){
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        let hash = hasher.finish();
        self.path_to_unit.insert(hash, unit);
    }

    pub fn contants_path(&self,path: &str) -> bool{
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        let hash = hasher.finish();
        self.path_to_unit.contains_key(&hash)
    }

    pub fn get_unit_with_path(&self,path: &str) -> Option<&Arc<dyn Unit>> {
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        let hash = hasher.finish();
        let id = match self.path_to_unit.get(&hash) {
            Some(id) => id,
            None => {
                return None;
            }
        };

        self.id_to_unit.get(id)
    }

    pub fn get_unit_with_id(&self,id: &usize) -> Option<&Arc<dyn Unit>>{
        self.id_to_unit.get(&id)
    }
}
