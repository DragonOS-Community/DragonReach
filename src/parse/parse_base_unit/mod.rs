use crate::{
    error::ParseError,
    unit::{target::TargetUnit, BaseUnitAttr, InstallPart, UnitPart},
};
//use drstd as std;
use std::rc::Rc;
use std::string::String;
use std::vec::Vec;

use super::{parse_util::UnitParseUtil, InstallUnitAttr};

//Unit同有部分的解析器
pub struct BaseUnitParser;

impl BaseUnitParser {
    /// @brief 为Unit解析Unit段属性并添加
    ///
    /// 为Unit解析Unit段属性并添加
    ///
    /// @param unit_part UnitPart(Unit段结构体)，解析结果将存在该结构体
    /// 
    /// @param attr BaseUnitAttr,Unit段的属性抽象
    /// 
    /// @param val 属性值
    ///
    /// @return 成功则返回Ok(())，否则返回Err
    pub fn parse_and_set_base_unit_attribute(
        unit_part: &mut UnitPart,
        attr: &BaseUnitAttr,
        val: &str,
    ) -> Result<(), ParseError> {
        match attr {
            BaseUnitAttr::None => {
                return Err(ParseError::ESyntaxError);
            }
            BaseUnitAttr::Description => unit_part.description = String::from(val),
            BaseUnitAttr::Documentation => unit_part
                .documentation
                .extend(UnitParseUtil::parse_url(val)?),
            BaseUnitAttr::Requires => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入requires列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    unit_part
                        .requires
                        .push(UnitParseUtil::parse_unit_no_type(unit_path)?);
                }
            }
            BaseUnitAttr::Wants => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    unit_part
                        .wants
                        .push(UnitParseUtil::parse_unit_no_type(unit_path)?);
                }
            }
            BaseUnitAttr::After => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    unit_part
                        .after
                        .push(UnitParseUtil::parse_unit_no_type(unit_path)?);
                }
            }
            BaseUnitAttr::Before => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    unit_part
                        .before
                        .push(UnitParseUtil::parse_unit_no_type(unit_path)?);
                }
            }
            BaseUnitAttr::BindsTo => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    unit_part
                        .binds_to
                        .push(UnitParseUtil::parse_unit_no_type(unit_path)?);
                }
            }
            BaseUnitAttr::PartOf => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    unit_part
                        .part_of
                        .push(UnitParseUtil::parse_unit_no_type(unit_path)?);
                }
            }
            BaseUnitAttr::OnFailure => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    unit_part
                        .on_failure
                        .push(UnitParseUtil::parse_unit_no_type(unit_path)?);
                }
            }
            BaseUnitAttr::Conflicts => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    let unit = UnitParseUtil::parse_unit_no_type(unit_path)?;
                    unit_part.conflicts.push(unit);
                }
            }
        }
        return Ok(());
    }

    /// @brief 为Unit解析Install段属性并添加
    ///
    /// 为Unit解析Install段属性并添加
    ///
    /// @param install_part InstallPart(Install段结构体)，解析结果将存在该结构体
    /// 
    /// @param attr BaseUnitAttr,Unit段的属性抽象
    /// 
    /// @param val 属性值
    ///
    /// @return 成功则返回Ok(())，否则返回Err
    pub fn parse_and_set_base_install_attribute(
        install_part: &mut InstallPart,
        attr: &InstallUnitAttr,
        val: &str,
    ) -> Result<(), ParseError> {
        match attr {
            InstallUnitAttr::RequiredBy => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    let unit = UnitParseUtil::parse_unit::<TargetUnit>(unit_path)?;
                    install_part.requires_by.push(unit);
                }
            }
            InstallUnitAttr::Also => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    let unit = UnitParseUtil::parse_unit_no_type(unit_path)?;
                    install_part.also.push(unit);
                }
            }
            InstallUnitAttr::WantedBy => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    let unit = UnitParseUtil::parse_unit::<TargetUnit>(unit_path)?;
                    install_part.wanted_by.push(unit);
                }
            }
            InstallUnitAttr::Alias => {
                install_part.alias = String::from(val);
            }
            InstallUnitAttr::None => {
                return Err(ParseError::EINVAL);
            }
        }
        return Ok(());
    }
}
