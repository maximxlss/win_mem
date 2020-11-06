use std::mem::{size_of, zeroed};
use std::ptr::null_mut;

use winapi::shared::minwindef::{DWORD, FALSE, LPCVOID, LPVOID, TRUE};
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::memoryapi::{ReadProcessMemory, WriteProcessMemory};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::tlhelp32::{Module32FirstW, Module32NextW, MODULEENTRY32W, Process32FirstW, Process32NextW, PROCESSENTRY32W};
use winapi::um::winnt::{HANDLE, PROCESS_ALL_ACCESS};

use module::Module;
use snapshot::Snapshot;
use utils::{close_h, WinErrorKind, WinResult, remove_nil_bytes};

/// Represents a system process, posses a PID, name and an open [`HANDLE`]
pub struct Process {
    name: String,
    pid: DWORD,
    handle: HANDLE,
}

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

            if !snapshot.handle().is_null() &&
                snapshot.handle() != INVALID_HANDLE_VALUE &&
                Process32FirstW(snapshot.handle(), &mut p_entry) != FALSE {
                while {
                    if let Ok(p_name) = remove_nil_bytes(&p_entry.szExeFile) {
                        if p_name.starts_with(name) {
                            let pid = p_entry.th32ProcessID;
                            // Desire all access despite *probably* only needing VM_READ and VM_WRITE
                            let h_proc = OpenProcess(PROCESS_ALL_ACCESS, FALSE, pid);
                            return Ok(Process {
                                name: p_name,
                                pid,
                                handle: h_proc,
                            });
                        }
                    }

                    Process32NextW(snapshot.handle(), &mut p_entry) != FALSE
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

            if !snapshot.handle().is_null() &&
                snapshot.handle() != INVALID_HANDLE_VALUE &&
                Module32FirstW(snapshot.handle(), &mut m_entry) != FALSE {
                while {
                    if let Ok(m_name) = remove_nil_bytes(&m_entry.szModule) {
                        if m_name.starts_with(name) {
                            return Ok(Module {
                                name: m_name,
                                address: m_entry.modBaseAddr as DWORD,
                                len: m_entry.modBaseSize,
                            });
                        }
                    }

                    Module32NextW(snapshot.handle(), &mut m_entry) != FALSE
                } {}
            }
        }

        Err(WinErrorKind::FindModuleError)
    }

    /// Write to a process's memory, not relative to module offset
    /// [Reference]: https://docs.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-writeprocessmemory
    pub fn write_mem<T>(&self, buffer: &T, address: DWORD) -> WinResult<()> {
        unsafe {
            if WriteProcessMemory(self.handle,
                                  address as LPVOID,
                                  buffer as *const T as LPCVOID,
                                  size_of::<T>(),
                                  null_mut()) == TRUE {
                Ok(())
            } else {
                Err(WinErrorKind::WriteMemoryError)
            }
        }
    }

    /// Write to a process's memory relative to the offset of a module
    #[inline]
    pub fn write_mem_relative<T>(&self, buffer: &T, module_name: &str, address: DWORD) -> WinResult<()> {
        if let Ok(module) = self.find_module(module_name) {
            self.write_mem(buffer, module.address() + address)
        } else {
            Err(WinErrorKind::WriteMemoryError)
        }
    }

    /// Read a process's memory, not relative to module offset
    /// [Reference]: https://docs.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-readprocessmemory
    pub fn read_mem<T>(&self, address: DWORD) -> WinResult<T> {
        unsafe {
            // Initialize buffer
            let mut buf = zeroed::<T>();
            if ReadProcessMemory(self.handle,
                                 address as LPVOID,
                                 &mut buf as *mut T as LPVOID,
                                 size_of::<T>(),
                                 null_mut()) == TRUE {
                Ok(buf)
            } else {
                Err(WinErrorKind::ReadMemoryError)
            }
        }
    }

    /// Read a process's memory address relative to the offset of a module
    #[inline]
    pub fn read_mem_relative<T>(&self, module_name: &str, address: DWORD) -> WinResult<T> {
        if let Ok(module) = self.find_module(module_name) {
            self.read_mem(module.address() + address)
        } else {
            Err(WinErrorKind::ReadMemoryError)
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
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

/// Close [`Process`] [`HANDLE`] when [`Process`] goes out of scope or the program exits/panics
impl Drop for Process {
    fn drop(&mut self) {
        close_h(self.handle)
    }
}
