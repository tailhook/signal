extern crate signal;
extern crate nix;

use std::env::{args};
use std::process::Command;

use nix::Error;
use nix::errno::Errno;
use nix::sys::signal::{SIGTERM, SIGINT, SIGCHLD};
use nix::sys::wait::{waitpid, WaitPidFlag};
use nix::sys::wait::WaitStatus::{Exited, Signaled, StillAlive};
use nix::libc::{c_int};


fn main() {
    let args = args().skip(1).collect::<Vec<_>>();
    let commandlines = args.split(|x| &x[..] == "---");

    for cline in commandlines {
        let mut cmd = Command::new(&cline[0]);
        cmd.args(&cline[1..]);
        println!("Starting {:?}", cmd);
        cmd.spawn().unwrap();
    }

    let trap = signal::trap::Trap::trap(&[SIGTERM, SIGINT, SIGCHLD]);
    for sig in trap {
        match sig {
            SIGCHLD => {
                // Current std::process::Command ip does not have a way to find
                // process id, so we just wait until we have no children
                loop {
                    match waitpid(None, Some(WaitPidFlag::WNOHANG)) {
                        Ok(Exited(pid, status)) => {
                            println!("{} exited with status {}", pid, status);
                            continue;
                        }
                        Ok(Signaled(pid, sig, _)) => {
                            println!("{} killed by {}", pid, sig as c_int);
                            continue;
                        }
                        Ok(StillAlive) => { break; }
                        Ok(status) => {
                            println!("Temporary status {:?}", status);
                            continue;
                        }
                        Err(Error::Sys(Errno::ECHILD)) => {
                            return;
                        }
                        Err(e) => {
                            panic!("Error {:?}", e);
                        }
                    }
                }
            }
            sig => {
                println!("Stopping because of {}", sig as c_int);
                // At this stage is probably good idea to forward signal
                // to children and wait until they all dead, but we omit
                // it for brevity
                break;
            }
        }
    }
}
