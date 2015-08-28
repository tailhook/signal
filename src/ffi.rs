use std::ffi::{CString, OsStr};
use std::os::unix::ffi::OsStrExt;

use nix::sys::signal::{SigSet, SigNum};
use libc::c_int;


pub trait ToCString {
    fn to_cstring(&self) -> CString;
    fn as_bytes(&self) -> &[u8];
}

impl<T:AsRef<OsStr>> ToCString for T {
    fn to_cstring(&self) -> CString {
        CString::new(self.as_ref().as_bytes())
        .unwrap()
    }
    fn as_bytes(&self) -> &[u8] {
        self.as_ref().as_bytes()
    }
}


// All the following should be moved to nix-rust

pub const SIG_BLOCK: c_int = 0;
pub const SIG_UNBLOCK: c_int = 1;
extern {
    pub fn pthread_sigmask(how: c_int, set: *const SigSet,
                           oldset: *mut SigSet) -> c_int;
    pub fn sigwait(set: *const SigSet, sig: *mut SigNum) -> c_int;
}
