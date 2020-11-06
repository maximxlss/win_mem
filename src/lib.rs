extern crate winapi;

use std::mem::{size_of, zeroed};

use winapi::_core::ptr::null_mut;
use winapi::shared::minwindef::{DWORD, FALSE, LPCVOID, LPVOID, TRUE};
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::memoryapi::{ReadProcessMemory, WriteProcessMemory};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Module32FirstW, Module32NextW, MODULEENTRY32W, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, TH32CS_SNAPPROCESS};
use winapi::um::winnt::{HANDLE, PROCESS_ALL_ACCESS};

/// Represents a system process, posses a PID and an open [`HANDLE`]
///
/// [Reference]: https://docs.microsoft.com/en-us/windows/win32/api/tlhelp32/nf-tlhelp32-createtoolhelp32snapshot
pub struct Process {
    pid: DWORD,
    handle: HANDLE,
}

pub struct Module {
    name: String,
    /// Memory address of the [`Module`] relative to the process
    address: DWORD,
    /// Length of the [`Module`] in bytes
    len: DWORD,
}

/// For internal use only!
struct Snapshot {
    handle: HANDLE,
}

pub type WinResult<T> = Result<T, WinErrorKind>;

#[derive(Debug)]
pub enum WinErrorKind {
    ReadMemoryError,
    WriteMemoryError,
    FindProcessError,
    FindModuleError,
}

impl Snapshot {
    #[inline]
    fn process() -> Self {
        unsafe {
            Snapshot {
                handle: CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
            }
        }
    }

    #[inline]
    fn module(process: &Process) -> Self {
        unsafe {
            Snapshot {
                handle: CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, process.pid)
            }
        }
    }
}

/// Close [`Snapshot#handle`] when variable goes out of scope so we
/// don't have to worry about closing it ourselves
impl Drop for Snapshot {
    fn drop(&mut self) {
        close_h(self.handle)
    }
}

impl Module {
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[inline]
    pub fn address(&self) -> DWORD {
        self.address
    }
    #[inline]
    pub fn len(&self) -> DWORD {
        self.len
    }
    /// True: if the module size ([`modBaseSize`]) is 0
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Close [`Process`] [`HANDLE`] when [`Process`] goes out of scope or the program exits/panics
impl Drop for Process {
    fn drop(&mut self) {
        close_h(self.handle)
    }
}

/// TODO: is #[inline] on getters a great idea?
impl Process {
    /// Find a [`Process`] from it's executable's name
    /// [Reference(s)]:
    /// https://docs.microsoft.com/en-us/windows/win32/api/tlhelp32/nf-tlhelp32-process32firstw
    /// https://docs.microsoft.com/en-us/windows/win32/api/tlhelp32/nf-tlhelp32-process32nextw
    /// https://docs.microsoft.com/en-us/windows/win32/api/tlhelp32/ns-tlhelp32-processentry32w
    pub fn find(name: &str) -> WinResult<Self> {
        unsafe {
            let snapshot = Snapshot::process();

            let mut p_entry = zeroed::<PROCESSENTRY32W>();
            // `dwSize` must be initialized with size of PROCESSENTRY32W before Process32FirstW or Process32NextW are called
            p_entry.dwSize = size_of::<PROCESSENTRY32W>() as DWORD;

            if !snapshot.handle.is_null() &&
                snapshot.handle != INVALID_HANDLE_VALUE &&
                Process32FirstW(snapshot.handle, &mut p_entry) != FALSE {
                while {
                    if let Ok(p_name) = String::from_utf16(&p_entry.szExeFile) {
                        if p_name.starts_with(name) {
                            let pid = p_entry.th32ProcessID;
                            // Desire all access despite *probably* only needing VM_READ and VM_WRITE
                            let h_proc = OpenProcess(PROCESS_ALL_ACCESS, FALSE, pid);
                            return Ok(Process {
                                pid,
                                handle: h_proc,
                            });
                        }
                    }

                    Process32NextW(snapshot.handle, &mut p_entry) != FALSE
                } {}
            }
        }

        Err(WinErrorKind::FindProcessError)
    }

    /// Find a process's module (dll) by it's name
    /// [Reference(s)]:
    /// https://docs.microsoft.com/en-us/windows/win32/api/tlhelp32/nf-tlhelp32-module32firstw
    /// https://docs.microsoft.com/en-us/windows/win32/api/tlhelp32/nf-tlhelp32-module32nextw
    /// https://docs.microsoft.com/en-us/windows/win32/api/tlhelp32/ns-tlhelp32-moduleentry32w
    pub fn find_module(&self, name: &str) -> WinResult<Module> {
        unsafe {
            let snapshot = Snapshot::module(self);

            let mut m_entry = zeroed::<MODULEENTRY32W>();
            // `dwSize` must be initialized with size of MODULEENTRY32W before Module32FirstW or Module32NextW are called
            m_entry.dwSize = size_of::<MODULEENTRY32W>() as DWORD;

            if !snapshot.handle.is_null() &&
                snapshot.handle != INVALID_HANDLE_VALUE &&
                Module32FirstW(snapshot.handle, &mut m_entry) != FALSE {
                while {
                    if let Ok(m_name) = String::from_utf16(&m_entry.szModule) {
                        // Compare with starts_with() because `m_name` will have `256 - len` nil bytes ('\0`) @ end
                        if m_name.starts_with(name) {
                            return Ok(Module {
                                name: m_name,
                                address: m_entry.modBaseAddr as DWORD,
                                len: m_entry.modBaseSize,
                            });
                        }
                    }

                    Module32NextW(snapshot.handle, &mut m_entry) != FALSE
                } {}
            }
        }

        Err(WinErrorKind::FindModuleError)
    }

    /// Write to a processes memory, not relative to module offset
    /// [Reference]: https://docs.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-writeprocessmemory
    pub fn write_memory<T>(&self, buffer: &T, address: DWORD) -> WinResult<()> {
        unsafe {
            if WriteProcessMemory(self.handle,
                                         address as LPVOID,
                                         buffer as *const T as LPCVOID,
                                         size_of::<T>(),
                                         null_mut()) == TRUE {
                Ok(())
            } else { Err(WinErrorKind::WriteMemoryError) }
        }
    }

    /// Read a processes memory, not relative to module offset
    /// [Reference]: https://docs.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-readprocessmemory
    pub fn read_memory<T>(&self, address: DWORD) -> WinResult<T> {
        unsafe {
            // Initialize buffer
            let mut buf = zeroed::<T>();
            if ReadProcessMemory(self.handle,
                                        address as LPVOID,
                                        &mut buf as *mut T as LPVOID,
                                        size_of::<T>(),
                                        null_mut()) == TRUE {
                Ok(buf)
            } else { Err(WinErrorKind::ReadMemoryError) }
        }
    }

    #[inline]
    pub fn pid(&self) -> DWORD {
        self.pid
    }

    #[inline]
    pub fn handle(&self) -> HANDLE {
        self.handle
    }
}

/// For internal use only: safe wrapper for [`CloseHandle`]
#[inline]
fn close_h(handle: HANDLE) {
    unsafe { CloseHandle(handle); }
}

#[cfg(test)]
mod tests {
    use Process;

    /// Find the 'firefox' process
    fn firefox() -> Process {
        Process::find("firefox.exe")
            .expect("Could not find process 'firefox.exe'")
    }

    /// Print the PID of firefox
    #[test]
    fn get_firefox_pid() {
        println!("Firefox PID = {}", firefox().pid())
    }

    /// Find and print the address of the DirectX11 DLL in firefox
    #[test]
    fn find_directx_11_module_firefox() {
        println!("Module Address = {}", firefox()
            .find_module("d3d11.dll")
            .expect("Could not find the 'd3d11.dll' module in the 'firefox.exe' process")
            .address())
    }
}