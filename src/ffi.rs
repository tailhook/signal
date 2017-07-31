use std::ffi::{CString, OsStr};
use std::os::unix::ffi::OsStrExt;

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

