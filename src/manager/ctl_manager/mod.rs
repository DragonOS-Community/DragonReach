use lazy_static::lazy_static;
use std::fs::File;
use std::os::fd::FromRawFd;
use std::sync::{Arc, Mutex};

use crate::error::runtime_error::RuntimeError;
use crate::error::runtime_error::RuntimeErrorType;
use crate::error::ErrorFormat;
use crate::parse::parse_util::UnitParseUtil;
use crate::systemctl::ctl_parser::{CommandOperation, Pattern};
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
            CommandOperation::ListUnits => Self::list_unit(cmd.patterns),
            CommandOperation::Start => Self::start(cmd.args.unwrap()),
            CommandOperation::Restart => Self::restart(cmd.args.unwrap(), false),
            CommandOperation::Stop => Self::stop(cmd.args.unwrap()),
            CommandOperation::Reboot => Ok(Self::reboot()),
            CommandOperation::ListSockets => todo!(),
            CommandOperation::ListTimers => todo!(),
            CommandOperation::Reload => todo!(),
            CommandOperation::TryRestart => Self::restart(cmd.args.unwrap(), true),
            CommandOperation::ReloadOrRestart => todo!(),
            CommandOperation::ReloadOrTryRestart => todo!(),
            CommandOperation::Isolate => todo!(),
            CommandOperation::Kill => todo!(),
            CommandOperation::IsActive => {
                let mut patterns = cmd.patterns.clone();
                patterns.push(Pattern::State(UnitState::Active));
                Self::list_unit(patterns)
            }
            CommandOperation::IsFailed => {
                let mut patterns = cmd.patterns.clone();
                patterns.push(Pattern::State(UnitState::Failed));
                Self::list_unit(patterns)
            }
            CommandOperation::Status => todo!(),
            CommandOperation::Show => todo!(),
            CommandOperation::Cat => todo!(),
            CommandOperation::SetProperty => todo!(),
            CommandOperation::Help => todo!(),
            CommandOperation::ResetFailed => todo!(),
            CommandOperation::ListDependencies => todo!(),
            CommandOperation::ListUnitFiles => todo!(),
            CommandOperation::Enable => todo!(),
            CommandOperation::Disable => todo!(),
            CommandOperation::Reenable => todo!(),
            CommandOperation::Preset => todo!(),
            CommandOperation::PresetAll => todo!(),
            CommandOperation::IsEnabled => todo!(),
            CommandOperation::Mask => todo!(),
            CommandOperation::UnMask => todo!(),
            CommandOperation::Link => todo!(),
            CommandOperation::AddWants => todo!(),
            CommandOperation::AddRequires => todo!(),
            CommandOperation::Edit => todo!(),
            CommandOperation::GetDefault => todo!(),
            CommandOperation::SetDefault => todo!(),
            CommandOperation::ListMachines => todo!(),
            CommandOperation::ListJobs => todo!(),
            CommandOperation::Cancel => todo!(),
            CommandOperation::Snapshot => todo!(),
            CommandOperation::Delete => todo!(),
            CommandOperation::ShowEnvironment => todo!(),
            CommandOperation::SetEnvironment => todo!(),
            CommandOperation::UnsetEnvironment => todo!(),
            CommandOperation::ImportEnvironment => todo!(),
            CommandOperation::DeamonReload => todo!(),
            CommandOperation::DeamonReexec => todo!(),
            CommandOperation::IsSystemRunning => todo!(),
            CommandOperation::Default => todo!(),
            CommandOperation::Rescue => todo!(),
            CommandOperation::Emergency => todo!(),
            CommandOperation::Halt => todo!(),
            CommandOperation::Poweroff => todo!(),
            CommandOperation::Kexec => todo!(),
            CommandOperation::Exit => todo!(),
            CommandOperation::SwitchRoot => todo!(),
            CommandOperation::Suspend => todo!(),
            CommandOperation::Hibernate => todo!(),
            CommandOperation::HybridSleep => todo!(),
            CommandOperation::UnSupported => todo!(),
            CommandOperation::None => {
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
        unsafe { libc::syscall(libc::SYS_reboot, 0, 0, 0, 0, 0, 0) };
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
