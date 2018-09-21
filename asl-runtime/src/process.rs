use winapi::shared::minwindef::{BOOL, DWORD};
use winapi::um::{
    handleapi::{CloseHandle, INVALID_HANDLE_VALUE}, memoryapi::ReadProcessMemory,
    processthreadsapi::{GetProcessTimes, OpenProcess},
    tlhelp32::{
        CreateToolhelp32Snapshot, MODULEENTRY32W, Module32FirstW, Module32NextW, PROCESSENTRY32W,
        Process32FirstW, Process32NextW, TH32CS_SNAPMODULE, TH32CS_SNAPPROCESS,
    },
    winnt::{HANDLE, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
};

use std::collections::HashMap;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::{mem, result, slice};

type Address = u64;
pub type Offset = i64;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        ListProcesses {}
        ProcessDoesntExist {}
        ListModules {}
        OpenProcess {}
        ModuleDoesntExist {}
        ReadMemory {}
    }
}

pub type Result<T> = result::Result<T, Error>;

pub struct Process {
    handle: HANDLE,
    modules: HashMap<String, Address>,
    is_64bit: bool,
}

impl Drop for Process {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

impl Process {
    pub fn is_64bit(&self) -> bool {
        self.is_64bit
    }

    pub fn with_name(name: &str) -> Result<Self> {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);

            if snapshot == INVALID_HANDLE_VALUE {
                return Err(Error::ListProcesses);
            }

            let mut creation_time = mem::uninitialized();
            let mut exit_time = mem::uninitialized();
            let mut kernel_time = mem::uninitialized();
            let mut user_time = mem::uninitialized();

            let mut best_process = None::<(DWORD, u64)>;
            let mut entry: PROCESSENTRY32W = mem::uninitialized();
            entry.dwSize = mem::size_of_val(&entry) as _;

            if Process32FirstW(snapshot, &mut entry) != 0 {
                loop {
                    {
                        let entry_name = &entry.szExeFile;
                        let len = entry_name.iter().take_while(|&&c| c != 0).count();
                        let entry_name = &entry_name[..len];
                        let entry_name = &OsString::from_wide(entry_name);
                        if entry_name == name {
                            let pid = entry.th32ProcessID;
                            let process = OpenProcess(PROCESS_QUERY_INFORMATION, false as _, pid);

                            if !process.is_null() {
                                let success = GetProcessTimes(
                                    process,
                                    &mut creation_time,
                                    &mut exit_time,
                                    &mut kernel_time,
                                    &mut user_time,
                                );
                                if success != 0 {
                                    let time = (creation_time.dwHighDateTime as u64) << 32
                                        | (creation_time.dwLowDateTime as u64);

                                    if best_process.map_or(true, |(_, oldest)| time > oldest) {
                                        best_process = Some((pid, time));
                                    }
                                }

                                CloseHandle(process);
                            }
                        }
                    }

                    if Process32NextW(snapshot, &mut entry) == 0 {
                        break;
                    }
                }
            }

            CloseHandle(snapshot);

            if let Some((pid, _)) = best_process {
                Process::with_pid(pid)
            } else {
                Err(Error::ProcessDoesntExist)
            }
        }
    }

    pub fn with_pid(pid: DWORD) -> Result<Self> {
        unsafe {
            let handle = OpenProcess(PROCESS_VM_READ | PROCESS_QUERY_INFORMATION, false as _, pid);

            if !handle.is_null() {
                let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pid);

                if snapshot == INVALID_HANDLE_VALUE {
                    CloseHandle(handle);
                    return Err(Error::ListModules);
                }

                let mut modules = HashMap::new();
                let mut entry: MODULEENTRY32W = mem::uninitialized();
                entry.dwSize = mem::size_of_val(&entry) as _;

                if Module32FirstW(snapshot, &mut entry) != 0 {
                    loop {
                        {
                            let base_address = entry.modBaseAddr as Address;
                            let name = &entry.szModule;
                            let len = name.iter().take_while(|&&c| c != 0).count();
                            let name = &name[..len];
                            let name = OsString::from_wide(name).to_string_lossy().into_owned();
                            modules.insert(name, base_address);
                        }

                        if Module32NextW(snapshot, &mut entry) == 0 {
                            break;
                        }
                    }
                }

                let is_64bit;
                #[cfg(target_pointer_width = "64")]
                {
                    use winapi::um::wow64apiset::IsWow64Process;

                    let mut pbool: BOOL = 0;
                    IsWow64Process(handle, &mut pbool);
                    is_64bit = pbool == 0;
                }
                #[cfg(not(target_pointer_width = "64"))]
                {
                    // TODO Actually idk if 32-bit apps can read from 64-bit
                    // apps. If they can, then this is wrong.
                    is_64bit = false;
                }

                CloseHandle(snapshot);

                Ok(Self {
                    handle,
                    modules,
                    is_64bit,
                })
            } else {
                Err(Error::OpenProcess)
            }
        }
    }

    pub fn module_address(&self, module: &str) -> Result<Address> {
        self.modules
            .get(module)
            .cloned()
            .ok_or(Error::ModuleDoesntExist)
    }

    pub fn read_buf(&self, address: Address, buf: &mut [u8]) -> Result<()> {
        unsafe {
            let mut bytes_read = mem::uninitialized();

            let successful = ReadProcessMemory(
                self.handle,
                address as _,
                buf.as_mut_ptr() as _,
                buf.len() as _,
                &mut bytes_read,
            ) != 0;

            if successful && bytes_read as usize == buf.len() {
                Ok(())
            } else {
                Err(Error::ReadMemory)
            }
        }
    }

    pub fn read<T: Copy>(&self, address: Address) -> Result<T> {
        // TODO Unsound af
        unsafe {
            let mut res = mem::uninitialized();
            let buf = slice::from_raw_parts_mut(mem::transmute(&mut res), mem::size_of::<T>());
            self.read_buf(address, buf).map(|_| res)
        }
    }
}
