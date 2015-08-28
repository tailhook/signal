//! Trap for handling signals synchronously
//!
//! It works as follows:
//!
//! 1. You create a trap (`Trap::trap()`), that is RAII-style guard that masks
//!    out signals and unignores them, preparing them for be handled when user
//!    wants
//! 2. Use trap as iterator yielding signals or `trap.wait(timeout)`
//!
//! Especially useful for running (multiple) child processes simultaneously.

use std::mem::uninitialized;
use std::ptr::null_mut;

use nix::sys::signal::{sigaction, SigAction, SigNum, SigSet, SockFlag};
use nix::errno::errno;

use ffi::{pthread_sigmask, sigwait, SIG_BLOCK, SIG_UNBLOCK};

pub struct Trap {
    oldset: SigSet,
    oldsigs: Vec<(SigNum, SigAction)>,
    sigset: SigSet,
}

extern "C" fn empty_handler(_: SigNum) { }

impl Trap {
    pub fn trap(signals: &[SigNum]) -> Trap {
        unsafe {
            let mut sigset = SigSet::empty();
            for &sig in signals {
                sigset.add(sig).unwrap();
            }
            let mut oldset = uninitialized();
            let mut oldsigs = Vec::new();
            pthread_sigmask(SIG_BLOCK, &sigset, &mut oldset);
            for &sig in signals {
                oldsigs.push((sig, sigaction(sig,
                    &SigAction::new(empty_handler, SockFlag::empty(), sigset))
                    .unwrap()));
            }
            Trap {
                oldset: oldset,
                oldsigs: oldsigs,
                sigset: sigset,
            }
        }
    }
}

impl Iterator for Trap {
    type Item = SigNum;
    fn next(&mut self) -> Option<SigNum> {
        let mut sig: SigNum = 0;
        if unsafe { sigwait(&self.sigset, &mut sig) } == 0 {
            return Some(sig);
        } else {
            panic!("Sigwait error: {}", errno());
        }
    }
}

impl Drop for Trap {
    fn drop(&mut self) {
        unsafe {
            for &(sig, ref sigact) in self.oldsigs.iter() {
                sigaction(sig, sigact).unwrap();
            }
            pthread_sigmask(SIG_UNBLOCK, &self.oldset, null_mut());
        }
    }
}
