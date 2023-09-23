use super::ErrorFormat;
#[cfg(target_os = "dragonos")]
use drstd as std;
use std::format;
use std::string::String;

#[derive(Debug)]
pub enum RuntimeErrorType {
    //启动失败
    ExecFailed,

    // 文件相关错误
    FileNotFound,
    FileAccessDenied,
    InvalidFileFormat,

    // 循环依赖
    CircularDependency,

    // 转型错误
    DowncastError,

    // 网络相关错误
    ConnectionError,
    Timeout,

    // 数据库相关错误
    DatabaseError,
    QueryError,

    // 并发相关错误
    Deadlock,
    ThreadPanicked,

    // 配置相关错误
    ConfigError,
    InvalidParameter,

    // 其他通用错误
    OutOfMemory,
    InvalidInput,
    UnsupportedOperation,

    // 自定义错误
    Custom(String),
}

pub struct RuntimeError(RuntimeErrorType);

impl RuntimeError {
    pub fn new(error_type: RuntimeErrorType) -> Self {
        return RuntimeError(error_type);
    }
}

impl ErrorFormat for RuntimeError {
    fn error_format(&self) -> String {
        format!("Runtime Error!,Error Type: {:?}", self.0)
    }
}
