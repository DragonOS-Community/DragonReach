//use drstd as std;
use std::string::String;

#[derive(Debug)]
pub struct CmdTask {
    pub path: String,
    pub cmd: String,
    pub ignore: bool, //表示忽略这个命令的错误，即使它运行失败也不影响unit正常运作
}
