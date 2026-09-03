#![cfg(target_os = "windows")]
#![allow(non_snake_case, non_camel_case_types, dead_code)]

use std::ffi::c_void;
use std::ptr;
use serde_json::{json, Value};
use crate::native_scan::{self, ParsedValue, ScanMatch, ScanType};

pub type HANDLE = *mut c_void;
pub type BOOL = i32;
pub type DWORD = u32;
pub type WORD = u16;
pub type ULONG_PTR = usize;
pub type SIZE_T = usize;
pub type WCHAR = u16;

pub const FALSE: BOOL = 0;
pub const TRUE: BOOL = 1;
pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;

pub const PROCESS_VM_READ: DWORD = 0x0010;
pub const PROCESS_VM_WRITE: DWORD = 0x0020;
pub const PROCESS_VM_OPERATION: DWORD = 0x0008;
pub const PROCESS_QUERY_INFORMATION: DWORD = 0x0400;
pub const PROCESS_QUERY_LIMITED_INFORMATION: DWORD = 0x1000;

pub const TH32CS_SNAPPROCESS: DWORD = 0x00000002;
pub const TH32CS_SNAPMODULE: DWORD = 0x00000008;
pub const TH32CS_SNAPMODULE32: DWORD = 0x00000010;

pub const MEM_COMMIT: DWORD = 0x1000;

pub const PAGE_NOACCESS: DWORD = 0x01;
pub const PAGE_READONLY: DWORD = 0x02;
pub const PAGE_READWRITE: DWORD = 0x04;
pub const PAGE_WRITECOPY: DWORD = 0x08;
pub const PAGE_EXECUTE: DWORD = 0x10;
pub const PAGE_EXECUTE_READ: DWORD = 0x20;
pub const PAGE_EXECUTE_READWRITE: DWORD = 0x40;
pub const PAGE_EXECUTE_WRITECOPY: DWORD = 0x80;
pub const PAGE_GUARD: DWORD = 0x100;

#[repr(C)]
pub struct PROCESSENTRY32W {
    pub dwSize: DWORD,
    pub cntUsage: DWORD,
    pub th32ProcessID: DWORD,
    pub th32DefaultHeapID: ULONG_PTR,
    pub th32ModuleID: DWORD,
    pub cntThreads: DWORD,
    pub th32ParentProcessID: DWORD,
    pub pcPriClassBase: i32,
    pub dwFlags: DWORD,
    pub szExeFile: [WCHAR; 260],
}

#[repr(C)]
pub struct MODULEENTRY32W {
    pub dwSize: DWORD,
    pub th32ModuleID: DWORD,
    pub th32ProcessID: DWORD,
    pub GlblcntUsage: DWORD,
    pub ProccntUsage: DWORD,
    pub modBaseAddr: *mut u8,
    pub modBaseSize: DWORD,
    pub hModule: HANDLE,
    pub szModule: [WCHAR; 256],
    pub szExePath: [WCHAR; 260],
}

#[repr(C)]
pub struct MEMORY_BASIC_INFORMATION {
    pub BaseAddress: *mut c_void,
    pub AllocationBase: *mut c_void,
    pub AllocationProtect: DWORD,
    pub PartitionId: WORD,
    pub RegionSize: SIZE_T,
    pub State: DWORD,
    pub Protect: DWORD,
    pub Type: DWORD,
}

#[link(name = "kernel32")]
extern "system" {
    pub fn OpenProcess(dwDesiredAccess: DWORD, bInheritHandle: BOOL, dwProcessId: DWORD) -> HANDLE;
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn CreateToolhelp32Snapshot(dwFlags: DWORD, th32ProcessID: DWORD) -> HANDLE;
    pub fn Process32FirstW(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32W) -> BOOL;
    pub fn Process32NextW(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32W) -> BOOL;
    pub fn Module32FirstW(hSnapshot: HANDLE, lpme: *mut MODULEENTRY32W) -> BOOL;
    pub fn Module32NextW(hSnapshot: HANDLE, lpme: *mut MODULEENTRY32W) -> BOOL;
    pub fn VirtualQueryEx(
        hProcess: HANDLE,
        lpAddress: *const c_void,
        lpBuffer: *mut MEMORY_BASIC_INFORMATION,
        dwLength: SIZE_T,
    ) -> SIZE_T;
    pub fn ReadProcessMemory(
        hProcess: HANDLE,
        lpBaseAddress: *const c_void,
        lpBuffer: *mut c_void,
        nSize: SIZE_T,
        lpNumberOfBytesRead: *mut SIZE_T,
    ) -> BOOL;
    pub fn WriteProcessMemory(
        hProcess: HANDLE,
        lpBaseAddress: *mut c_void,
        lpBuffer: *const c_void,
        nSize: SIZE_T,
        lpNumberOfBytesWritten: *mut SIZE_T,
    ) -> BOOL;
    pub fn GetLastError() -> DWORD;
}

pub struct AutoCloseHandle(pub HANDLE);
impl Drop for AutoCloseHandle {
    fn drop(&mut self) {
        if !self.0.is_null() && self.0 != INVALID_HANDLE_VALUE {
            unsafe { CloseHandle(self.0) };
        }
    }
}

fn wchar_to_string(slice: &[u16]) -> String {
    let len = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
    String::from_utf16_lossy(&slice[..len])
}

pub fn is_pid_alive(pid: u64) -> bool {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid as DWORD);
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            return false;
        }
        CloseHandle(handle);
        true
    }
}

pub fn list_processes(filter: Option<&str>) -> Result<Vec<String>, String> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(format!("failed to create process snapshot: error code {}", GetLastError()));
        }
        let _guard = AutoCloseHandle(snapshot);

        let mut entry = std::mem::zeroed::<PROCESSENTRY32W>();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as DWORD;

        let mut results = Vec::new();
        let filter = filter.unwrap_or("").to_lowercase();

        if Process32FirstW(snapshot, &mut entry) != FALSE {
            loop {
                let name = wchar_to_string(&entry.szExeFile);
                let pid = entry.th32ProcessID;
                if pid > 0 {
                    let line = format!("{:>7} {}", pid, name);
                    if filter.is_empty() || name.to_lowercase().contains(&filter) || pid.to_string().contains(&filter) {
                        results.push(line);
                    }
                }
                if results.len() >= 100 {
                    break;
                }
                if Process32NextW(snapshot, &mut entry) == FALSE {
                    break;
                }
            }
        }
        Ok(results)
    }
}

pub fn get_process_info(pid: u64) -> Result<Value, String> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(format!("failed to inspect process: error code {}", GetLastError()));
        }
        let _guard = AutoCloseHandle(snapshot);

        let mut entry = std::mem::zeroed::<PROCESSENTRY32W>();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as DWORD;

        if Process32FirstW(snapshot, &mut entry) != FALSE {
            loop {
                if entry.th32ProcessID == pid as DWORD {
                    let name = wchar_to_string(&entry.szExeFile);
                    return Ok(json!({
                        "pid": pid,
                        "name": name,
                        "parent_pid": entry.th32ParentProcessID,
                        "threads": entry.cntThreads,
                        "state": "running (Windows Win32)",
                        "platform": "windows"
                    }));
                }
                if Process32NextW(snapshot, &mut entry) == FALSE {
                    break;
                }
            }
        }
        Err(format!("process with pid {pid} not found"))
    }
}

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub name: String,
    pub base: u64,
    pub size: u64,
    pub path: String,
}

pub fn list_modules(pid: u64) -> Result<Vec<ModuleInfo>, String> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid as DWORD);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(format!("failed to snapshot modules for pid {pid}: error code {}", GetLastError()));
        }
        let _guard = AutoCloseHandle(snapshot);

        let mut entry = std::mem::zeroed::<MODULEENTRY32W>();
        entry.dwSize = std::mem::size_of::<MODULEENTRY32W>() as DWORD;

        let mut modules = Vec::new();
        if Module32FirstW(snapshot, &mut entry) != FALSE {
            loop {
                let name = wchar_to_string(&entry.szModule);
                let path = wchar_to_string(&entry.szExePath);
                let base = entry.modBaseAddr as u64;
                let size = entry.modBaseSize as u64;

                modules.push(ModuleInfo {
                    name,
                    base,
                    size,
                    path,
                });

                if Module32NextW(snapshot, &mut entry) == FALSE {
                    break;
                }
            }
        }
        Ok(modules)
    }
}

pub fn get_module_base(pid: u64, module_name: &str) -> Result<Option<u64>, String> {
    let mods = list_modules(pid)?;
    let target = module_name.to_lowercase();
    for m in mods {
        if m.name.to_lowercase() == target || m.name.to_lowercase().contains(&target) {
            return Ok(Some(m.base));
        }
    }
    Ok(None)
}

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub start: u64,
    pub end: u64,
    pub perms: String,
    pub size: u64,
}

fn protect_to_perms(protect: DWORD) -> &'static str {
    let base = protect & 0xFF;
    match base {
        PAGE_READONLY => "r--",
        PAGE_READWRITE => "rw-",
        PAGE_WRITECOPY => "rw-",
        PAGE_EXECUTE => "--x",
        PAGE_EXECUTE_READ => "r-x",
        PAGE_EXECUTE_READWRITE => "rwx",
        PAGE_EXECUTE_WRITECOPY => "rwx",
        _ => "---",
    }
}

fn is_writable(protect: DWORD) -> bool {
    let base = protect & 0xFF;
    base == PAGE_READWRITE || base == PAGE_WRITECOPY || base == PAGE_EXECUTE_READWRITE || base == PAGE_EXECUTE_WRITECOPY
}

fn is_readable(protect: DWORD) -> bool {
    let base = protect & 0xFF;
    base != PAGE_NOACCESS && (protect & PAGE_GUARD) == 0
}

pub fn query_memory_regions(pid: u64, writable_only: bool) -> Result<Vec<MemoryRegion>, String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, FALSE, pid as DWORD);
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            return Err(format!("failed to open process {pid}: error code {}", GetLastError()));
        }
        let _guard = AutoCloseHandle(handle);

        let mut regions = Vec::new();
        let mut mbi = std::mem::zeroed::<MEMORY_BASIC_INFORMATION>();
        let mut addr: usize = 0;
        let limit: usize = 0x7FFF_FFFF_0000;

        while addr < limit {
            let ret = VirtualQueryEx(
                handle,
                addr as *const c_void,
                &mut mbi,
                std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
            );
            if ret == 0 {
                break;
            }

            let start = mbi.BaseAddress as u64;
            let size = mbi.RegionSize as u64;
            let end = start.saturating_add(size);

            if mbi.State == MEM_COMMIT {
                let readable = is_readable(mbi.Protect);
                let writable = is_writable(mbi.Protect);
                if readable && (!writable_only || writable) {
                    regions.push(MemoryRegion {
                        start,
                        end,
                        perms: protect_to_perms(mbi.Protect).to_string(),
                        size,
                    });
                }
            }

            if size == 0 {
                break;
            }
            match (addr as u64).checked_add(size) {
                Some(next) => addr = next as usize,
                None => break,
            }
        }

        Ok(regions)
    }
}

pub fn read_process_memory(pid: u64, address: u64, len: usize) -> Result<Vec<u8>, String> {
    unsafe {
        let handle = OpenProcess(PROCESS_VM_READ | PROCESS_QUERY_INFORMATION, FALSE, pid as DWORD);
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            return Err(format!("failed to open process {pid} for memory read: error code {}", GetLastError()));
        }
        let _guard = AutoCloseHandle(handle);

        let mut buffer = vec![0u8; len];
        let mut bytes_read: SIZE_T = 0;
        let success = ReadProcessMemory(
            handle,
            address as *const c_void,
            buffer.as_mut_ptr() as *mut c_void,
            len,
            &mut bytes_read,
        );

        if success == FALSE || bytes_read == 0 {
            return Err(format!("failed to read memory at 0x{:08x} (len={len}): error code {}", address, GetLastError()));
        }
        buffer.truncate(bytes_read);
        Ok(buffer)
    }
}

pub fn write_process_memory(pid: u64, address: u64, data: &[u8]) -> Result<usize, String> {
    unsafe {
        let handle = OpenProcess(PROCESS_VM_WRITE | PROCESS_VM_OPERATION | PROCESS_QUERY_INFORMATION, FALSE, pid as DWORD);
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            return Err(format!("failed to open process {pid} for memory write: error code {}", GetLastError()));
        }
        let _guard = AutoCloseHandle(handle);

        let mut bytes_written: SIZE_T = 0;
        let success = WriteProcessMemory(
            handle,
            address as *mut c_void,
            data.as_ptr() as *const c_void,
            data.len(),
            &mut bytes_written,
        );

        if success == FALSE {
            return Err(format!("failed to write memory at 0x{:08x}: error code {}", address, GetLastError()));
        }
        Ok(bytes_written)
    }
}

pub fn scan_process_memory_exact(
    pid: u64,
    scan_type: ScanType,
    target: &ParsedValue,
) -> Result<Vec<ScanMatch>, String> {
    let regions = query_memory_regions(pid, true)?;
    let mut matches = Vec::new();
    let chunk_size = 64 * 1024; // 64KB chunks

    unsafe {
        let handle = OpenProcess(PROCESS_VM_READ | PROCESS_QUERY_INFORMATION, FALSE, pid as DWORD);
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            return Err(format!("failed to open process {pid}: error code {}", GetLastError()));
        }
        let _guard = AutoCloseHandle(handle);

        let mut chunk = vec![0u8; chunk_size];

        for region in regions {
            if matches.len() >= native_scan::MAX_MATCHES_SAVED {
                break;
            }
            let mut curr = region.start;
            while curr < region.end {
                let to_read = ((region.end - curr) as usize).min(chunk_size);
                let mut bytes_read: SIZE_T = 0;
                let success = ReadProcessMemory(
                    handle,
                    curr as *const c_void,
                    chunk.as_mut_ptr() as *mut c_void,
                    to_read,
                    &mut bytes_read,
                );
                if success != FALSE && bytes_read >= scan_type.size() {
                    native_scan::scan_buffer_exact(
                        curr,
                        &chunk[..bytes_read],
                        scan_type,
                        target,
                        scan_type.alignment(),
                        &mut matches,
                    );
                }
                curr = curr.saturating_add(to_read as u64);
                if matches.len() >= native_scan::MAX_MATCHES_SAVED {
                    break;
                }
            }
        }
    }

    Ok(matches)
}

pub fn scan_process_memory_range(
    pid: u64,
    scan_type: ScanType,
    low: &ParsedValue,
    high: &ParsedValue,
) -> Result<Vec<ScanMatch>, String> {
    let regions = query_memory_regions(pid, true)?;
    let mut matches = Vec::new();
    let chunk_size = 64 * 1024;

    unsafe {
        let handle = OpenProcess(PROCESS_VM_READ | PROCESS_QUERY_INFORMATION, FALSE, pid as DWORD);
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            return Err(format!("failed to open process {pid}: error code {}", GetLastError()));
        }
        let _guard = AutoCloseHandle(handle);

        let mut chunk = vec![0u8; chunk_size];

        for region in regions {
            if matches.len() >= native_scan::MAX_MATCHES_SAVED {
                break;
            }
            let mut curr = region.start;
            while curr < region.end {
                let to_read = ((region.end - curr) as usize).min(chunk_size);
                let mut bytes_read: SIZE_T = 0;
                let success = ReadProcessMemory(
                    handle,
                    curr as *const c_void,
                    chunk.as_mut_ptr() as *mut c_void,
                    to_read,
                    &mut bytes_read,
                );
                if success != FALSE && bytes_read >= scan_type.size() {
                    native_scan::scan_buffer_range(
                        curr,
                        &chunk[..bytes_read],
                        scan_type,
                        low,
                        high,
                        scan_type.alignment(),
                        &mut matches,
                    );
                }
                curr = curr.saturating_add(to_read as u64);
                if matches.len() >= native_scan::MAX_MATCHES_SAVED {
                    break;
                }
            }
        }
    }

    Ok(matches)
}

pub fn scan_process_memory_unknown(
    pid: u64,
    scan_type: ScanType,
) -> Result<Vec<ScanMatch>, String> {
    let regions = query_memory_regions(pid, true)?;
    let mut matches = Vec::new();
    let chunk_size = 64 * 1024;

    unsafe {
        let handle = OpenProcess(PROCESS_VM_READ | PROCESS_QUERY_INFORMATION, FALSE, pid as DWORD);
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            return Err(format!("failed to open process {pid}: error code {}", GetLastError()));
        }
        let _guard = AutoCloseHandle(handle);

        let mut chunk = vec![0u8; chunk_size];

        for region in regions {
            if matches.len() >= native_scan::MAX_MATCHES_SAVED {
                break;
            }
            let mut curr = region.start;
            while curr < region.end {
                let to_read = ((region.end - curr) as usize).min(chunk_size);
                let mut bytes_read: SIZE_T = 0;
                let success = ReadProcessMemory(
                    handle,
                    curr as *const c_void,
                    chunk.as_mut_ptr() as *mut c_void,
                    to_read,
                    &mut bytes_read,
                );
                if success != FALSE && bytes_read >= scan_type.size() {
                    native_scan::scan_buffer_any(
                        curr,
                        &chunk[..bytes_read],
                        scan_type,
                        scan_type.alignment(),
                        &mut matches,
                    );
                }
                curr = curr.saturating_add(to_read as u64);
                if matches.len() >= native_scan::MAX_MATCHES_SAVED {
                    break;
                }
            }
        }
    }

    Ok(matches)
}

pub fn refine_scan_matches(
    pid: u64,
    matches: &mut Vec<ScanMatch>,
    scan_type: ScanType,
    op: &str,
    target: Option<&ParsedValue>,
) -> Result<usize, String> {
    unsafe {
        let handle = OpenProcess(PROCESS_VM_READ | PROCESS_QUERY_INFORMATION, FALSE, pid as DWORD);
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            return Err(format!("failed to open process {pid}: error code {}", GetLastError()));
        }
        let _guard = AutoCloseHandle(handle);

        let size = scan_type.size();
        let mut buf = vec![0u8; size];

        matches.retain_mut(|m| {
            let mut bytes_read: SIZE_T = 0;
            let ok = ReadProcessMemory(
                handle,
                m.address as *const c_void,
                buf.as_mut_ptr() as *mut c_void,
                size,
                &mut bytes_read,
            );
            if ok == FALSE || bytes_read < size {
                return false;
            }
            if let Some(curr_val) = ParsedValue::from_bytes(&buf, scan_type) {
                let keep = match op {
                    "+" | ">" => curr_val.is_greater_than(&m.value),
                    "-" | "<" => curr_val.is_less_than(&m.value),
                    "!=" | "changed" => curr_val.is_changed(&m.value),
                    "=" | "unchanged" => curr_val.is_unchanged(&m.value),
                    "exact" => {
                        if let Some(t) = target {
                            curr_val.matches_exact(t)
                        } else {
                            false
                        }
                    }
                    _ => false,
                };
                if keep {
                    m.value = curr_val;
                    return true;
                }
            }
            false
        });

        Ok(matches.len())
    }
}
