use winapi::um::winnt::HANDLE;
use winapi::um::handleapi::CloseHandle;

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