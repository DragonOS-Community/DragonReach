pub mod dep_graph;
pub mod service_executor;

use crate::{
    error::runtime_error::{RuntimeError, RuntimeErrorType},
    manager::UnitManager,
    unit::UnitState,
};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum ExitStatus {
    Success,  // 成功退出
    Failure,  // 启动失败
    Abnormal, // 异常退出
    Abort,    // 显式退出
    Watchdog, // 超时检测
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
    /// ## 全局执行器入口，将会进行启动检测以及循环依赖检测
    pub fn exec(unit_id: usize) -> Result<(), RuntimeError> {
        // TODO: 添加超时检测，这个工作应该在线程执行

        {
            // 设置Unit状态为正在启动
            // TODO: 目前单线程工作这样设置是无意义的
            UnitManager::get_unit_with_id(&unit_id)
                .unwrap()
                .lock()
                .unwrap()
                .unit_base_mut()
                .set_state(UnitState::Activating);
        }
        match Self::exec_(unit_id) {
            Ok(_) => {
                UnitManager::get_unit_with_id(&unit_id)
                .unwrap()
                .lock()
                .unwrap()
                .unit_base_mut()
                .set_state(UnitState::Active);
                Ok(())
            },
            Err(e) => {
                let mutex = UnitManager::get_unit_with_id(&unit_id).unwrap();
                let mut unit = mutex.lock().unwrap();

                // 启动失败时启动onfailure项目
                for id in unit.unit_base().unit_part().on_failure() {
                    // TODO: 待日志库开发后，这里的错误处理应该是打印日志
                    let _ = Executor::exec(*id);
                }

                unit.unit_base_mut().set_state(UnitState::Failed);
                unit.after_exit(ExitStatus::Failure);
                return Err(e);
            }
        }
    }
    pub fn exec_(unit_id: usize) -> Result<(), RuntimeError> {
        // TODO： 目前的启动逻辑还是串行启动，后续需更改为并行启动某些项

        let unit = match UnitManager::get_unit_with_id(&unit_id) {
            Some(s) => s,
            None => {
                return Err(RuntimeError::new(RuntimeErrorType::FileNotFound));
            }
        };

        let mut unit = unit.lock().unwrap();

        //TODO: 优化此处，解析时也用到了拓扑排序，尝试使用那次拓扑排序的结果
        // 此处不需要再次拓扑排序，在parse时已经确定不会出现循环依赖，现在仅需按照启动流程启动即可
        // let mut graph = DepGraph::construct_graph(&unit);
        // let sort_ret = graph.topological_sort()?;

        // 优先启动After
        for u in unit.unit_base().unit_part().after() {
            if UnitManager::is_running_unit(&u) {
                continue;
            }

            let mutex = UnitManager::get_unit_with_id(&u).unwrap();
            let mut after = mutex.lock().unwrap();
            after.run()?;
        }

        // 启动Requires
        for u in unit.unit_base().unit_part().requires() {
            if UnitManager::is_running_unit(&u) {
                continue;
            }
            let mutex = UnitManager::get_unit_with_id(&u).unwrap();
            let mut after = mutex.lock().unwrap();
            after.run()?;
        }

        // 启动binds
        for u in unit.unit_base().unit_part().binds_to() {
            if UnitManager::is_running_unit(&u) {
                continue;
            }
            let mutex = UnitManager::get_unit_with_id(&u).unwrap();
            let mut after = mutex.lock().unwrap();
            after.run()?;
        }

        // 启动Wants
        for u in unit.unit_base().unit_part().wants() {
            if UnitManager::is_running_unit(&u) {
                continue;
            }
            let mutex = UnitManager::get_unit_with_id(&u).unwrap();
            let mut after = mutex.lock().unwrap();
            let _ = after.run();
        }

        // 启动自身
        unit.run()?;
        return Ok(());
    }

    pub fn restart(id: usize) -> Result<(), RuntimeError> {
        if let Some(unit) = UnitManager::get_unit_with_id(&id) {
            unit.lock().unwrap().restart()?;
        }
        Ok(())
    }
}
