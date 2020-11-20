use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, TH32CS_SNAPPROCESS};
use winapi::um::winnt::HANDLE;

use process::Process;
use utils::close_h;

/// A Snapshot handle
///
/// [Reference]: https://docs.microsoft.com/en-us/windows/win32/api/tlhelp32/nf-tlhelp32-createtoolhelp32snapshot
pub struct Snapshot(HANDLE);

impl Snapshot {
    /// Creates a snapshot handle to parse into [`Process32First(W)`] and [`Process32Next(W)`]
    #[inline]
    pub fn process() -> Self {
        unsafe { Snapshot(CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)) }
    }
    /// Creates a snapshot handle to parse into [`Module32First(W)`] and [`Module32Next(W)`]
    #[inline]
    pub fn module(process: &Process) -> Self {
        unsafe { Snapshot(CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, process.pid())) }
    }
    /// Returns the handle created by [`CreateToolhelp32Snapshot`]
    #[inline]
    pub fn handle(&self) -> HANDLE {
        self.0
    }
}

/// Close the Snapshot handle when variable goes out of scope so we
/// don't have to worry about closing it ourselves
impl Drop for Snapshot {
    fn drop(&mut self) {
        close_h(self.0)
    }
}