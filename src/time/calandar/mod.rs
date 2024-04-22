//use crate::error::parse_error::ParseError;

#[derive(Debug, Default, Clone)]
pub struct CalendarComponent {
    // start: usize,
    // stop: usize,
    // repeat: usize,
    // next: Arc<CalendarComponent>,//暂时不清楚为什么要链式设计
}
#[derive(Debug, Default, Clone)]
pub struct CalendarStandard {
    //@brief 星期几
    // weekdays_bits: Vec<WeekdayBits>,
    // year: usize,
    // month:  usize,
    // day: usize,
    // hour: usize,
    // minute: usize,
    // microsecond: usize,
    // end_of_month: bool,
    // utc: bool,//是否使用协调世界时（UTC）,暂未实现
    // dst: usize,//夏令时的偏移量，暂未实现
    // timezone: String,//表示时区的名称,暂未实现
}

// 使用枚举而不是结构体来模拟C的复杂成员

//
//pub fn calendar_standard_to_string(spec: &CalendarStandard)->Result<String,ParseError>{
//    unimplemented!()
//}
//pub fn calendar_standard_from_string(spec: &CalendarStandard)->Result<String,ParseError>{
//    unimplemented!()
//}

//@brief 解析日历格式，目前功能较弱，只能识别例如 Mon,Tue,Wed 2004-06-10 12:00:00的格式，
// 且暂不支持"*"，
// pub fn parse_calendar(s: &str) -> Result<CalendarStandard, String> {
//     // Weekbits YYYY-MM-DD HH:mm:SS ,目前只支持Weekbits用","隔开

// //      OnCalendar=*-*-* 00:00:00：每天的午夜触发。
// //      OnCalendar=*-*-* 08:00:00：每天早上8点触发。
// //      OnCalendar=Mon,Tue,Wed *-*-* 12:00:00：每周一、周二和周三的中午12点触发。
// //      OnCalendar=*-*-1 00:00:00：每个月的第一天午夜触发。
// //      OnCalendar=2019-01-01 00:00:00：指定的日期（2019年1月1日）触发。
//     let parts: Vec<&str> = s.split_whitespace().collect();

//      if parts.len() < 2 || parts.len() > 3 {
//          return Err("Invalid calendar format".to_string());
//      }
//      let mut index:usize=0;

//     let mut calendar = CalendarStandard {
//         weekdays_bits: Vec::default(),
//         year: 0,
//         month: 0,
//         day: 0,
//         hour: 0,
//         minute: 0,
//         microsecond: 0,
//         // end_of_month: false,
//         // utc: false,
//         // dst: 0,
//         // timezone: String::new(),
//     };

//     // 解析字符串并填充 calendar 结构体的字段
//     // 注意：这里的解析逻辑仅供示例，实际情况可能需要更复杂的处理

//     // 解析Weekbits
//     if parts.len() == 3 {
//         for day in parts[index].split(",") {
//             match day.trim() {
//                 "Mon" =>  calendar.weekdays_bits.push(WeekdayBits::Mon),
//                 "Tue" =>  calendar.weekdays_bits.push(WeekdayBits::Tue),
//                 "Wed" =>  calendar.weekdays_bits.push(WeekdayBits::Wed),
//                 "Thu" =>  calendar.weekdays_bits.push(WeekdayBits::Thu),
//                 "Fri" =>  calendar.weekdays_bits.push(WeekdayBits::Fri),
//                 "Sat" =>  calendar.weekdays_bits.push(WeekdayBits::Sat),
//                 "Sun" =>  calendar.weekdays_bits.push(WeekdayBits::Sun),
//                 _ => return Err("Invalid weekday".to_string()),
//             }
//         }
//         index+=1;
//     }
//       // 解析YYYY-MM-DD
//       let mut iter = parts[index].split("-");

//       let year = iter.next().unwrap().parse::<i32>().unwrap(); // 提取年并转换为i32类型
//       let month = iter.next().unwrap().parse::<i32>().unwrap(); // 提取月并转换为i32类型
//       let day = iter.next().unwrap().parse::<i32>().unwrap(); // 提取日并转换为i32类型
//       index+=1;

//       //解析HH:mm:SS
//       let mut iter = parts[index].split(":");

//       let year = iter.next().unwrap().parse::<i32>().unwrap(); // 提取年并转换为i32类型
//       let month = iter.next().unwrap().parse::<i32>().unwrap(); // 提取月并转换为i32类型
//       let day = iter.next().unwrap().parse::<i32>().unwrap(); // 提取日并转换为i32类型

//     Ok(calendar)
// }
