use std::ffi::{CString, OsStr};
use std::os::unix::ffi::OsStrExt;

use nix::sys::signal::{SigNum};
use libc::{c_int, c_void, timespec, sigset_t};


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
extern {
    pub fn sigwait(set: *const sigset_t, sig: *mut SigNum) -> c_int;
    pub fn sigtimedwait(set: *const sigset_t, info: *mut c_void,
        timeout: *const timespec) -> c_int;
}
