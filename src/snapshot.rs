use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, TH32CS_SNAPPROCESS};
use winapi::um::winnt::HANDLE;

use process::Process;
use utils::close_h;

/// A Snapshot handle
///
/// [Reference]: https://docs.microsoft.com/en-us/windows/win32/api/tlhelp32/nf-tlhelp32-createtoolhelp32snapshot
pub struct Snapshot(HANDLE);

impl Snapshot {
    #[inline]
    pub fn process() -> Self {
        unsafe { Snapshot(CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)) }
    }

    #[inline]
    pub fn module(process: &Process) -> Self {
        unsafe { Snapshot(CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, process.pid())) }
    }

    #[inline]
    pub fn handle(&self) -> HANDLE {
        self.0
    }
}

/// Close [`Snapshot#handle`] when variable goes out of scope so we
/// don't have to worry about closing it ourselves
impl Drop for Snapshot {
    fn drop(&mut self) {
        close_h(self.0)
    }
}