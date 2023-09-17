use crate::{
    contants::{AF_INET, AF_INET6, IPV4_MIN_MTU, IPV6_MIN_MTU, PRIO_MAX, PRIO_MIN},
    error::ParseError,
    task::cmdtask::CmdTask,
    unit::{Unit, Url},
    FileDescriptor,
};
//use drstd::std;
use std::{format, fs, path::Path, rc::Rc, string::String, string::ToString, vec, vec::Vec};

use super::{parse_service::ServiceParser, BASE_IEC, BASE_SI, SEC_UNIT_TABLE};

#[derive(PartialEq)]
pub enum SizeBase {
    IEC,
    Si,
}

pub struct UnitParseUtil;

impl UnitParseUtil {
    /// @brief 解析布尔值
    ///
    /// 将传入的字符串解析为布尔值
    /// "yes","y","1","true","t","on"均可表示true
    /// "no","n","0","false","f","off"均可表示false
    ///
    /// @param s 需解析的字符串
    ///
    /// @return 解析成功则返回Ok(解析后的值)，否则返回Err
    pub fn parse_boolean(s: &str) -> Result<bool, ParseError> {
        let t_table: Vec<&str> = vec!["yes", "y", "1", "true", "t", "on"];
        let f_table: Vec<&str> = vec!["no", "n", "0", "false", "f", "off"];

        if t_table.contains(&s) {
            return Ok(true);
        } else if f_table.contains(&s) {
            return Ok(false);
        }

        return Err(ParseError::EINVAL);
    }

    /// @brief 解析pid
    ///
    /// 将传入的字符串解析为pid
    ///
    /// @param s 需解析的字符串
    ///
    /// @return 解析成功则返回Ok(解析后的值)，否则返回Err
    pub fn parse_pid(s: &str) -> Result<i32, ParseError> {
        let s = s.trim();
        //先使用u64变换
        let pid_ul = match s.parse::<u64>() {
            Ok(val) => val,
            Err(_) => {
                return Err(ParseError::EINVAL);
            }
        };
        let pid: i32 = pid_ul as i32;

        if (pid as u64) != pid_ul {
            //如果在从pid_t转换为u64之后与之前不等，则说明发生了截断，返回错误
            return Err(ParseError::ERANGE);
        }

        if pid < 0 {
            //pid小于0不合法
            return Err(ParseError::EINVAL);
        }

        return Ok(pid);
    }

    /// @brief 解析pid
    ///
    /// 将传入的字符串解析为mode_t
    ///
    /// @param s 需解析的字符串
    ///
    /// @return 解析成功则返回Ok(解析后的值)，否则返回Err
    pub fn parse_mode(s: &str) -> Result<u32, ParseError> {
        let s = s.trim();
        let m = match u32::from_str_radix(s, 8) {
            Ok(val) => val,
            Err(_) => {
                return Err(ParseError::EINVAL);
            }
        };

        //如果模式大于权限的最大值则为非法权限，返回错误
        if m > 0o7777 {
            return Err(ParseError::ERANGE);
        }

        return Ok(m);
    }

    /// @brief 解析网络接口索引
    ///
    /// 将传入的字符串解析为网络接口索引具体值
    ///
    /// @param s 需解析的字符串
    ///
    /// @return 解析成功则返回Ok(解析后的值)，否则返回Err
    pub fn parse_ifindex(s: &str) -> Result<i32, ParseError> {
        let s = s.trim();
        let ret: i32 = match s.parse::<i32>() {
            Ok(val) => val,
            Err(_) => {
                return Err(ParseError::EINVAL);
            }
        };

        if ret <= 0 {
            return Err(ParseError::EINVAL);
        }

        return Ok(ret);
    }

    /// @brief 解析最大传输单元(MTU)
    ///
    /// 将传入的字符串解析为具体值
    ///
    /// @param s 需解析的字符串
    ///
    /// @param family 网络地址族
    ///
    /// @return 解析成功则返回Ok(解析后的值)，否则返回Err
    pub fn parse_mtu(s: &str, family: i32) -> Result<u32, ParseError> {
        let s = s.trim();
        let mtu = match s.parse::<u64>() {
            Ok(val) => val,
            Err(_) => {
                //针对非法字符出错时
                return Err(ParseError::EINVAL);
            }
        };

        //针对数据溢出时的报错
        if mtu > u32::MAX as u64 {
            return Err(ParseError::ERANGE);
        }

        let mtu: u32 = mtu as u32;

        let mut min_mtu: u32 = 0;
        //判断mtu是否合法
        if family == AF_INET6 {
            min_mtu = IPV6_MIN_MTU;
        } else if family == AF_INET {
            min_mtu = IPV4_MIN_MTU;
        } else {
            return Err(ParseError::EINVAL);
        }

        return Ok(mtu);
    }

    /// @brief 解析Size
    ///
    /// 将传入的字符串解析为具体的字节数
    /// 可支持IEC二进制后缀，也可支持SI十进制后缀
    ///
    /// @param s 需解析的字符串
    ///
    /// @param base 设置为IEC二进制后缀或者SI十进制后缀
    ///
    /// @return 解析成功则返回Ok(解析后的值)，否则返回Err
    pub fn parse_size(s: &str, base: SizeBase) -> Result<u64, ParseError> {
        let s = s.trim();
        //将s分解为数字和后缀部分
        let (number_str, suffix) = match s.find(|c: char| !c.is_digit(10) && c != '.') {
            Some(mid) => s.split_at(mid),
            None => (s, ""),
        };

        //获得数字部分的整数和小数部分
        let (integer, fraction) = match number_str.find(".") {
            Some(mid) => {
                let (integer, fraction) = number_str.split_at(mid);
                let integer = integer.parse::<u64>().unwrap();
                let fraction = match fraction[1..].parse::<u64>() {
                    Ok(val) => val,
                    Err(_) => {
                        return Err(ParseError::EINVAL);
                    }
                };
                (integer, fraction)
            }
            None => (number_str.parse::<u64>().unwrap(), 0),
        };

        //从表中查找到后缀所对应的字节倍数
        let mut factor: u64 = 0;
        if base == SizeBase::IEC {
            factor = match BASE_IEC.get(suffix) {
                Some(val) => *val,
                None => {
                    return Err(ParseError::EINVAL);
                }
            }
        } else if base == SizeBase::Si {
            factor = match BASE_SI.get(suffix) {
                Some(val) => *val,
                None => {
                    return Err(ParseError::EINVAL);
                }
            }
        }

        Ok(integer * factor + (fraction * factor) / (10u64.pow(fraction.to_string().len() as u32)))
    }

    /// @brief 解析扇区大小
    ///
    /// 将传入的字符串解析为具体的扇区大小
    /// 若扇区大小小于512或者大于4096,将会返回错误，若扇区大小不为2的幂，返回错误。
    ///
    /// @param s 需解析的字符串
    ///
    /// @return 解析成功则返回Ok(解析后的值)，否则返回Err
    pub fn parse_sector_size(s: &str) -> Result<u64, ParseError> {
        let s = s.trim();
        let size: u64 = match s.parse::<u64>() {
            Ok(val) => val,
            Err(_) => {
                return Err(ParseError::EINVAL);
            }
        };

        if size < 512 || size > 4096 {
            return Err(ParseError::ERANGE);
        }

        //判断是否为2的幂，如果不是则报错
        if (size & (size - 1)) != 0 {
            return Err(ParseError::EINVAL);
        }

        return Ok(size);
    }

    /// @brief 解析范围
    ///
    /// 将传入的字符串解析为具体的范围
    ///
    /// @param s 需解析的字符串
    ///
    /// @return 解析成功则返回Ok(解析后的值)，否则返回Err
    pub fn parse_range(s: &str) -> Result<(u32, u32), ParseError> {
        let mid = match s.find('-') {
            Some(val) => val,
            None => {
                //如果字符串中没有'-'符号，则表示一个值，所以范围两端都为该值
                let s = s.trim();
                let ret = match s.parse::<u32>() {
                    Ok(val) => val,
                    Err(_) => {
                        return Err(ParseError::EINVAL);
                    }
                };
                return Ok((ret, ret));
            }
        };

        //若字符串中存在'-'，则分别解析为u32，解析失败则报错
        let (l, r) = s.split_at(mid);

        let l = l.trim();
        let l = match l.parse::<u32>() {
            Ok(val) => val,
            Err(_) => {
                return Err(ParseError::EINVAL);
            }
        };
        let r = r.trim();
        let r = match r.parse::<u32>() {
            Ok(val) => val,
            Err(_) => {
                return Err(ParseError::EINVAL);
            }
        };

        return Ok((l, r));
    }

    /// @brief 解析文件描述符
    ///
    /// 将传入的字符串解析为文件描述符fd
    ///
    /// @param s 需解析的字符串
    ///
    /// @return 解析成功则返回Ok(解析后的值)，否则返回Err
    pub fn parse_fd(s: &str) -> Result<FileDescriptor, ParseError> {
        let s = s.trim();
        let fd = match s.parse::<i32>() {
            Ok(val) => val,
            Err(_) => {
                return Err(ParseError::EINVAL);
            }
        };

        if fd < 0 {
            return Err(ParseError::EBADF);
        }

        return Ok(FileDescriptor(fd as usize));
    }

    /// @brief 解析nice
    ///
    /// 将传入的字符串解析为nice
    ///
    /// @param s 需解析的字符串
    ///
    /// @return 解析成功则返回Ok(解析后的值)，否则返回Err
    pub fn parse_nice(s: &str) -> Result<i8, ParseError> {
        let s = s.trim();
        let nice = match s.parse::<i8>() {
            Ok(val) => val,
            Err(_) => {
                return Err(ParseError::EINVAL);
            }
        };

        if nice > PRIO_MAX || nice < PRIO_MIN {
            return Err(ParseError::ERANGE);
        }

        return Ok(nice);
    }

    /// @brief 解析端口号
    ///
    /// 将传入的字符串解析为端口号
    ///
    /// @param s 需解析的字符串
    ///
    /// @return 解析成功则返回Ok(解析后的值)，否则返回Err
    pub fn parse_ip_port(s: &str) -> Result<u16, ParseError> {
        let s = s.trim();
        let port = match s.parse::<u16>() {
            Ok(val) => val,
            Err(_) => {
                return Err(ParseError::EINVAL);
            }
        };

        if port == 0 {
            return Err(ParseError::EINVAL);
        }

        return Ok(port);
    }

    /// @brief 解析端口范围
    ///
    /// 将传入的字符串解析为端口范围
    ///
    /// @param s 需解析的字符串
    ///
    /// @return 解析成功则返回Ok((u16,u16))，否则返回Err
    pub fn parse_ip_port_range(s: &str) -> Result<(u16, u16), ParseError> {
        let (l, h) = Self::parse_range(s)?;

        let l = l as u16;
        let h = h as u16;
        if l <= 0 || l >= 65535 || h <= 0 || h >= 65535 {
            return Err(ParseError::EINVAL);
        }

        return Ok((l, h));
    }

    /// @brief 解析OOM（Out-of-Memory）分数调整值
    ///
    /// 将传入的字符串解析为OOM（Out-of-Memory）分数调整值
    ///
    /// @param s 需解析的字符串
    ///
    /// @return 解析成功则返回Ok(u32)，否则返回Err
    pub fn parse_ip_prefix_length(s: &str) -> Result<u32, ParseError> {
        let len = match s.parse::<u32>() {
            Ok(val) => val,
            Err(_) => {
                return Err(ParseError::EINVAL);
            }
        };

        if len > 128 {
            return Err(ParseError::ERANGE);
        }

        return Ok(len);
    }

    /// @brief 目前为简单的分割字符串，并未提供严谨的Url解析
    ///
    /// 将传入的字符串解析为Url结构体的Vec，若Url非法则返回错误
    ///
    /// @param s 需解析的字符串
    ///
    /// @return 解析成功则返回Ok(Url)，否则返回Err
    pub fn parse_url(s: &str) -> Result<Vec<Url>, ParseError> {
        let mut url = Url::default();
        let url_strs = s.split_whitespace().collect::<Vec<&str>>();
        let mut urls = Vec::new();
        for s in url_strs {
            urls.push(Url {
                url_string: String::from(s),
            })
        }
        return Ok(urls);
    }

    /// @brief 将对应的str解析为对应Unit
    ///
    /// 将传入的字符串解析为Unit，解析失败返回错误
    ///
    /// @param path 需解析的文件
    ///
    /// @return 解析成功则返回Ok(Rc<dyn Unit>)，否则返回Err
    pub fn parse_unit<T: Unit>(path: &str) -> Result<Rc<T>, ParseError> {
        return T::from_path(path);
    }

    /// @brief 将对应的str解析为Rc<dyn Unit>
    ///
    /// 将传入的字符串解析为Rc<dyn Unit>，解析失败返回错误
    ///
    /// @param path 需解析的文件
    ///
    /// @return 解析成功则返回Ok(Rc<dyn Unit>)，否则返回Err
    pub fn parse_unit_no_type(path: &str) -> Result<Rc<dyn Unit>, ParseError> {
        let idx = match path.rfind('.') {
            Some(val) => val,
            None => {
                return Err(ParseError::EINVAL);
            }
        };

        if idx == path.len() - 1 {
            //处理非法文件xxxx. 类型
            return Err(ParseError::EINVAL);
        }

        let suffix = &path[idx + 1..];

        //通过文件后缀分发给不同类型的Unit解析器解析
        let unit = match suffix {
            "service" => ServiceParser::parse(path)?,
            _ => {
                return Err(ParseError::EINVAL);
            }
        };

        return Ok(unit);
    }

    /// @brief 将对应的str解析为对应CmdTask
    ///
    /// 将传入的字符串解析为CmdTask组，解析失败返回错误
    ///
    /// @param path 需解析的文件
    ///
    /// @return 解析成功则返回Ok(Vec<CmdTask>>)，否则返回Err
    pub fn parse_cmd_task(s: &str) -> Result<Vec<CmdTask>, ParseError> {
        //分拆成单词Vec
        let cmds = s.split_whitespace().collect::<Vec<&str>>();
        let mut tasks = Vec::new();
        let mut i = 0;
        while i < cmds.len() {
            let mut cmd_task = CmdTask {
                path: String::new(),
                cmd: String::new(),
                ignore: false,
            };
            //匹配到这里时，这个单词肯定是路径，若路径以-开头则设置ignore
            cmd_task.ignore = cmds[i].starts_with('-');

            //获取到一个CmdTask的路径部分
            let mut path = "";
            if cmd_task.ignore {
                path = &cmds[i][1..];
            } else {
                path = &cmds[i];
            }

            //得到的非绝对路径则不符合语法要求，报错
            if !UnitParseUtil::is_absolute_path(path) {
                return Err(ParseError::EINVAL);
            }

            cmd_task.path = String::from(path);

            //i += 1,继续匹配下一个单词
            i += 1;
            let mut cmd_str = String::new();
            while i < cmds.len() && !UnitParseUtil::is_absolute_path(cmds[i]) {
                //命令可能会有多个单词，将多个命令整理成一个
                let cmd = cmds[i];
                cmd_str = format!("{} {}", cmd_str, cmd);
                i += 1;
            }
            cmd_task.cmd = cmd_str;
            tasks.push(cmd_task);
            //经过while到这里之后，cmds[i]对应的单词一点是路径，i不需要加一
        }
        return Ok(tasks);
    }

    /// @brief 判断是否为绝对路径
    ///
    /// @param path 路径
    ///
    /// @return 解析成功则返回true，否则返回false
    pub fn is_absolute_path(path: &str) -> bool {
        let path = Path::new(path);
        path.is_absolute()
    }

    /// @brief 将对应的str解析为us(微秒)
    ///
    /// 将传入的字符串解析为秒数，解析失败返回错误
    ///
    /// @param path 需解析的文件
    ///
    /// @return 解析成功则返回Ok(u64)，否则返回Err
    pub fn parse_sec(s: &str) -> Result<u64, ParseError> {
        //下列参数分别记录整数部分，小数部分以及单位
        let mut integer: u64 = 0;
        let mut frac: u64 = 0;
        let mut unit: &str = "";

        match s.find('.') {
            Some(idx) => {
                //解析整数部分
                integer = match s[..idx].parse::<u64>() {
                    Ok(val) => val,
                    Err(_) => return Err(ParseError::EINVAL),
                };
                //获得小数+单位的字符串
                let frac_and_unit = &s[(idx + 1)..];
                match frac_and_unit.find(|c: char| !c.is_digit(10)) {
                    Some(val) => {
                        //匹配小数部分
                        frac = match frac_and_unit[..val].parse::<u64>() {
                            Ok(val) => val,
                            Err(_) => return Err(ParseError::EINVAL),
                        };
                        //单位部分
                        unit = &frac_and_unit[val..];
                    }
                    None => {
                        //没有单位的情况，直接匹配小数
                        frac = match frac_and_unit.parse::<u64>() {
                            Ok(val) => val,
                            Err(_) => return Err(ParseError::EINVAL),
                        };
                        unit = "";
                    }
                };
            }
            None => {
                //没有小数点则直接匹配整数部分和单位部分
                match s.find(|c: char| !c.is_digit(10)) {
                    Some(idx) => {
                        integer = match s[..idx].parse::<u64>() {
                            Ok(val) => val,
                            Err(_) => return Err(ParseError::EINVAL),
                        };
                        unit = &s[idx..];
                    }
                    None => {
                        integer = match s.parse::<u64>() {
                            Ok(val) => val,
                            Err(_) => return Err(ParseError::EINVAL),
                        };
                        unit = "";
                    }
                };
            }
        };

        //从时间单位转换表中获取到单位转换为ns的倍数
        let factor = match SEC_UNIT_TABLE.get(unit) {
            Some(val) => val,
            None => {
                return Err(ParseError::EINVAL);
            }
        };

        //计算ns
        return Ok(integer * factor + (frac * factor) / (10u64.pow(frac.to_string().len() as u32)));
    }

    /// @brief 判断对应路径是否为目录
    ///
    /// @param path 路径
    ///
    /// @return true/false
    pub fn is_dir(path: &str) -> bool {
        if let Ok(metadata) = fs::metadata(path) {
            if metadata.is_dir() {
                return true;
            }
            return false;
        }
        return false;
    }
}
