//! Signal handling library
//!
//! The library is focused on higher-level abstractions for handling signals.
//! All low-level stuff should be in `nix`.
//!
//! Currently we have two mechanisms for handling exeptions:
//!
//! 1. The `exec_handler` module for replacing process with newly runned
//!    command designed as crash safety measure
//! 2. The `Trap` mechanism that masks out signals and allows wait for them
//!    explicitly
//!
//!
//! Both are specifically suited for making process supervisors.
//!
//! Note, masking out signals may also be achieved by trap (just don't call
//! either `wait()` or `next()`)
//!
//! On TODO list:
//!
//! * `signalfd`
//!
//! The library tested only on linux
//!

extern crate libc;
extern crate nix;

mod ffi;
pub mod exec_handler;
pub mod trap;
