#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Clone)]
#[allow(dead_code, non_camel_case_types)]
pub enum ParseError {
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
}
