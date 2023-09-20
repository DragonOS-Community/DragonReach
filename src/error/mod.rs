#[cfg(target_os = "dragonos")]
use drstd as std;

pub mod parse_error;
pub mod runtime_error;

use std::string::String;

pub trait ErrorFormat {
    fn error_format(&self) -> String;
}
