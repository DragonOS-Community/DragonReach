
#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Clone)]
#[allow(dead_code, non_camel_case_types)]
pub enum SystemError {
    EPERM = 1,
    /// 没有指定的文件或目录 No such file or directory.
    ENOENT = 2,
    /// 没有这样的进程 No such process.
    ESRCH = 3,
    /// 被中断的函数 Interrupted function.
    EINTR = 4,
    /// I/O错误 I/O error.
    EIO = 5,
    /// 没有这样的设备或地址 No such device or address.
    ENXIO = 6,
    /// 参数列表过长，或者在输出buffer中缺少空间 或者参数比系统内建的最大值要大 Argument list too long.
    E2BIG = 7,
    /// 可执行文件格式错误 Executable file format error
    ENOEXEC = 8,
    /// 错误的文件描述符 Bad file descriptor.
    EBADF = 9,
    /// 没有子进程 No child processes.
    ECHILD = 10,
    /// 资源不可用，请重试。 Resource unavailable, try again.(may be the same value as [EWOULDBLOCK])
    ///
    /// 操作将被禁止 Operation would block.(may be the same value as [EAGAIN]).
    EAGAIN_OR_EWOULDBLOCK = 11,
    /// 没有足够的空间 Not enough space.
    ENOMEM = 12,
    /// 访问被拒绝 Permission denied
    EACCES = 13,
    /// 错误的地址 Bad address
    EFAULT = 14,
    /// 需要块设备 Block device required
    ENOTBLK = 15,
    /// 设备或资源忙 Device or resource busy.
    EBUSY = 16,
    /// 文件已存在 File exists.
    EEXIST = 17,
    /// 跨设备连接 Cross-device link.
    EXDEV = 18,
    /// 没有指定的设备 No such device.
    ENODEV = 19,
    /// 不是目录 Not a directory.
    ENOTDIR = 20,
    /// 是一个目录 Is a directory
    EISDIR = 21,
    /// 不可用的参数 Invalid argument.
    EINVAL = 22,
    /// 系统中打开的文件过多 Too many files open in system.
    ENFILE = 23,
    /// 文件描述符的值过大 File descriptor value too large.
    EMFILE = 24,
    /// 不正确的I/O控制操作 Inappropriate I/O control operation.
    ENOTTY = 25,
    /// 文本文件忙 Text file busy.
    ETXTBSY = 26,
    /// 文件太大 File too large.
    EFBIG = 27,
    /// 设备上没有空间 No space left on device.
    ENOSPC = 28,
    /// 错误的寻道.当前文件是pipe，不允许seek请求  Invalid seek.
    ESPIPE = 29,
    /// 只读的文件系统 Read-only file system.
    EROFS = 30,
    /// 链接数过多 Too many links.
    EMLINK = 31,
    /// 断开的管道 Broken pipe.
    EPIPE = 32,
    /// 数学参数超出作用域 Mathematics argument out of domain of function.
    EDOM = 33,
    /// 结果过大 Result too large.
    ERANGE = 34,
}