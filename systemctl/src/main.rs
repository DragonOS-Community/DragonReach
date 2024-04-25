use std::{env, fs::File, io::Write};

const REACH_PIPE_PATH: &str = "/home/fz/myetc/reach/ipc/ctl";
//const REACH_PIPE_PATH: &str = "etc/reach/ipc/ctl";

fn main() {
    let mut args: Vec<String> = env::args().collect();

    if args.len() <= 2 {
        args = Vec::new();
        args.push(String::from("list-units"));
    } else {
        args.remove(0);
        args.remove(0);
    }

    let mut msg = String::new();
    for arg in args {
        msg = format!("{} {}", msg, arg);
    }

    let mut file = File::open(REACH_PIPE_PATH).unwrap();
    if let Err(err) = file.write_all(msg.as_bytes()) {
        eprintln!("write error {}", err);
        eprintln!("write error {:?}", err);
    }
}
