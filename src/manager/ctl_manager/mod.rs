use lazy_static::lazy_static;
use std::fs::File;
use std::os::fd::FromRawFd;
use std::string::String;
use std::string::ToString;
use std::sync::Arc;
use std::{eprint, eprintln, format};
use std::{libc, print, println};
use std::{sync::Mutex, vec::Vec};

use crate::error::runtime_error::RuntimeError;
use crate::error::runtime_error::RuntimeErrorType;
use crate::error::ErrorFormat;
use crate::parse::parse_util::UnitParseUtil;
use crate::systemctl::ctl_parser::Pattern;
use crate::systemctl::ctl_path;
use crate::systemctl::listener::Command;
use crate::unit::Unit;
use crate::unit::UnitState;

use super::{UnitManager, ID_TO_UNIT_MAP};
pub struct CtlManager;

lazy_static! {
    static ref CTL_WRITER: Mutex<Arc<File>> = {
        let file = CtlManager::init_ctl_writer();
        Mutex::new(Arc::new(file))
    };
}

impl CtlManager {
    pub fn exec_ctl(cmd: Command) -> Result<(), RuntimeError> {
        // TODO:目前假设一个时刻只有一个进程使用systemdctl,后续应该使用DBus等更灵活的进程通信方式
        match cmd.operation {
            crate::systemctl::ctl_parser::CommandOperation::ListUnits => {
                Self::list_unit(cmd.patterns)
            }
            crate::systemctl::ctl_parser::CommandOperation::Start => Self::start(cmd.args.unwrap()),
            crate::systemctl::ctl_parser::CommandOperation::Restart => {
                Self::restart(cmd.args.unwrap(), false)
            }
            crate::systemctl::ctl_parser::CommandOperation::Stop => Self::stop(cmd.args.unwrap()),
            crate::systemctl::ctl_parser::CommandOperation::Reboot => Ok(Self::reboot()),
            crate::systemctl::ctl_parser::CommandOperation::ListSockets => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::ListTimers => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Reload => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::TryRestart => {
                Self::restart(cmd.args.unwrap(), true)
            }
            crate::systemctl::ctl_parser::CommandOperation::ReloadOrRestart => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::ReloadOrTryRestart => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Isolate => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Kill => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::IsActive => {
                let mut patterns = cmd.patterns.clone();
                patterns.push(Pattern::State(UnitState::Active));
                Self::list_unit(patterns)
            }
            crate::systemctl::ctl_parser::CommandOperation::IsFailed => {
                let mut patterns = cmd.patterns.clone();
                patterns.push(Pattern::State(UnitState::Failed));
                Self::list_unit(patterns)
            }
            crate::systemctl::ctl_parser::CommandOperation::Status => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Show => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Cat => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::SetProperty => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Help => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::ResetFailed => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::ListDependencies => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::ListUnitFiles => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Enable => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Disable => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Reenable => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Preset => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::PresetAll => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::IsEnabled => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Mask => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::UnMask => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Link => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::AddWants => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::AddRequires => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Edit => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::GetDefault => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::SetDefault => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::ListMachines => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::ListJobs => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Cancel => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Snapshot => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Delete => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::ShowEnvironment => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::SetEnvironment => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::UnsetEnvironment => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::ImportEnvironment => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::DeamonReload => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::DeamonReexec => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::IsSystemRunning => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Default => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Rescue => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Emergency => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Halt => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Poweroff => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Kexec => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Exit => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::SwitchRoot => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Suspend => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::Hibernate => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::HybridSleep => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::UnSupported => todo!(),
            crate::systemctl::ctl_parser::CommandOperation::None => {
                println!("No such command!");
                return Err(RuntimeError::new(RuntimeErrorType::InvalidInput));
            }
        }
    }

    pub fn list_unit(pattern: Vec<Pattern>) -> Result<(), RuntimeError> {
        let units = Self::filter_units(pattern)?;

        let mut res = "UNIT\t\t\t\tLOAD\t\tACTIVE\t\tSUB\t\tDESCRIPTION".to_string();
        res.push_str("\n----------------------------------------------------------------------------------------------");
        for unit in units {
            res = format!("{}\n{}", res, unit.lock().unwrap().unit_base().unit_info());
        }

        // if let Err(err) = CTL_WRITER.lock().unwrap().write_all(res.as_bytes()) {
        //     eprintln!("write ctl error :{}", err);
        // }
        println!("{}", res);
        Ok(())
    }

    pub fn stop(names: Vec<String>) -> Result<(), RuntimeError> {
        // TODO:打日志
        for name in names {
            match UnitManager::get_unit_with_name(&name) {
                Some(unit) => {
                    unit.lock().unwrap().exit();
                }
                None => {
                    eprintln!("{} is not a unit", name);
                    return Err(RuntimeError::new(RuntimeErrorType::FileNotFound));
                }
            }
        }
        Ok(())
    }

    pub fn start(names: Vec<String>) -> Result<(), RuntimeError> {
        // TODO:打日志
        for name in names {
            match UnitManager::get_unit_with_name(&name) {
                Some(unit) => unit.lock().unwrap().run()?,
                None => match UnitParseUtil::parse_unit_no_type(&name) {
                    Ok(i) => {
                        let unit = UnitManager::get_unit_with_id(&i).unwrap();
                        let mut unit = unit.lock().unwrap();
                        unit.run()?;
                    }
                    Err(err) => {
                        eprintln!("parse unit {} error :{}", name, err.error_format());
                    }
                },
            }
        }
        Ok(())
    }

    pub fn reboot() {
        #[cfg(target_os = "dragonos")]
        unsafe {
            dsc::syscall!(SYS_REBOOT)
        };
    }

    pub fn restart(names: Vec<String>, is_try: bool) -> Result<(), RuntimeError> {
        // TODO:打日志
        for name in names {
            match UnitManager::get_unit_with_name(&name) {
                Some(unit) => {
                    let mut unit = unit.lock().unwrap();
                    if is_try && *unit.unit_base().state() == UnitState::Active {
                        unit.restart()?;
                    } else {
                        unit.restart()?;
                    }
                }
                None => match UnitParseUtil::parse_unit_no_type(&name) {
                    Ok(i) => {
                        let unit = UnitManager::get_unit_with_id(&i).unwrap();
                        unit.lock().unwrap().run()?;
                    }
                    Err(err) => {
                        eprintln!("parse unit {} error :{}", name, err.error_format());
                        return Err(RuntimeError::new(RuntimeErrorType::InvalidFileFormat));
                    }
                },
            }
        }
        Ok(())
    }

    pub fn init_ctl_writer() -> File {
        let fd = unsafe { libc::open(ctl_path().as_ptr(), libc::O_WRONLY) };
        if fd < 0 {
            panic!("open ctl pipe error");
        }
        unsafe { File::from_raw_fd(fd) }
    }

    pub fn filter_units(patterns: Vec<Pattern>) -> Result<Vec<Arc<Mutex<dyn Unit>>>, RuntimeError> {
        let reader = ID_TO_UNIT_MAP.read().unwrap();

        // TODO: 这里可以优化
        let bindings = reader.values().collect::<Vec<_>>();
        let mut units = Vec::new();
        for unit in bindings {
            units.push(unit.clone());
        }
        for pat in patterns {
            match pat {
                Pattern::Type(t) => {
                    units = units
                        .into_iter()
                        .filter(|x| x.lock().unwrap().unit_type() == t)
                        .collect::<Vec<_>>()
                }
                Pattern::State(s) => {
                    units = units
                        .into_iter()
                        .filter(|x| *x.lock().unwrap().unit_base().state() == s)
                        .collect::<Vec<_>>()
                }
                Pattern::None => {}
                _ => {
                    return Err(RuntimeError::new(RuntimeErrorType::InvalidInput));
                }
            }
        }
        Ok(units)
    }
}
