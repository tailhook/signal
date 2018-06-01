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

use std::fmt;
use std::mem::uninitialized;
use std::ptr::null_mut;

use std::time::{Instant, Duration};
use nix::sys::signal::{sigaction, SigAction, Signal, SigSet, SaFlags};
use nix::sys::signal::{pthread_sigmask, SigmaskHow, SigHandler};
use nix::errno::{Errno, errno};
use libc::{self, timespec, sigwait};

/// A RAII guard for masking out signals and waiting for them synchronously
///
/// Trap temporarily replaces signal handlers to an empty handler, effectively
/// activating singnals that are ignored by default.
///
/// Old signal handlers are restored in `Drop` handler.
pub struct Trap {
    oldset: SigSet,
    oldsigs: Vec<(Signal, SigAction)>,
    sigset: SigSet,
}

extern "C" fn empty_handler(_: libc::c_int) { }

impl Trap {
    /// Create and activate the signal trap for specified signals. Signals not
    /// in list will be delivered asynchronously as always.
    pub fn trap(signals: &[Signal]) -> Trap {
        unsafe {
            let mut sigset = SigSet::empty();
            for &sig in signals {
                sigset.add(sig);
            }
            let mut oldset = uninitialized();
            let mut oldsigs = Vec::new();
            pthread_sigmask(SigmaskHow::SIG_BLOCK, Some(&sigset), Some(&mut oldset))
                .unwrap();
            // Set signal handlers to an empty function, this allows ignored
            // signals to become pending, effectively allowing them to be
            // waited for.
            for &sig in signals {
                oldsigs.push((sig, sigaction(sig,
                    &SigAction::new(SigHandler::Handler(empty_handler),
                        SaFlags::empty(), sigset))
                    .unwrap()));
            }
            Trap {
                oldset: oldset,
                oldsigs: oldsigs,
                sigset: sigset,
            }
        }
    }

    /// Wait until any of signals arrived or timeout occurs. In case of
    /// timeout returns None, otherwise returns signal number.
    ///
    /// Note the argument here is a deadline, not timeout. It's easier to work
    /// with deadline if you call wait() function in a loop.
    #[cfg(target_os = "linux")]
    pub fn wait(&self, deadline: Instant) -> Option<Signal> {
        use libc::sigtimedwait;

        loop {
            let now = Instant::now();
            let timeout = if deadline > now {
                deadline.duration_since(now)
            } else {
                Duration::from_secs(0)
            };
            let tm = timespec {
                tv_sec: timeout.as_secs() as libc::time_t,
                tv_nsec: (timeout - Duration::from_secs(timeout.as_secs()))
                         .subsec_nanos() as libc::c_long,
            };
            let sig = unsafe { sigtimedwait(self.sigset.as_ref(),
                                            null_mut(), &tm) };
            if sig > 0 {
                return Some(Signal::from_c_int(sig).unwrap());
            } else {
                match Errno::last() {
                    Errno::EAGAIN => {
                        return None;
                    }
                    Errno::EINTR => {
                        continue;
                    }
                    _ => {
                        panic!("Sigwait error: {}", errno());
                    }
                }
            }
        }
    }
}

impl Iterator for Trap {
    type Item = Signal;
    fn next(&mut self) -> Option<Signal> {
        let mut sig: libc::c_int = 0;
        loop {
            if unsafe { sigwait(self.sigset.as_ref(), &mut sig) } == 0 {
                return Some(Signal::from_c_int(sig).unwrap());
            } else {
                if Errno::last() == Errno::EINTR {
                    continue;
                }
                panic!("Sigwait error: {}", errno());
            }
        }
    }
}

impl Drop for Trap {
    fn drop(&mut self) {
        unsafe {
            for &(sig, ref sigact) in self.oldsigs.iter() {
                sigaction(sig, sigact).unwrap();
            }
            pthread_sigmask(SigmaskHow::SIG_SETMASK, Some(&self.oldset), None)
                .unwrap();
        }
    }
}

impl fmt::Debug for Trap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Trap")
        .finish()
    }
}
