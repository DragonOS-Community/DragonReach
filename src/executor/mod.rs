#[cfg(target_os = "dragonos")]
use drstd as std;

pub mod dep_graph;
pub mod service_executor;

use std::process::Child;
use std::sync::Arc;

use crate::{
    error::runtime_error::{RuntimeError, RuntimeErrorType},
    unit::Unit, manager::GLOBAL_UNIT_MANAGER,
};

use self::dep_graph::DepGraph;

//Unit的全局执行器
pub struct Executor;

impl Executor {
    pub fn exec(unit: &Arc<dyn Unit>) -> Result<(), RuntimeError> {
        //TODO: 优化此处，解析时也用到了拓扑排序，尝试使用那次拓扑排序的结果
        let mut graph = DepGraph::construct_graph(unit);

        let sort_ret = graph.topological_sort()?;
        let manager = GLOBAL_UNIT_MANAGER.read().unwrap();
        for u in sort_ret {
            if let Err(e) = manager.get_unit_with_id(&u).unwrap().run() {
                return Err(e);
            }
        }

        return Ok(());
    }
}
