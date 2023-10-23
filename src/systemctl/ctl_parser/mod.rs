use crate::error::parse_error::{ParseError, ParseErrorType};
use crate::parse::parse_util::UnitParseUtil;
use crate::unit::{UnitState, UnitType};
use hashbrown::HashMap;
use lazy_static::lazy_static;
use std::format;
use std::string::String;
use std::string::ToString;
use std::vec::Vec;

use super::listener::Command;

#[derive(Debug, Clone)]
pub enum CommandOperation {
    ListUnits,
    ListSockets,
    ListTimers,
    Start,
    Restart,
    Stop,
    Reload,
    TryRestart,
    ReloadOrRestart,
    ReloadOrTryRestart,
    Isolate,
    Kill,
    IsActive,
    IsFailed,
    Status,
    Show,
    Cat,
    SetProperty,
    Help,
    ResetFailed,
    ListDependencies,
    ListUnitFiles,
    Enable,
    Disable,
    Reenable,
    Preset,
    PresetAll,
    IsEnabled,
    Mask,
    UnMask,
    Link,
    AddWants,
    AddRequires,
    Edit,
    GetDefault,
    SetDefault,
    ListMachines,
    ListJobs,
    Cancel,
    Snapshot,
    Delete,
    ShowEnvironment,
    SetEnvironment,
    UnsetEnvironment,
    ImportEnvironment,
    DeamonReload,
    DeamonReexec,
    IsSystemRunning,
    Default,
    Rescue,
    Emergency,
    Halt,
    Poweroff,
    Reboot,
    Kexec,
    Exit,
    SwitchRoot,
    Suspend,
    Hibernate,
    HybridSleep,
    UnSupported,
    None,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Help,
    Version,
    System,
    Host(String),
    Machine(String),
    Type(UnitType),
    State(UnitState),
    Property(String),
    All,
    Full,
    Recursive,
    Reverse,
    JobMode(String),
    ShowTypes,
    IgnoreInhibitors,
    KillWho(String),
    Signal(String),
    Now,
    Quiet,
    NoBlock,
    NoWall,
    NoReload,
    NoLegend,
    NoPaper,
    NoAskPassword,
    Global,
    Runtime,
    Force,
    PresetMode,
    Root(String),
    Lines(i32),
    Output(String),
    Plain,
    None,
}

impl ToString for Pattern {
    fn to_string(&self) -> String {
        match self {
            Pattern::Help => "help".to_string(),
            Pattern::Version => "version".to_string(),
            Pattern::System => "system".to_string(),
            Pattern::Host(s) => format!("{}={}", "host".to_string(), s),
            Pattern::Machine(s) => format!("{}={}", "machine".to_string(), s),
            Pattern::Type(t) => format!("{}={:?}", "type".to_string(), t),
            Pattern::State(s) => format!("{}={:?}", "state".to_string(), s),
            Pattern::Property(s) => format!("{}={}", "property".to_string(), s),
            Pattern::All => "all".to_string(),
            Pattern::Full => "full".to_string(),
            Pattern::Recursive => "recursive".to_string(),
            Pattern::Reverse => "reverse".to_string(),
            Pattern::JobMode(s) => format!("{}={}", "job-mode".to_string(), s),
            Pattern::ShowTypes => "show-types".to_string(),
            Pattern::IgnoreInhibitors => "ignore-inhibitors".to_string(),
            Pattern::KillWho(s) => format!("{}={}", "kill-who".to_string(), s),
            Pattern::Signal(s) => format!("{}={}", "signal".to_string(), s),
            Pattern::Now => "now".to_string(),
            Pattern::Quiet => "quiet".to_string(),
            Pattern::NoBlock => "no-block".to_string(),
            Pattern::NoWall => "no-wall".to_string(),
            Pattern::NoReload => "no-reload".to_string(),
            Pattern::NoLegend => "no-legend".to_string(),
            Pattern::NoPaper => "no-paper".to_string(),
            Pattern::NoAskPassword => "no-ask-password".to_string(),
            Pattern::Global => "global".to_string(),
            Pattern::Runtime => "runtime".to_string(),
            Pattern::Force => "force".to_string(),
            Pattern::PresetMode => "preset-mode".to_string(),
            Pattern::Root(s) => format!("{}={}", "root".to_string(), s),
            Pattern::Lines(i) => format!("{}={}", "lines".to_string(), i),
            Pattern::Output(s) => format!("{}={}", "output".to_string(), s),
            Pattern::Plain => "plain".to_string(),
            Pattern::None => "none".to_string(),
        }
    }
}

lazy_static! {
    pub static ref CTL_COMMAND: HashMap<&'static str, CommandOperation> = {
        let mut map = HashMap::new();
        map.insert("list-units", CommandOperation::ListUnits);
        map.insert("list-sockets", CommandOperation::UnSupported);
        map.insert("list-timers", CommandOperation::UnSupported);
        map.insert("start", CommandOperation::Start);
        map.insert("stop", CommandOperation::Stop);
        map.insert("reload", CommandOperation::UnSupported);
        map.insert("restart", CommandOperation::Restart);
        map.insert("try-restart", CommandOperation::TryRestart);
        map.insert("reload-or-restart", CommandOperation::ReloadOrRestart);
        map.insert(
            "reload-or-try-restart",
            CommandOperation::ReloadOrTryRestart,
        );
        map.insert("isolate", CommandOperation::Isolate);
        map.insert("kill", CommandOperation::Kill);
        map.insert("is-active", CommandOperation::IsActive);
        map.insert("is-failed", CommandOperation::IsFailed);
        map.insert("status", CommandOperation::Status);
        map.insert("show", CommandOperation::Show);
        map.insert("cat", CommandOperation::Cat);
        map.insert("set-property", CommandOperation::SetProperty);
        map.insert("help", CommandOperation::Help);
        map.insert("reset-failed", CommandOperation::ResetFailed);
        map.insert("list-dependencies", CommandOperation::ListDependencies);
        map.insert("list-unit-files", CommandOperation::ListUnitFiles);
        map.insert("enable", CommandOperation::Enable);
        map.insert("disable", CommandOperation::Disable);
        map.insert("reenable", CommandOperation::Reenable);
        map.insert("preset", CommandOperation::Preset);
        map.insert("preset-all", CommandOperation::PresetAll);
        map.insert("is-enabled", CommandOperation::IsEnabled);
        map.insert("mask", CommandOperation::Mask);
        map.insert("unmask", CommandOperation::UnMask);
        map.insert("link", CommandOperation::Link);
        map.insert("add-wants", CommandOperation::AddWants);
        map.insert("add-requires", CommandOperation::AddRequires);
        map.insert("edit", CommandOperation::Edit);
        map.insert("get-default", CommandOperation::GetDefault);
        map.insert("set-default", CommandOperation::SetDefault);
        map.insert("list-machines", CommandOperation::ListMachines);
        map.insert("list-jobs", CommandOperation::ListJobs);
        map.insert("cancel", CommandOperation::Cancel);
        map.insert("snapshot", CommandOperation::Snapshot);
        map.insert("delete", CommandOperation::Delete);
        map.insert("show-environment", CommandOperation::ShowEnvironment);
        map.insert("set-environment", CommandOperation::SetEnvironment);
        map.insert("unset-environment", CommandOperation::UnsetEnvironment);
        map.insert("import-environment", CommandOperation::ImportEnvironment);
        map.insert("daemon-reload", CommandOperation::DeamonReload);
        map.insert("daemon-reexec", CommandOperation::DeamonReexec);
        map.insert("is-system-running", CommandOperation::IsSystemRunning);
        map.insert("default", CommandOperation::Default);
        map.insert("rescue", CommandOperation::Rescue);
        map.insert("emergency", CommandOperation::Emergency);
        map.insert("halt", CommandOperation::Halt);
        map.insert("poweroff", CommandOperation::Poweroff);
        map.insert("reboot", CommandOperation::Reboot);
        map.insert("kexec", CommandOperation::Kexec);
        map.insert("exit", CommandOperation::Exit);
        map.insert("switch-root", CommandOperation::SwitchRoot);
        map.insert("suspend", CommandOperation::Suspend);
        map.insert("hibernate", CommandOperation::Hibernate);
        map.insert("hybrid-sleep", CommandOperation::HybridSleep);
        map
    };
    pub static ref CTL_PATTERN: HashMap<&'static str, Pattern> = {
        let mut map = HashMap::new();
        map.insert("help", Pattern::Help);
        map.insert("version", Pattern::Version);
        map.insert("system", Pattern::System);
        map.insert("host", Pattern::Host(String::new()));
        map.insert("machine", Pattern::Machine(String::new()));
        map.insert("type", Pattern::Type(UnitType::Unknown));
        map.insert("state", Pattern::State(UnitState::Active));
        map.insert("property", Pattern::Property(String::new()));
        map.insert("all", Pattern::All);
        map.insert("full", Pattern::Full);
        map.insert("recursive", Pattern::Recursive);
        map.insert("reverse", Pattern::Reverse);
        map.insert("job-mode", Pattern::JobMode(String::new()));
        map.insert("show-types", Pattern::ShowTypes);
        map.insert("ignore-inhibitors", Pattern::IgnoreInhibitors);
        map.insert("kill-who", Pattern::KillWho(String::new()));
        map.insert("signal", Pattern::Signal(String::new()));
        map.insert("now", Pattern::Now);
        map.insert("quiet", Pattern::Quiet);
        map.insert("no-block", Pattern::NoBlock);
        map.insert("no-wall", Pattern::NoWall);
        map.insert("no-reload", Pattern::NoReload);
        map.insert("no-legend", Pattern::NoLegend);
        map.insert("no-paper", Pattern::NoPaper);
        map.insert("no-ask-password", Pattern::NoAskPassword);
        map.insert("global", Pattern::Global);
        map.insert("runtime", Pattern::Runtime);
        map.insert("force", Pattern::Force);
        map.insert("preset-mode", Pattern::PresetMode);
        map.insert("root", Pattern::Root(String::new()));
        map.insert("lines", Pattern::Lines(-1));
        map.insert("output", Pattern::Output(String::new()));
        map.insert("plain", Pattern::Plain);
        map
    };
}

pub struct CtlParser;

impl CtlParser {
    pub fn parse_ctl(s: &str) -> Result<Command, ParseError> {
        let mut words = s.split_whitespace().collect::<Vec<&str>>();
        let mut ctl = Command::default();
        if let Some(op) = CTL_COMMAND.get(words.remove(0)) {
            ctl.operation = op.clone();
        }

        let mut opt: Option<Vec<String>>;
        match ctl.operation {
            CommandOperation::Start
            | CommandOperation::Restart
            | CommandOperation::Stop
            | CommandOperation::TryRestart
            | CommandOperation::Reload
            | CommandOperation::AddRequires
            | CommandOperation::AddWants
            | CommandOperation::Kill
            | CommandOperation::ListDependencies
            | CommandOperation::Enable
            | CommandOperation::Disable
            | CommandOperation::Reenable
            | CommandOperation::Preset
            | CommandOperation::IsEnabled
            | CommandOperation::Mask
            | CommandOperation::UnMask
            | CommandOperation::Link
            | CommandOperation::Edit
            | CommandOperation::SetDefault
            | CommandOperation::SetEnvironment
            | CommandOperation::UnsetEnvironment
            | CommandOperation::ImportEnvironment => {
                opt = Some(Vec::new());
            }
            _ => {
                opt = None;
            }
        }

        for word in words {
            if word.starts_with("--") {
                ctl.patterns.push(Self::parse_pattern(&word[2..])?);
            } else {
                if let Some(ref mut v) = opt {
                    v.push(word.to_string());
                } else {
                    return Err(ParseError::new(ParseErrorType::EINVAL, s.to_string(), 0));
                }
            }
        }

        ctl.args = opt;

        Ok(ctl)
    }

    fn parse_pattern(s: &str) -> Result<Pattern, ParseError> {
        let pattern: Pattern;
        if s.contains("=") {
            let words = s.split("=").collect::<Vec<&str>>();
            if words.len() > 2 {
                return Err(ParseError::new(ParseErrorType::EINVAL, s.to_string(), 0));
            }

            let option = CTL_PATTERN.get(words[0]);
            if let Some(p) = option {
                pattern = match *p {
                    Pattern::Host(_) => Pattern::Host(words[1].to_string()),
                    Pattern::Machine(_) => Pattern::Machine(words[1].to_string()),
                    Pattern::Type(_) => Pattern::Type(UnitParseUtil::parse_type(words[1])),
                    Pattern::State(_) => {
                        todo!()
                    }
                    Pattern::JobMode(_) => Pattern::JobMode(words[1].to_string()),
                    Pattern::KillWho(_) => Pattern::KillWho(words[1].to_string()),
                    Pattern::Lines(_) => match words[1].parse::<i32>() {
                        Ok(val) => Pattern::Lines(val),
                        Err(_) => {
                            return Err(ParseError::new(ParseErrorType::EINVAL, s.to_string(), 0));
                        }
                    },
                    Pattern::Output(_) => Pattern::Output(words[1].to_string()),
                    Pattern::Property(_) => Pattern::Property(words[1].to_string()),
                    Pattern::Root(_) => Pattern::Root(words[1].to_string()),
                    Pattern::Signal(_) => Pattern::Signal(words[1].to_string()),
                    _ => {
                        return Err(ParseError::new(ParseErrorType::EINVAL, s.to_string(), 0));
                    }
                };
            } else {
                return Err(ParseError::new(ParseErrorType::EINVAL, s.to_string(), 0));
            }
        } else {
            pattern = match CTL_PATTERN.get(s) {
                Some(val) => val.clone(),
                None => {
                    return Err(ParseError::new(ParseErrorType::EINVAL, s.to_string(), 0));
                }
            };
        }
        return Ok(pattern);
    }
}
