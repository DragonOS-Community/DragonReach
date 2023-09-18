use super::UnitParser;
use crate::error::ParseError;
use crate::unit::target::TargetUnit;

#[cfg(target_os = "dragonos")]
use drstd as std;

use std::rc::Rc;

pub struct TargetParser;

impl TargetParser {
    /// @brief 解析Service类型Unit的
    ///
    /// 从path解析Service类型Unit
    ///
    /// @param path 需解析的文件路径
    ///
    /// @return 成功则返回Ok(Rc<ServiceUnit>)，否则返回Err
    pub fn parse(path: &str) -> Result<Rc<TargetUnit>, ParseError> {
        //交付总解析器
        let service = UnitParser::parse::<TargetUnit>(path, crate::unit::UnitType::Service)?;
        return Ok(service);
    }
}