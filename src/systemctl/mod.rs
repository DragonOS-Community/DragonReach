use std::ffi::CString;

pub mod ctl_parser;
pub mod listener;

pub const DRAGON_REACH_CTL_PIPE: &'static str = "/etc/reach/ipc/ctl";
//pub const DRAGON_REACH_CTL_PIPE: &'static str = "/home/fz/myetc/reach/ipc/ctl";

pub fn ctl_path() -> CString {
    CString::new(DRAGON_REACH_CTL_PIPE).expect("Failed to create pipe CString")
}
