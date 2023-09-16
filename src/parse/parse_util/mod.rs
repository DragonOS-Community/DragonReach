use drstd::std as std;
use std::{
    num,
    vec,
    vec::Vec,
    string::ToString
};
use crate::{error::SystemError, types::{pid_t, mode_t}, contants::{AF_INET6, IPV6_MIN_MTU, AF_INET, IPV4_MIN_MTU, PRIO_MAX, PRIO_MIN}};

/// @brief 解析布尔值
/// 
/// 将传入的字符串解析为布尔值
/// "yes","y","1","true","t","on"均可表示true
/// "no","n","0","false","f","off"均可表示false
/// 
/// @param s 需解析的字符串
/// 
/// @return 解析成功则返回Ok(解析后的值)，否则返回Err
pub fn parse_boolean(s: &str) -> Result<bool,SystemError> {
    let t_table: Vec<&str> = vec!["yes","y","1","true","t","on"];
    let f_table: Vec<&str> = vec!["no","n","0","false","f","off"];

    if t_table.contains(&s) {
        return Ok(true);
    }else if f_table.contains(&s) {
        return Ok(false);
    }
    
    return Err(SystemError::EINVAL);
}

/// @brief 解析pid
/// 
/// 将传入的字符串解析为pid
/// 
/// @param s 需解析的字符串
/// 
/// @return 解析成功则返回Ok(解析后的值)，否则返回Err
pub fn parse_pid(s: &str) -> Result<pid_t,SystemError>{
    let s = s.trim();
    //先使用u64变换
    let pid_ul = match s.parse::<u64>() {
        Ok(val) => val,
        Err(_) => {
            return Err(SystemError::EINVAL);
        }
    };
    let pid: pid_t = pid_ul as pid_t;

    if (pid as u64) != pid_ul {
        //如果在从pid_t转换为u64之后与之前不等，则说明发生了截断，返回错误
        return Err(SystemError::ERANGE);
    }

    if pid < 0 {
        //pid小于0不合法
        return Err(SystemError::EINVAL);
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
pub fn parse_mode(s: &str) -> Result<mode_t,SystemError> {
    let s = s.trim();
    let m = match u32::from_str_radix(s, 8) {
        Ok(val) => val,
        Err(_) => {
            return Err(SystemError::EINVAL);
        }
    };

    //如果模式大于权限的最大值则为非法权限，返回错误
    if m > 0o7777 {
        return Err(SystemError::ERANGE);
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
pub fn parse_ifindex(s: &str) -> Result<i32,SystemError> {
    let s = s.trim();
    let ret: i32 = match s.parse::<i32>(){
        Ok(val) => val,
        Err(_) => {
            return Err(SystemError::EINVAL);
        }
    };

    if ret <= 0 {
        return Err(SystemError::EINVAL);
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
pub fn parse_mtu(s: &str,family: i32) -> Result<u32,SystemError> {
    let s = s.trim();
    let mtu = match s.parse::<u64>() {
        Ok(val) => val,
        Err(_) => {
            //针对非法字符出错时
            return Err(SystemError::EINVAL);
        }
    };
    
    //针对数据溢出时的报错
    if mtu > u32::MAX as u64 {
        return Err(SystemError::ERANGE);
    }

    let mtu: u32 = mtu as u32;

    let mut min_mtu: u32 = 0;
    //判断mtu是否合法
    if family == AF_INET6 {
        min_mtu = IPV6_MIN_MTU;
    }else if family == AF_INET {
        min_mtu = IPV4_MIN_MTU;
    }else {
        return Err(SystemError::EINVAL);
    }

    return Ok(mtu);
}

#[derive(PartialEq)]
pub enum SizeBase {
    IEC,
    Si
}

struct BaseMapper<'a> {
    suffix: &'a str,
    factor: u64
}

// IEC表
const BASE_IEC:[BaseMapper;8] = [
    BaseMapper{ suffix: "E", factor: 1024u64*1024u64*1024u64*1024u64*1024u64*1024u64 },
    BaseMapper{ suffix: "P", factor: 1024u64*1024u64*1024u64*1024u64*1024u64 },
    BaseMapper{ suffix: "T", factor: 1024u64*1024u64*1024u64*1024u64 },
    BaseMapper{ suffix: "G", factor: 1024u64*1024u64*1024u64 },
    BaseMapper{ suffix: "M", factor: 1024u64*1024u64 },
    BaseMapper{ suffix: "K", factor: 1024u64 },
    BaseMapper{ suffix: "B", factor: 1u64 },
    BaseMapper{ suffix: "",  factor: 1u64 },
];

// SI表
const BASE_SI:[BaseMapper;8] = [
    BaseMapper{ suffix: "E", factor: 1000u64*1000u64*1000u64*1000u64*1000u64*1000u64 },
    BaseMapper{ suffix: "P", factor: 1000u64*1000u64*1000u64*1000u64*1000u64 },
    BaseMapper{ suffix: "T", factor: 1000u64*1000u64*1000u64*1000u64 },
    BaseMapper{ suffix: "G", factor: 1000u64*1000u64*1000u64 },
    BaseMapper{ suffix: "M", factor: 1000u64*1000u64 },
    BaseMapper{ suffix: "K", factor: 1000u64 },
    BaseMapper{ suffix: "B", factor: 1u64 },
    BaseMapper{ suffix: "",  factor: 1u64 },
];

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
pub fn parse_size(s: &str, base: SizeBase) -> Result<u64, SystemError> {
    let s = s.trim();
    //将s分解为数字和后缀部分
    let (number_str,suffix) = match s.find(|c: char| !c.is_digit(10) && c != '.'){
        Some(mid) => s.split_at(mid),
        None => (s,"")
    };

    //获得数字部分的整数和小数部分
    let (integer, fraction) = match number_str.find(".") {
        Some(mid) => {
            let (integer, fraction) = number_str.split_at(mid);
            let integer = integer.parse::<u64>().unwrap();
            let fraction = match fraction[1..].parse::<u64>() {
                Ok(val) => val,
                Err(_) => {
                    return Err(SystemError::EINVAL);
                }
            };
            (integer,fraction)
        }
        None => (number_str.parse::<u64>().unwrap(),0)
    };

    //从表中查找到后缀所对应的字节倍数
    let mut factor: u64 = 0;
    if base == SizeBase::IEC {
        factor = match BASE_IEC.iter().find(|mapper|mapper.suffix == suffix) {
            Some(val) => val.factor,
            None => {
                return Err(SystemError::EINVAL);
            }
        } 
    } else if base == SizeBase::Si {
        factor = match BASE_SI.iter().find(|mapper|mapper.suffix == suffix) {
            Some(val) => val.factor,
            None => {
                return Err(SystemError::EINVAL);
            }
        } 
    }
    
    Ok(integer*factor + (fraction*factor)/(10u64.pow(fraction.to_string().len() as u32)))
}

/// @brief 解析扇区大小
/// 
/// 将传入的字符串解析为具体的扇区大小
/// 若扇区大小小于512或者大于4096,将会返回错误，若扇区大小不为2的幂，返回错误。
/// 
/// @param s 需解析的字符串
/// 
/// @return 解析成功则返回Ok(解析后的值)，否则返回Err
pub fn parse_sector_size(s: &str) -> Result<u64,SystemError> {
    let s = s.trim();
    let size: u64 = match s.parse::<u64>(){
        Ok(val) => val,
        Err(_) => {
            return Err(SystemError::EINVAL);
        }
    };

    if size < 512 || size > 4096 {
        return Err(SystemError::ERANGE);
    }

    //判断是否为2的幂，如果不是则报错
    if (size & (size - 1)) != 0 {
        return Err(SystemError::EINVAL);
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
pub fn parse_range(s: &str) -> Result<(u32,u32),SystemError> {
    let mid = match s.find('-') {
        Some(val) => val,
        None =>{
            //如果字符串中没有'-'符号，则表示一个值，所以范围两端都为该值
            let s = s.trim();
            let ret = match s.parse::<u32>() {
                Ok(val) => val,
                Err(_) => {
                    return Err(SystemError::EINVAL);
                }
            };
            return Ok((ret,ret));
        }
    };

    //若字符串中存在'-'，则分别解析为u32，解析失败则报错
    let (l,r) = s.split_at(mid);

    let l = l.trim();
    let l = match l.parse::<u32>(){
        Ok(val) => val,
        Err(_) => {
            return Err(SystemError::EINVAL);
        }
    };
    let r = r.trim();
    let r = match r.parse::<u32>(){
        Ok(val) => val,
        Err(_) => {
            return Err(SystemError::EINVAL);
        }
    };

    return Ok((l,r));
}

/// @brief 解析文件描述符
/// 
/// 将传入的字符串解析为文件描述符fd
/// 
/// @param s 需解析的字符串
/// 
/// @return 解析成功则返回Ok(解析后的值)，否则返回Err
pub fn parse_fd(s: &str) -> Result<i32,SystemError> {
    let s = s.trim();
    let fd = match s.parse::<i32>() {
        Ok(val) => val,
        Err(_) => {
            return Err(SystemError::EINVAL);
        }
    };

    if fd < 0 {
        return Err(SystemError::EBADF);
    }

    return Ok(fd);
}

/// @brief 解析nice
/// 
/// 将传入的字符串解析为nice
/// 
/// @param s 需解析的字符串
/// 
/// @return 解析成功则返回Ok(解析后的值)，否则返回Err
pub fn parse_nice(s: &str) -> Result<i8,SystemError> {
    let s = s.trim();
    let nice = match s.parse::<i8>() {
        Ok(val) => val,
        Err(_) => {
            return Err(SystemError::EINVAL);
        }
    };

    if nice > PRIO_MAX || nice < PRIO_MIN {
        return Err(SystemError::ERANGE);
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
pub fn parse_ip_port(s: &str) -> Result<u16,SystemError> {
    let s = s.trim();
    let port = match s.parse::<u16>() {
        Ok(val) => val,
        Err(_) => {
            return Err(SystemError::EINVAL);
        }
    };

    if port == 0 {
        return Err(SystemError::EINVAL);
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
pub fn parse_ip_port_range(s: &str) -> Result<(u16,u16),SystemError> {
    let (l,h) = parse_range(s)?;

    let l = l as u16;
    let h = h as u16;
    if l <=0 || l >= 65535 || h <= 0 || h >= 65535 {
        return Err(SystemError::EINVAL);
    }

    return Ok((l,h));
}

/// @brief 解析OOM（Out-of-Memory）分数调整值
/// 
/// 将传入的字符串解析为OOM（Out-of-Memory）分数调整值
/// 
/// @param s 需解析的字符串
/// 
/// @return 解析成功则返回Ok(u32)，否则返回Err
pub fn parse_ip_prefix_length(s: &str) -> Result<u32,SystemError> {
    let len = match s.parse::<u32>() {
        Ok(val) => val,
        Err(_) => {
            return Err(SystemError::EINVAL);
        }
    };

    if len > 128 {
        return Err(SystemError::ERANGE);
    }

    return Ok(len);
}