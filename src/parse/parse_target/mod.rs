use super::UnitParser;
use super::graph::Graph;
use super::parse_util::UnitParseUtil;
use crate::error::parse_error::ParseError;
use crate::manager::GLOBAL_UNIT_MANAGER;
use crate::unit::target::TargetUnit;

#[cfg(target_os = "dragonos")]
use drstd as std;

use std::rc::Rc;
use std::sync::Arc;

pub struct TargetParser;

impl TargetParser {
    /// @brief 解析Service类型Unit的
    ///
    /// 从path解析Service类型Unit
    ///
    /// @param path 需解析的文件路径
    ///
    /// @return 成功则返回Ok(Rc<ServiceUnit>)，否则返回Err
    pub fn parse(path: &str) -> Result<Arc<TargetUnit>, ParseError> {
        //预先检查是否存在循环依赖
        let mut graph = Graph::construct_graph(path.to_string())?;
        let ret = graph.topological_sort()?;
        for p in ret {
            let temp_unit = UnitParseUtil::parse_unit_no_type(&p)?;
        }

        let manager = GLOBAL_UNIT_MANAGER.read().unwrap();
        let result = manager.get_unit_with_path(path).unwrap();

        let result : TargetUnit = result.as_any().downcast_ref::<TargetUnit>().unwrap().clone();
        return Ok(Arc::new(result));
    }
}