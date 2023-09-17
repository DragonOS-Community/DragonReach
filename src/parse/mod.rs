use crate::unit::{BaseUnit, Unit};
use crate::{
    error::ParseError,
    unit::{service::ServiceUnitAttr, BaseUnitAttr, InstallUnitAttr, UnitType},
};
use core::cell::RefCell;
//use drstd as std;
use hashbrown::HashMap;
use lazy_static::lazy_static;
use std::boxed::Box;
use std::format;
use std::fs::File;
use std::io::{self, BufRead};
use std::rc::Rc;
use std::string::String;
use std::vec::Vec;

use self::parse_base_unit::BaseUnitParser;

pub mod parse_base_unit;
pub mod parse_service;
mod parse_util;

pub trait AttrParse<T> {
    fn parse_and_set_attribute(unit: &mut T, attr: &str, val: &str) -> Result<(), ParseError>;
}

//对应Unit段类型
#[derive(PartialEq, Clone, Copy)]
pub enum Segment {
    None,
    Unit,
    Install,
    Service,
}

lazy_static! {
    static ref UNIT_SUFFIX: HashMap<&'static str, UnitType> = {
        let mut table = HashMap::new();
        table.insert("automount", UnitType::Automount);
        table.insert("device", UnitType::Device);
        table.insert("mount", UnitType::Mount);
        table.insert("path", UnitType::Path);
        table.insert("scope", UnitType::Scope);
        table.insert("service", UnitType::Service);
        table.insert("slice", UnitType::Automount);
        table.insert("automount", UnitType::Slice);
        table.insert("socket", UnitType::Socket);
        table.insert("swap", UnitType::Swap);
        table.insert("target", UnitType::Target);
        table.insert("timer", UnitType::Timer);
        table
    };
    static ref SEGMENT_TABLE: HashMap<&'static str, Segment> = {
        let mut table = HashMap::new();
        table.insert("[Unit]", Segment::Unit);
        table.insert("[Install]", Segment::Install);
        table.insert("[Service]", Segment::Service);
        table
    };
    static ref INSTALL_UNIT_ATTR_TABLE: HashMap<&'static str, InstallUnitAttr> = {
        let mut unit_attr_table = HashMap::new();
        unit_attr_table.insert("WantedBy", InstallUnitAttr::WantedBy);
        unit_attr_table.insert("RequiredBy", InstallUnitAttr::RequiredBy);
        unit_attr_table.insert("Also", InstallUnitAttr::Also);
        unit_attr_table.insert("Alias", InstallUnitAttr::Alias);
        unit_attr_table
    };
    static ref SERVICE_UNIT_ATTR_TABLE: HashMap<&'static str, ServiceUnitAttr> = {
        let mut unit_attr_table = HashMap::new();
        unit_attr_table.insert("Type", ServiceUnitAttr::Type);
        unit_attr_table.insert("RemainAfterExit", ServiceUnitAttr::RemainAfterExit);
        unit_attr_table.insert("ExecStart", ServiceUnitAttr::ExecStart);
        unit_attr_table.insert("ExecStartPre", ServiceUnitAttr::ExecStartPre);
        unit_attr_table.insert("ExecStartPos", ServiceUnitAttr::ExecStartPos);
        unit_attr_table.insert("ExecReload", ServiceUnitAttr::ExecReload);
        unit_attr_table.insert("ExecStop", ServiceUnitAttr::ExecStop);
        unit_attr_table.insert("ExecStopPost", ServiceUnitAttr::ExecStopPost);
        unit_attr_table.insert("RestartSec", ServiceUnitAttr::RestartSec);
        unit_attr_table.insert("Restart", ServiceUnitAttr::Restart);
        unit_attr_table.insert("TimeoutStartSec", ServiceUnitAttr::TimeoutStartSec);
        unit_attr_table.insert("TimeoutStopSec", ServiceUnitAttr::TimeoutStopSec);
        unit_attr_table.insert("Environment", ServiceUnitAttr::Environment);
        unit_attr_table.insert("EnvironmentFile", ServiceUnitAttr::EnvironmentFile);
        unit_attr_table.insert("Nice", ServiceUnitAttr::Nice);
        unit_attr_table.insert("WorkingDirectory", ServiceUnitAttr::WorkingDirectory);
        unit_attr_table.insert("RootDirectory", ServiceUnitAttr::RootDirectory);
        unit_attr_table.insert("User", ServiceUnitAttr::User);
        unit_attr_table.insert("Group", ServiceUnitAttr::Group);
        unit_attr_table.insert("MountFlags", ServiceUnitAttr::MountFlags);
        unit_attr_table
    };
    static ref BASE_UNIT_ATTR_TABLE: HashMap<&'static str, BaseUnitAttr> = {
        let mut unit_attr_table = HashMap::new();
        unit_attr_table.insert("Description", BaseUnitAttr::Description);
        unit_attr_table.insert("Documentation", BaseUnitAttr::Documentation);
        unit_attr_table.insert("Requires", BaseUnitAttr::Requires);
        unit_attr_table.insert("Wants", BaseUnitAttr::Wants);
        unit_attr_table.insert("After", BaseUnitAttr::After);
        unit_attr_table.insert("Before", BaseUnitAttr::Before);
        unit_attr_table.insert("Binds To", BaseUnitAttr::BindsTo);
        unit_attr_table.insert("Part Of", BaseUnitAttr::PartOf);
        unit_attr_table.insert("OnFailure", BaseUnitAttr::OnFailure);
        unit_attr_table.insert("Conflicts", BaseUnitAttr::Conflicts);
        unit_attr_table
    };
    static ref BASE_IEC: HashMap<&'static str, u64> = {
        let mut table = HashMap::new();
        table.insert(
            "E",
            1024u64 * 1024u64 * 1024u64 * 1024u64 * 1024u64 * 1024u64,
        );
        table.insert("P", 1024u64 * 1024u64 * 1024u64 * 1024u64 * 1024u64);
        table.insert("T", 1024u64 * 1024u64 * 1024u64 * 1024u64);
        table.insert("G", 1024u64 * 1024u64 * 1024u64);
        table.insert("M", 1024u64 * 1024u64);
        table.insert("K", 1024u64);
        table.insert("B", 1u64);
        table.insert("", 1u64);
        table
    };
    static ref BASE_SI: HashMap<&'static str, u64> = {
        let mut table = HashMap::new();
        table.insert(
            "E",
            1000u64 * 1000u64 * 1000u64 * 1000u64 * 1000u64 * 1000u64,
        );
        table.insert("P", 1000u64 * 1000u64 * 1000u64 * 1000u64 * 1000u64);
        table.insert("T", 1000u64 * 1000u64 * 1000u64 * 1000u64);
        table.insert("G", 1000u64 * 1000u64 * 1000u64);
        table.insert("M", 1000u64 * 1000u64);
        table.insert("K", 1000u64);
        table.insert("B", 1u64);
        table.insert("", 1u64);
        table
    };
    static ref SEC_UNIT_TABLE: HashMap<&'static str, u64> = {
        let mut table = HashMap::new();
        table.insert("h", 60 * 60 * 1000 * 1000 * 1000);
        table.insert("min", 60 * 1000 * 1000 * 1000);
        table.insert("m", 60 * 1000 * 1000 * 1000);
        table.insert("s", 1000 * 1000 * 1000);
        table.insert("", 1000 * 1000 * 1000);
        table.insert("ms", 1000 * 1000);
        table.insert("us", 1000);
        table.insert("ns", 1);
        table
    };
}

//用于解析Unit共有段的方法
pub struct UnitParser;

impl UnitParser {
    /// @brief 从path获取到BufReader,此方法将会检验文件类型
    ///
    /// 从path获取到BufReader,此方法将会检验文件类型
    ///
    /// @param path 需解析的文件路径
    /// 
    /// @param unit_type 指定Unit类型
    ///
    /// @return 成功则返回对应BufReader，否则返回Err
    fn get_unit_reader(path: &str, unit_type: UnitType) -> Result<io::BufReader<File>, ParseError> {
        let suffix = match path.rfind('.') {
            Some(idx) => &path[idx + 1..],
            None => {
                return Err(ParseError::EINVAL);
            }
        };
        let u_type = UNIT_SUFFIX.get(suffix);
        if u_type.is_none() {
            return Err(ParseError::EFILE);
        }
        if *(u_type.unwrap()) != unit_type {
            return Err(ParseError::EFILE);
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => {
                return Err(ParseError::EINVAL);
            }
        };
        return Ok(io::BufReader::new(file));
    }

    /// @brief 将path路径的文件解析为unit_type类型的Unit
    ///
    /// 该方法解析每个Unit共有的段(Unit,Install),其余独有的段属性将会交付T类型的Unit去解析
    /// TODO:该方法多态性做得不是很优雅，后期可以重构一下
    ///
    /// @param path 需解析的文件路径
    /// 
    /// @param unit_type 指定Unit类型
    /// 
    /// @param unit 需要的unit对象,解析结果会放置在里面,将会调用unit的set_attr方法设置独有属性
    /// 
    /// @param unit_base 共有段的解析结果将会放置在unit_base中
    ///
    /// @return 解析成功则返回Ok(())，否则返回Err
    pub fn parse<T: Unit>(
        path: &str,
        unit_type: UnitType,
        unit: &mut T,
        unit_base: &mut BaseUnit,
    ) -> Result<(), ParseError> {
        unit_base.unit_type = unit_type;
        let reader = UnitParser::get_unit_reader(path, unit_type)?;

        //用于记录当前段的类型
        let mut segment = Segment::None;
        //用于处理多行对应一个属性的情况
        let mut last_attr = ServiceUnitAttr::None;

        //一行一行向下解析
        let lines = reader
            .lines()
            .map(|line| line.unwrap())
            .collect::<Vec<String>>();
        let mut i = 0;
        while i < lines.len() {
            let line = &lines[i];
            //空行跳过
            if line.chars().all(char::is_whitespace) {
                i += 1;
                continue;
            }
            //注释跳过
            if line.starts_with('#') {
                i += 1;
                continue;
            }
            let mut line = line.trim();
            let segment_flag = SEGMENT_TABLE.get(&line);
            if !segment_flag.is_none() {
                //如果当前行匹配到的为段名，则切换段类型继续匹配下一行
                segment = *segment_flag.unwrap();
                i += 1;
                continue;
            }
            if segment == Segment::None {
                //未找到段名则不能继续匹配
                return Err(ParseError::ESyntaxError);
            }

            //下面进行属性匹配
            //合并多行为一个属性的情况
            //最后一个字符为\，代表换行，将多行转换为一行统一解析
            if lines[i].ends_with('\\') {
                let mut templine = String::new();
                while lines[i].ends_with('\\') {
                    let temp = &lines[i][..lines[i].len() - 1];
                    templine = format!("{} {}", templine, temp);
                    i += 1;
                }
                templine = format!("{} {}", templine, lines[i]);
                line = templine.as_str();
                i += 1;
                break;
            }
            //=号分割后第一个元素为属性，后面的均为值，若一行出现两个等号则是语法错误
            let attr_val_map = line.split('=').collect::<Vec<&str>>();
            if attr_val_map.len() != 2 {
                return Err(ParseError::ESyntaxError);
            }

            //首先匹配所有unit文件都有的unit段和install段
            if BASE_UNIT_ATTR_TABLE.get(attr_val_map[0]).is_some() {
                if segment != Segment::Unit {
                    return Err(ParseError::EINVAL);
                }
                BaseUnitParser::parse_and_set_base_unit_attribute(
                    &mut unit_base.unit_part,
                    BASE_UNIT_ATTR_TABLE.get(attr_val_map[0]).unwrap(),
                    attr_val_map[1],
                )?
            } else if INSTALL_UNIT_ATTR_TABLE.get(attr_val_map[0]).is_some() {
                if segment != Segment::Install {
                    return Err(ParseError::EINVAL);
                }
                BaseUnitParser::parse_and_set_base_install_attribute(
                    &mut unit_base.install_part,
                    INSTALL_UNIT_ATTR_TABLE.get(attr_val_map[0]).unwrap(),
                    attr_val_map[1],
                )?
            } else {
                unit.set_attr(segment, attr_val_map[0], attr_val_map[1])?;
            }
            i += 1;
        }
        return Ok(());
    }
}
