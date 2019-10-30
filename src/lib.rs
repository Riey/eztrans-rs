#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(feature = "encoding")]
extern crate encoding_rs;

#[cfg(feature = "encoding")]
use encoding_rs::{EUC_KR, SHIFT_JIS};

use libc::{c_char, c_int, c_void};
use dlopen::symbor::{Symbol, SymBorApi, Container};
use dlopen_derive::SymBorApi;

use std::ffi::{CStr, CString, OsStr};

#[derive(SymBorApi)]
pub struct EzTransLib<'a> {
    J2K_InitializeEx: Symbol<'a, J2K_InitializeEx>,
    J2K_TranslateMMNT: Symbol<'a, J2K_TranslateMMNT>,
    J2K_Terminate: Symbol<'a, J2K_Terminate>,
}

type J2K_InitializeEx = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;
type J2K_TranslateMMNT = unsafe extern "C" fn(c_int, *const c_char) -> *mut c_char;
type J2K_Terminate = unsafe extern "C" fn() -> c_int;

impl<'a> EzTransLib<'a> {
    pub fn initialize(&self, init_str: &str, home_dir: &str) -> c_int {
        unsafe {
            let init_str = CString::new(init_str).unwrap();
            let home_dir = CString::new(home_dir).unwrap();
            (self.J2K_InitializeEx)(init_str.as_ptr(), home_dir.as_ptr())
        }
    }

    #[cfg(feature = "encoding")]
    pub fn translate(&self, original_str: &str) -> Result<String, String> {
        let (res, _enc, errors) = SHIFT_JIS.encode(original_str);

        if errors {
            Err(format!("Encode [{}] to SHIFT_JIS failed", original_str))
        } else {
            let ret = self.translate_raw(res.as_ref());

            let (res, _) = EUC_KR.decode_without_bom_handling(ret.as_bytes());

            Ok(res.into_owned())
        }
    }

    #[inline]
    pub fn translate_raw(&self, original_str: &[u8]) -> EzString {
        unsafe {
            let ret = (self.J2K_TranslateMMNT)(0, original_str.as_ptr() as _);
            EzString(CStr::from_ptr(ret))
        }
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
