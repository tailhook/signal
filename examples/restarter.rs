extern crate signal;
extern crate nix;

use std::time::Duration;
use std::thread::sleep;
use std::str::FromStr;
use std::env::{args, vars_os, current_exe};

use nix::sys::signal::SIGQUIT;


fn main() {
    let num: u64 = FromStr::from_str(
        &args().skip(1).next().unwrap_or("0".to_string())
        ).unwrap();
    println!("Restared {} times", num);
    signal::exec_handler::set_command_line(
        current_exe().unwrap(),
        &["restarter".to_string(), (num+1).to_string()],
        vars_os());
    signal::exec_handler::set_handler(&[SIGQUIT], true).unwrap();
    sleep(Duration::new(10000, 0));
}
