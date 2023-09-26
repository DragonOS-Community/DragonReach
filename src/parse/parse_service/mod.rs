use super::graph::Graph;
use super::parse_util::UnitParseUtil;
use super::UnitParser;
use crate::error::parse_error::ParseError;
use crate::manager::UnitManager;
use crate::unit::service::{self, ServiceUnit};

#[cfg(target_os = "dragonos")]
use drstd as std;

use std::any::Any;
use std::rc::Rc;
use std::string::ToString;
use std::sync::Arc;
pub struct ServiceParser;

impl ServiceParser {
    /// @brief 解析Service类型Unit的
    ///
    /// 从path解析Service类型Unit
    ///
    /// @param path 需解析的文件路径
    ///
    /// @return 成功则返回Ok(Rc<ServiceUnit>)，否则返回Err
    pub fn parse(path: &str) -> Result<usize, ParseError> {
        //预先检查是否存在循环依赖
        let mut graph = Graph::construct_graph(path.to_string())?;
        let ret = graph.topological_sort()?;
        for p in ret {
            UnitParseUtil::parse_unit_no_type(&p)?;
        }

        let result = UnitManager::get_id_with_path(path).unwrap();

        return Ok(result);
    }
}
