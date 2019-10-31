#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use libc::{c_char, c_int, c_void};
use dlopen::symbor::{Symbol, SymBorApi};
use dlopen_derive::SymBorApi;
pub use dlopen::symbor::Container;

use std::ffi::{CStr, OsStr};


#[derive(SymBorApi)]
pub struct EzTransLib<'a> {
    J2K_InitializeEx: Symbol<'a, J2K_InitializeEx>,
    J2K_TranslateMMNT: Symbol<'a, J2K_TranslateMMNT>,
    J2K_Terminate: Symbol<'a, J2K_Terminate>,
}

type J2K_InitializeEx = unsafe extern "stdcall" fn(*const c_char, *const c_char) -> c_int;
type J2K_TranslateMMNT = unsafe extern "stdcall" fn(c_int, *const c_char) -> *mut c_char;
type J2K_Terminate = unsafe extern "stdcall" fn() -> c_int;

impl<'a> EzTransLib<'a> {
    /// return false when failed
    pub unsafe fn initialize(&self, init_str: &CStr, home_dir: &CStr) -> bool {
        let ret = (self.J2K_InitializeEx)(init_str.as_ptr(), home_dir.as_ptr());

        ret == 1
    }

    #[inline]
    pub unsafe fn translate(&self, shift_jis_str: &CStr) -> EzString {
        let ret = (self.J2K_TranslateMMNT)(0, shift_jis_str.as_ptr() as _);
        EzString(CStr::from_ptr(ret))
    }
}

impl<'a> Drop for EzTransLib<'a> {
    fn drop(&mut self) {
        unsafe {
            (self.J2K_Terminate)();
        }
    }
}

pub unsafe fn load_library(dll_path: impl AsRef<OsStr>) -> Result<Container<EzTransLib<'static>>, dlopen::Error> {
    Container::load(dll_path)
}

pub struct EzString(&'static CStr);

impl Drop for EzString {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.0.as_ptr() as *mut c_char as *mut c_void);
        }
    }
}

impl EzString {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.to_bytes()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
