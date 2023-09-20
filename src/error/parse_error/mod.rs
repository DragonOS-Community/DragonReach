#[cfg(target_os = "dragonos")]
use drstd as std;
use std::format;
use std::string::String;
use std::string::ToString;

use super::ErrorFormat;
/// 解析错误，错误信息应该包括文件名以及行号
#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Clone)]
#[allow(dead_code, non_camel_case_types)]
pub enum ParseErrorType {
    /// 不合法参数
    EINVAL,
    /// 结果过大 Result too large.
    ERANGE,
    /// 重复定义
    EREDEF,
    /// 未预料到的空值
    EUnexpectedEmpty,
    /// 语法错误
    ESyntaxError,
    /// 非法文件描述符
    EBADF,
    /// 非法文件
    EFILE,
    /// 不是目录
    ENODIR,
    /// 循环依赖
    ECircularDependency,
}
/// 错误信息应该包括错误类型ParseErrorType,当前解析的文件名，当前解析的行号
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParseError(ParseErrorType,String,usize);

impl ParseError {
    pub fn new(error_type: ParseErrorType,file_name: String,line_number: usize) -> ParseError {
        ParseError(error_type,file_name,line_number)
    }

    pub fn set_file(&mut self,path: &str) {
        self.1 = path.to_string();
    }

    pub fn set_linenum(&mut self,linenum: usize) {
        self.2 = linenum;
    }
}

impl ErrorFormat for ParseError {
    fn error_format(&self) -> String {
        format!("Parse Error!,Error Type: {:?}, File: {}, Line: {}",self.0,self.1,self.2)
    }
}