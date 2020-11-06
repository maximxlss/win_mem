use winapi::um::winnt::{HANDLE, WCHAR};
use winapi::um::handleapi::CloseHandle;
use std::string::FromUtf16Error;

pub type WinResult<T> = Result<T, WinErrorKind>;

#[derive(Debug)]
pub enum WinErrorKind {
    ReadMemoryError,
    WriteMemoryError,
    FindProcessError,
    FindModuleError,
}

/// For internal use only: safe wrapper for [`CloseHandle`]
#[inline]
pub(crate) fn close_h(handle: HANDLE) {
    unsafe { CloseHandle(handle); }
}

pub(crate) fn remove_nil_bytes<const C_STR_SIZE: usize>(c_style_str: &[WCHAR; C_STR_SIZE]) -> Result<String, FromUtf16Error> {
    for i in 0..c_style_str.len() {
        if c_style_str[i] == 0 {
            return String::from_utf16(&c_style_str[..i]);
        }
    }
    // If loop falls thought it means all `C_STR_SIZE`
    // `WCHAR`s of the `c_style_str` were non-nil
    String::from_utf16(c_style_str)
}