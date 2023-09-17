use crate::error::ParseError;
use crate::parse::Segment;
//use drstd as std;
use std::any::Any;
use std::boxed::Box;
use std::rc::Rc;
use std::string::String;
use std::vec::Vec;

pub mod service;
pub mod target;

use self::target::TargetUnit;

//所有可解析的Unit都应该实现该trait
pub trait Unit {
    /// @brief 从文件获取到Unit,该函数是解析Unit文件的入口函数
    ///
    /// 从path解析Unit属性
    ///
    /// @param path 需解析的文件
    ///
    /// @return 解析成功则返回对应Unit的Rc指针，否则返回Err
    fn from_path(path: &str) -> Result<Rc<Self>, ParseError>
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
}

//Unit状态
#[derive(Clone, Copy, Debug)]
enum UnitState {
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
pub struct BaseUnit {
    pub unit_part: UnitPart,
    pub install_part: InstallPart,
    pub state: UnitState,
    pub unit_type: UnitType,
}

impl Default for BaseUnit {
    fn default() -> Self {
        BaseUnit {
            unit_part: UnitPart::default(),
            install_part: InstallPart::default(),
            state: UnitState::Disabled,
            unit_type: UnitType::Unknown,
        }
    }
}

impl BaseUnit {
    pub fn parse_and_set_attribute(&self) -> Result<(), ParseError> {
        return Ok(());
    }
}

#[derive(Default)]
pub struct Url {
    pub url_string: String, // pub protocol: String,
                            // pub host: String,
                            // pub port: Option<u16>,
                            // pub path: String,
                            // pub query: Option<String>,
                            // pub fragment: Option<String>,
}

//对应Unit文件的Unit段
pub struct UnitPart {
    pub description: String,
    pub documentation: Vec<Url>,
    pub requires: Vec<Rc<dyn Unit>>,
    pub wants: Vec<Rc<dyn Unit>>,
    pub after: Vec<Rc<dyn Unit>>,
    pub before: Vec<Rc<dyn Unit>>,
    pub binds_to: Vec<Rc<dyn Unit>>,
    pub part_of: Vec<Rc<dyn Unit>>,
    pub on_failure: Vec<Rc<dyn Unit>>,
    pub conflicts: Vec<Rc<dyn Unit>>,
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

//对应Unit文件的Install段
pub struct InstallPart {
    pub wanted_by: Vec<Rc<TargetUnit>>,
    pub requires_by: Vec<Rc<TargetUnit>>,
    pub also: Vec<Rc<dyn Unit>>,
    pub alias: String,
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
