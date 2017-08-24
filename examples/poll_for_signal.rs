extern crate signal;
extern crate nix;

use std::time::{Instant, Duration};
use std::thread::sleep;

use nix::sys::signal::{SIGINT};

use signal::trap::Trap;


#[cfg(target_os="linux")]
fn main() {
    let trap = Trap::trap(&[SIGINT]);
    loop {
        if let Some(SIGINT) = trap.wait(Instant::now()) {
            println!("Gracefully interrupted...");
            break;
        }
        sleep(Duration::from_millis(100));
    }
}

#[cfg(not(target_os="linux"))]
fn main() {
    println!("unfortunately this example works only for linux");
}
