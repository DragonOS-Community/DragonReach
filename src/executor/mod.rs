#[cfg(target_os = "dragonos")]
use drstd as std;

pub mod dep_graph;
pub mod service_executor;

use std::sync::Arc;
use std::{process::Child, sync::Mutex};

use crate::{
    error::runtime_error::{RuntimeError, RuntimeErrorType},
    manager::UnitManager,
    unit::Unit,
};

use self::dep_graph::DepGraph;

#[derive(Debug, Clone, Copy)]
pub enum ExitStatus {
    Success,
    Failure,
    Abnormal,
    Abort,
    Watchdog,
}

impl ExitStatus {
    /// ## 从错误码获得退出状态
    ///
    /// 注意，该方法只会返回Success(exit_code == 0)和Abnormal(exit_code != 0)两种状态
    /// 其他DragonReach定义的退出状态需要手动指定
    ///
    /// ### return Success(exit_code == 0)、Abnormal(exit_code != 0)
    pub fn from_exit_code(exit_code: i32) -> Self {
        match exit_code {
            0 => return Self::Success,
            _ => return Self::Abnormal,
        }
    }
}

//Unit的全局执行器
pub struct Executor;

impl Executor {
    pub fn exec(unit: &Arc<Mutex<dyn Unit>>) -> Result<(), RuntimeError> {
        //TODO: 优化此处，解析时也用到了拓扑排序，尝试使用那次拓扑排序的结果
        let mut graph = DepGraph::construct_graph(unit);

        let sort_ret = graph.topological_sort()?;
        for u in sort_ret {
            if UnitManager::is_running_unit(&u) {
                continue;
            }

            let mutex = UnitManager::get_unit_with_id(&u).unwrap();
            let mut unit = mutex.lock().unwrap();
            if let Err(e) = unit.run() {
                return Err(e);
            }
        }
        return Ok(());
    }
}
