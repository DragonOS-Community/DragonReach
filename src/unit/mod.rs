use crate::error::parse_error::ParseError;
use crate::error::parse_error::ParseErrorType;
use crate::error::runtime_error::RuntimeError;
use crate::error::runtime_error::RuntimeErrorType;
use crate::executor::ExitStatus;
use crate::parse::parse_util::UnitParseUtil;
use crate::parse::Segment;

#[cfg(target_os = "dragonos")]
use drstd as std;
use hashbrown::HashMap;

use std::any::Any;
use std::default::Default;
use std::fmt::Debug;
use std::marker::{Send, Sized, Sync};
use std::option::Option::Some;
use std::result::Result;
use std::result::Result::Err;
use std::result::Result::Ok;
use std::string::String;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::vec::Vec;

pub mod service;
pub mod target;

use self::target::TargetUnit;
use lazy_static::lazy_static;

pub fn generate_unit_id() -> usize {
    static UNIT_ID: AtomicUsize = AtomicUsize::new(1);
    return UNIT_ID.fetch_add(1, Ordering::SeqCst);
}

//所有可解析的Unit都应该实现该trait
pub trait Unit: Sync + Send + Debug {
    /// @brief 从文件获取到Unit,该函数是解析Unit文件的入口函数
    ///
    /// 从path解析Unit属性
    ///
    /// @param path 需解析的文件
    ///
    /// @return 解析成功则返回对应Unit的id，否则返回Err
    fn from_path(path: &str) -> Result<usize, ParseError>
    where
        Self: Sized;

    fn as_any(&self) -> &dyn Any;

    /// @brief 设置Unit属性
    ///
    /// 设置对应Unit属性
    ///
    /// @param segment  属性段类型
    ///
    /// @param attr     属性名
    ///
    /// @param val      属性值
    ///
    /// @return 设置成功则返回Ok(())，否则返回Err
    fn set_attr(&mut self, segment: Segment, attr: &str, val: &str) -> Result<(), ParseError>;

    /// # 设置每个Unit都应该有的属性
    ///
    /// 设置BaseUnit
    ///
    /// ## param unit_base  设置值
    fn set_unit_base(&mut self, unit_base: BaseUnit);

    /// # 获取UnitType
    ///
    /// ## return UnitType
    fn unit_type(&self) -> UnitType;

    fn unit_base(&self) -> &BaseUnit;

    fn mut_unit_base(&mut self) -> &mut BaseUnit;

    fn unit_id(&self) -> usize;

    /// ## Unit的工作逻辑
    ///
    /// ### return OK(())/Err
    fn run(&mut self) -> Result<(), RuntimeError>;

    /// ## 设置unit_id
    ///
    /// ### return OK(())/Err
    fn set_unit_id(&mut self) -> usize {
        let ret = generate_unit_id();
        self.mut_unit_base().set_id(ret);
        ret
    }

    /// ## Unit退出后逻辑
    /// 
    /// 一般只有可运行的Unit(如Service)需要重写此函数
    fn after_exit(&mut self,exit_status: ExitStatus){

    }
}

//Unit状态
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UnitState {
    Enabled,
    Disabled,
    Static,
    Masked,
}

//Unit类型
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum UnitType {
    Automount,
    Device,
    Mount,
    Path,
    Scope,
    Service,
    Slice,
    Snapshot,
    Socket,
    Swap,
    Target,
    Timer,
    Unknown,
}

//记录unit文件基本信息，这个结构体里面的信息是所有Unit文件都可以有的属性
#[derive(Debug, Clone)]
pub struct BaseUnit {
    unit_part: UnitPart,
    install_part: InstallPart,
    state: UnitState,
    unit_type: UnitType,
    unit_id: usize,
}

impl Default for BaseUnit {
    fn default() -> Self {
        BaseUnit {
            unit_part: UnitPart::default(),
            install_part: InstallPart::default(),
            state: UnitState::Disabled,
            unit_type: UnitType::Unknown,
            unit_id: 0,
        }
    }
}

impl BaseUnit {
    pub fn set_state(&mut self, state: UnitState) {
        self.state = state;
    }

    pub fn set_unit_type(&mut self, utype: UnitType) {
        self.unit_type = utype;
    }

    pub fn set_unit_part_attr(
        &mut self,
        attr_type: &BaseUnitAttr,
        val: &str,
    ) -> Result<(), ParseError> {
        return self.unit_part.set_attr(attr_type, val);
    }

    pub fn set_install_part_attr(
        &mut self,
        attr_type: &InstallUnitAttr,
        val: &str,
    ) -> Result<(), ParseError> {
        return self.install_part.set_attr(attr_type, val);
    }

    pub fn parse_and_set_attribute(&self) -> Result<(), ParseError> {
        return Ok(());
    }

    pub fn unit_part(&self) -> &UnitPart {
        &self.unit_part
    }

    pub fn install_part(&self) -> &InstallPart {
        &self.install_part
    }

    pub fn state(&self) -> &UnitState {
        &self.state
    }

    pub fn unit_type(&self) -> &UnitType {
        &self.unit_type
    }

    pub fn set_id(&mut self, id: usize) {
        self.unit_id = id;
    }
}

#[derive(Default, Debug, Clone)]
pub struct Url {
    pub url_string: String, // pub protocol: String,
                            // pub host: String,
                            // pub port: Option<u16>,
                            // pub path: String,
                            // pub query: Option<String>,
                            // pub fragment: Option<String>,
}

//对应Unit文件的Unit段
#[derive(Debug, Clone)]
pub struct UnitPart {
    description: String,
    documentation: Vec<Url>,
    requires: Vec<usize>,
    wants: Vec<usize>,
    after: Vec<usize>,
    before: Vec<usize>,
    binds_to: Vec<usize>,
    part_of: Vec<usize>,
    on_failure: Vec<usize>,
    conflicts: Vec<usize>,
}

impl Default for UnitPart {
    fn default() -> Self {
        UnitPart {
            description: String::new(),
            documentation: Vec::new(),
            requires: Vec::new(),
            wants: Vec::new(),
            after: Vec::new(),
            before: Vec::new(),
            binds_to: Vec::new(),
            part_of: Vec::new(),
            on_failure: Vec::new(),
            conflicts: Vec::new(),
        }
    }
}

impl UnitPart {
    pub fn set_attr(&mut self, attr: &BaseUnitAttr, val: &str) -> Result<(), ParseError> {
        match attr {
            BaseUnitAttr::None => {
                return Err(ParseError::new(
                    ParseErrorType::ESyntaxError,
                    String::new(),
                    0,
                ));
            }
            BaseUnitAttr::Description => self.description = String::from(val),
            BaseUnitAttr::Documentation => {
                self.documentation.extend(UnitParseUtil::parse_url(val)?)
            }
            BaseUnitAttr::Requires => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入requires列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    self.requires
                        .push(UnitParseUtil::parse_unit_no_type(unit_path)?);
                }
            }
            BaseUnitAttr::Wants => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    self.wants
                        .push(UnitParseUtil::parse_unit_no_type(unit_path)?);
                }
            }
            BaseUnitAttr::After => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    self.after
                        .push(UnitParseUtil::parse_unit_no_type(unit_path)?);
                }
            }
            BaseUnitAttr::Before => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    self.before
                        .push(UnitParseUtil::parse_unit_no_type(unit_path)?);
                }
            }
            BaseUnitAttr::BindsTo => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    self.binds_to
                        .push(UnitParseUtil::parse_unit_no_type(unit_path)?);
                }
            }
            BaseUnitAttr::PartOf => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    self.part_of
                        .push(UnitParseUtil::parse_unit_no_type(unit_path)?);
                }
            }
            BaseUnitAttr::OnFailure => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    self.on_failure
                        .push(UnitParseUtil::parse_unit_no_type(unit_path)?);
                }
            }
            BaseUnitAttr::Conflicts => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    let unit = UnitParseUtil::parse_unit_no_type(unit_path)?;
                    self.conflicts.push(unit);
                }
            }
        }
        return Ok(());
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn documentation(&self) -> &[Url] {
        &self.documentation
    }

    pub fn requires(&self) -> &[usize] {
        &self.requires
    }

    pub fn wants(&self) -> &[usize] {
        &self.wants
    }

    pub fn after(&self) -> &[usize] {
        &self.after
    }

    pub fn before(&self) -> &[usize] {
        &self.before
    }

    pub fn binds_to(&self) -> &[usize] {
        &self.binds_to
    }

    pub fn part_of(&self) -> &[usize] {
        &self.part_of
    }

    pub fn on_failure(&self) -> &[usize] {
        &self.on_failure
    }

    pub fn conflicts(&self) -> &[usize] {
        &self.conflicts
    }
}

//对应Unit文件的Install段
#[derive(Debug, Clone)]
pub struct InstallPart {
    wanted_by: Vec<usize>,
    requires_by: Vec<usize>,
    also: Vec<usize>,
    alias: String,
}

impl Default for InstallPart {
    fn default() -> Self {
        InstallPart {
            wanted_by: Vec::new(),
            requires_by: Vec::new(),
            also: Vec::new(),
            alias: String::new(),
        }
    }
}

impl InstallPart {
    pub fn set_attr(&mut self, attr: &InstallUnitAttr, val: &str) -> Result<(), ParseError> {
        match attr {
            InstallUnitAttr::RequiredBy => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    let unit = UnitParseUtil::parse_unit::<TargetUnit>(unit_path)?;
                    self.requires_by.push(unit);
                }
            }
            InstallUnitAttr::Also => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    let unit = UnitParseUtil::parse_unit_no_type(unit_path)?;
                    self.also.push(unit);
                }
            }
            InstallUnitAttr::WantedBy => {
                let units = val.split_whitespace().collect::<Vec<&str>>();
                //TODO:目前先加入列表，可能会出现循环依赖问题，后续应解决循环依赖问题
                for unit_path in units {
                    let unit = UnitParseUtil::parse_unit::<TargetUnit>(unit_path)?;
                    self.wanted_by.push(unit);
                }
            }
            InstallUnitAttr::Alias => {
                self.alias = String::from(val);
            }
            InstallUnitAttr::None => {
                return Err(ParseError::new(ParseErrorType::EINVAL, String::new(), 0));
            }
        }
        return Ok(());
    }

    pub fn wanted_by(&self) -> &[usize] {
        &self.wanted_by
    }

    pub fn requires_by(&self) -> &[usize] {
        &self.requires_by
    }

    pub fn also(&self) -> &[usize] {
        &self.also
    }

    pub fn alias(&self) -> &str {
        &self.alias
    }
}
//对应Unit文件的各种属性
pub enum BaseUnitAttr {
    None,

    //Unit段
    //描述该Unit文件的信息
    Description,
    //指定服务文档
    Documentation,
    //依赖的其它 Unit 列表
    Requires,
    //这个 Unit 启动时，触发启动列出的每个 Unit 模块，而不去考虑这些模板启动是否成功
    Wants,
    //后面列出的所有模块全部启动完成以后，才会启动当前的服务
    After,
    //在启动指定的任务一个模块之间，都会首先确证当前服务已经运行
    Before,
    //这些Unit启动失败时该任务失败，都成功时该任务成功，在这些模板中有任意一个出现意外结束或重启时，这个服务也会跟着终止或重启
    BindsTo,
    //仅在列出的任务模块失败或重启时，终止或重启当前服务，而不会随列出模板的启动而启动
    PartOf,
    //当这个模板启动失败时，就会自动启动列出的每个模块
    OnFailure,
    //与这个模块有冲突的模块，如果列出的模块中有已经在运行的，这个服务就不能启动，反之亦然
    Conflicts,
}

pub enum InstallUnitAttr {
    None,
    //Install段
    //依赖当前服务的模块。当前 Unit 激活时（enable）符号链接会放入 /etc/systemd/system 目录下面以 <Target 名> + .wants 后缀构成的子目录中
    WantedBy,
    //依赖当前服务的模块。当前 Unit 激活时（enable）符号链接会放入 /etc/systemd/system 目录下面以 <Target 名> + .required 后缀构成的子目录中
    RequiredBy,
    //当前 Unit enable/disable 时，同时 enable/disable 的其他 Unit
    Also,
    //别名
    Alias,
}
