//! Set a signal handler that executes command-line
//!
//! This is supposed to work on panics, deadlocks and event many kinds of
//! memory errors. But may leak file descriptors which have no CLOEXEC flag set
//!
//! This is used as fail-safe signal handler for the supervisors, which are
//! able to re-execute in place and continue to work. Also may be used for
//! configuration reloading signal (e.g. SIGHUP) if appropriate.

use std::io::Write;
use std::mem::{forget, transmute};
use std::ptr::{null, null_mut};
use std::ffi::CString;
use std::env::{current_exe, args_os, vars_os};

use nix;
use libc::{execve, c_char};
use nix::sys::signal::{sigaction, SigAction, SigNum, SigSet, SockFlag};

use ffi::{ToCString, pthread_sigmask, SIG_UNBLOCK};


static mut exec_command_line: *const ExecCommandLine =
    0usize as *const ExecCommandLine;


#[allow(unused)]
struct ExecCommandLine {
    program: CString,
    args: Vec<CString>,
    c_args: Vec<*const c_char>,
    env: Vec<CString>,
    c_env: Vec<*const c_char>,
}


/// Sets command-line and environment to execute when signal happens
///
/// If nothing is set current command-line is used.
///
/// You may change it freely even after calling set_handler. But only single
/// command-line may be set for all signal handlers used in this module.
pub fn set_command_line<P, Ai, A, Ek, Ev, E>(program: P, args: A, environ: E)
    where P: ToCString,
          Ai: ToCString,
          A: IntoIterator<Item=Ai>,
          Ek: ToCString,
          Ev: ToCString,
          E: IntoIterator<Item=(Ek, Ev)>,
{
    let args = args.into_iter().map(|x| x.to_cstring()).collect::<Vec<_>>();
    let mut c_args = args.iter().map(|x| x.as_ptr()).collect::<Vec<_>>();
    c_args.push(null());
    let env = environ.into_iter().map(|(k, v)| {
        let mut pair = Vec::new();
        pair.write(k.as_bytes()).unwrap();
        pair.push(b'=');
        pair.write(v.as_bytes()).unwrap();
        CString::new(pair).unwrap()
    }).collect::<Vec<_>>();
    let mut c_env = env.iter().map(|x| x.as_ptr()).collect::<Vec<_>>();
    c_env.push(null());
    unsafe {
        if exec_command_line != null() {
            transmute::<_, Box<ExecCommandLine>>(exec_command_line);
        }
        let new = Box::new(ExecCommandLine {
            program: program.to_cstring(),
            args: args,
            c_args: c_args,
            env: env,
            c_env: c_env,
        });

        exec_command_line = &*new;
        forget(new);
    }
}

extern "C" fn exec_handler(sig: SigNum) {
    let err = unsafe {
        execve((*exec_command_line).program.as_ptr(),
               (*exec_command_line).c_args.as_ptr(),
               (*exec_command_line).c_env.as_ptr())
    };
    panic!("Couldn't exec on signal {}, err code {}", sig, err);
}


/// Set a handler for multiple signals. If no `set_command_line` was called
/// before this function the command-line is set from ``std::env``
///
/// The `avoid_race_condition` fixes race condition when one of the signals in
/// set is delivered second time before the new process is able to set signal
/// handler itself (most probably leading to process death). But if it's set
/// to `true` process is started with another signal mask, so it should fix
/// signal mask (which this function does too).
///
/// In other words `avoid_race_condition=true` is useful is the same program
/// is executed and it calls `set_handler` on early startup.
///
/// For `avoid_race_condition=true` is also important to use single call for
/// `set_handler` for all signals, because it avoids race condition between
/// all combinations of subsequent signals in the set.
pub fn set_handler(signals: &[SigNum], avoid_race_condition: bool)
    -> nix::Result<()>
{
    unsafe {
        if exec_command_line == null() {
            set_command_line(current_exe().unwrap(), args_os(), vars_os());
        }
        let mut sigset = SigSet::empty();
        if avoid_race_condition {
            for &sig in signals {
                sigset.add(sig).unwrap();
            }
        }
        let mut res = Ok(());
        for &sig in signals {
            res = res.and_then(|()| {
                try!(sigaction(sig, &SigAction::new(exec_handler,
                    SockFlag::empty(), sigset)));
                Ok(())
            });
        }
        // TODO(tailhook) is this error reporting is ok? or maybe just panic?
        if avoid_race_condition && res.is_ok() {
            pthread_sigmask(SIG_UNBLOCK, &sigset, null_mut());
        }
        res
    }
}
