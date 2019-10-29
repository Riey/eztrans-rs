#![allow(non_camel_case_types)]

extern crate libloading as lib;
extern crate encoding_rs;

use encoding_rs::{
    SHIFT_JIS,
    EUC_KR,
};

use std::os::raw::{
    c_char,
    c_int,
};

#[cfg(not(windows))]
use lib::os::unix::Symbol as LibSymbol;
#[cfg(windows)]
use lib::os::windows::Symbol as LibSymbol;

use std::ffi::{
    CString,
    OsStr,
    NulError,
};

pub struct EzTransLib {
    #[allow(dead_code)]
    lib: lib::Library,
    init_fn: LibSymbol<J2K_InitializeEx>,
    translate_fn: LibSymbol<J2K_TranslateMMNT>,
    terminate_fn: LibSymbol<J2K_Terminate>,
}

type J2K_InitializeEx = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;
type J2K_TranslateMMNT = unsafe extern "C" fn(c_int, *const c_char) -> *mut c_char;
type J2K_Terminate = unsafe extern "C" fn() -> c_int;

impl EzTransLib {
    pub fn new(lib: lib::Library) -> lib::Result<Self> {
        unsafe {
            Ok(Self {
                init_fn: lib.get::<J2K_InitializeEx>(stringify!(J2K_InitializeEx).as_bytes())?.into_raw(),
                translate_fn: lib.get::<J2K_TranslateMMNT>(stringify!(J2K_TranslateMMNT).as_bytes())?.into_raw(),
                terminate_fn: lib.get::<J2K_Terminate>(stringify!(J2K_Terminate).as_bytes())?.into_raw(),
                lib,
            })
        }
    }

    pub fn load_from(dll_path: impl AsRef<OsStr>) -> lib::Result<Self> {
        Ok(Self::new(lib::Library::new(dll_path)?)?)
    }

    pub fn initialize(&self, init_str: &str, home_dir: &str) -> Result<c_int, NulError> {
        unsafe {
            let init_str = CString::new(init_str)?;
            let home_dir = CString::new(home_dir)?;
            Ok((self.init_fn)(init_str.as_ptr(), home_dir.as_ptr()))
        }
    }

    pub fn translate(&self, original_str: &str) -> Result<String, String> {
        unsafe {
            let (res, _enc, errors) = SHIFT_JIS.encode(original_str);

            if errors {
                Err(format!("Encode [{}] to SHIFT_JIS failed", original_str))
            } else {
                let ret = (self.translate_fn)(0, res.as_ref().as_ptr() as _);
                let ret = CString::from_raw(ret);

                let (res, _env, errors) = EUC_KR.decode(ret.as_bytes());

                if errors {
                    Err(format!("Decode [{:?}] from EUC_KR failed", ret.as_bytes()))
                } else {
                    Ok(res.into_owned())
                }
            }
        }
    }
}

impl Drop for EzTransLib {
    fn drop(&mut self) {
        unsafe {
            (self.terminate_fn)();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
