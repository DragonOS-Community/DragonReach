use super::{AttrParse, ServiceUnitAttr, UnitParser, SERVICE_UNIT_ATTR_TABLE};
use crate::error::ParseError;
use crate::parse::parse_util::UnitParseUtil;
use crate::unit::service::{MountFlag, RestartOption, ServiceType, ServiceUnit};
use crate::unit::{BaseUnit, Unit};
use core::borrow::BorrowMut;
use core::cell::RefCell;
//use drstd as std;
use std::boxed::Box;
use std::fs;
use std::rc::Rc;
use std::string::String;
pub struct ServiceParser;

impl ServiceParser {
    /// @brief 解析Service类型Unit的
    ///
    /// 从path解析Service类型Unit
    ///
    /// @param path 需解析的文件路径
    ///
    /// @return 成功则返回Ok(Rc<ServiceUnit>)，否则返回Err
    pub fn parse(path: &str) -> Result<Rc<ServiceUnit>, ParseError> {
        let mut service = ServiceUnit::default();
        let mut unit_base = BaseUnit::default();

        //交付总解析器
        UnitParser::parse::<ServiceUnit>(
            path,
            crate::unit::UnitType::Service,
            &mut service,
            &mut unit_base,
        )?;

        //设置
        service.unit_base = unit_base;
        let mut service = Rc::new(service);
        return Ok(service);
    }
}

impl AttrParse<ServiceUnit> for ServiceParser {
    /// @brief 为Service类型Unit解析并添加属性
    ///
    /// 为Service类型Unit解析并添加属性
    ///
    /// @param service ServiceUnit对象
    /// 
    /// @param attr_str 属性名
    /// 
    /// @param val 属性值
    ///
    /// @return 成功则返回Ok(())，否则返回Err
    fn parse_and_set_attribute(
        service: &mut ServiceUnit,
        attr_str: &str,
        val: &str,
    ) -> Result<(), ParseError> {
        //let mut service = *unit;

        let attr = match SERVICE_UNIT_ATTR_TABLE.get(attr_str) {
            Some(val) => val,
            None => {
                return Err(ParseError::EINVAL);
            }
        };
        match attr {
            ServiceUnitAttr::Type => match val {
                "simple" => service.service_part.service_type = ServiceType::Simple,
                "forking" => service.service_part.service_type = ServiceType::Forking,
                "oneshot" => service.service_part.service_type = ServiceType::OneShot,
                "dbus" => service.service_part.service_type = ServiceType::Dbus,
                "notify" => service.service_part.service_type = ServiceType::Notify,
                "idle" => service.service_part.service_type = ServiceType::Idle,
                _ => {
                    return Err(ParseError::EINVAL);
                }
            },
            ServiceUnitAttr::RemainAfterExit => {
                service.service_part.remain_after_exit = UnitParseUtil::parse_boolean(val)?
            }
            ServiceUnitAttr::ExecStart => {
                service
                    .service_part
                    .exec_start
                    .extend(UnitParseUtil::parse_cmd_task(val)?);
            }
            ServiceUnitAttr::ExecStartPre => {
                service
                    .service_part
                    .exec_start_pre
                    .extend(UnitParseUtil::parse_cmd_task(val)?);
            }
            ServiceUnitAttr::ExecStartPos => {
                service
                    .service_part
                    .exec_start_pos
                    .extend(UnitParseUtil::parse_cmd_task(val)?);
            }
            ServiceUnitAttr::ExecReload => {
                service
                    .service_part
                    .exec_reload
                    .extend(UnitParseUtil::parse_cmd_task(val)?);
            }
            ServiceUnitAttr::ExecStopPost => {
                service
                    .service_part
                    .exec_stop_post
                    .extend(UnitParseUtil::parse_cmd_task(val)?);
            }
            ServiceUnitAttr::ExecStop => {
                service
                    .service_part
                    .exec_stop
                    .extend(UnitParseUtil::parse_cmd_task(val)?);
            }
            ServiceUnitAttr::RestartSec => {
                service.service_part.restart_sec = UnitParseUtil::parse_sec(val)?
            }
            ServiceUnitAttr::Restart => match val {
                "always" => service.service_part.restart = RestartOption::AlwaysRestart,
                "on-success" => service.service_part.restart = RestartOption::OnSuccess,
                "on-failure" => service.service_part.restart = RestartOption::OnFailure,
                "on-abnormal" => service.service_part.restart = RestartOption::OnAbnormal,
                "on-abort" => service.service_part.restart = RestartOption::OnAbort,
                "on-watchdog" => service.service_part.restart = RestartOption::OnWatchdog,
                _ => {
                    return Err(ParseError::EINVAL);
                }
            },
            ServiceUnitAttr::TimeoutStartSec => {
                service.service_part.timeout_start_sec = UnitParseUtil::parse_sec(val)?
            }
            ServiceUnitAttr::TimeoutStopSec => {
                service.service_part.timeout_stop_sec = UnitParseUtil::parse_sec(val)?
            }
            ServiceUnitAttr::Environment => {
                service.service_part.environment = String::from(val);
            }
            ServiceUnitAttr::EnvironmentFile => {
                if !UnitParseUtil::is_absolute_path(val) {
                    return Err(ParseError::EFILE);
                }
                service.service_part.environment_file = String::from(val);
            }
            ServiceUnitAttr::Nice => {
                service.service_part.nice = UnitParseUtil::parse_nice(val)?;
            }
            ServiceUnitAttr::WorkingDirectory => {
                if !UnitParseUtil::is_dir(val) {
                    return Err(ParseError::ENODIR);
                }
                service.service_part.working_directory = String::from(val);
            }
            ServiceUnitAttr::User => {
                //TODO: 检查系统是否存在这个用户
                service.service_part.user = String::from(val);
            }
            ServiceUnitAttr::Group => {
                //TODO: 检查系统是否存在该用户组
                service.service_part.group = String::from(val);
            }
            ServiceUnitAttr::MountFlags => match val {
                "shared" => service.service_part.mount_flags = MountFlag::Shared,
                "slave" => service.service_part.mount_flags = MountFlag::Slave,
                "private" => service.service_part.mount_flags = MountFlag::Private,
                _ => {
                    return Err(ParseError::EINVAL);
                }
            },
            _ => {
                return Err(ParseError::EINVAL);
            }
        }

        return Ok(());
    }
}
