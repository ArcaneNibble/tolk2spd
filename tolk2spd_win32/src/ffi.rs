use core::fmt;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr;
use std::sync::atomic::{AtomicU64, Ordering};

use windows_sys::Wdk::Storage::FileSystem::NtQueryVirtualMemory;
use windows_sys::Win32::Foundation::HINSTANCE;
use windows_sys::Win32::System::LibraryLoader::DisableThreadLibraryCalls;
use windows_sys::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::core::BOOL;

unsafe extern "C" {
    static __ImageBase: std::ffi::c_void;
}
#[allow(non_upper_case_globals)]
static __wine_unixlib_handle: AtomicU64 = AtomicU64::new(0);
#[link(name = "ntdll", kind = "raw-dylib")]
unsafe extern "system" {
    pub static __wine_unix_call_dispatcher:
        unsafe extern "system" fn(unixlib_handle: u64, code: u32, ptr: *const c_void) -> u32;
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "system" fn DllMain(hinstDLL: HINSTANCE, fdwReason: u32, _: *mut ()) -> BOOL {
    if fdwReason != DLL_PROCESS_ATTACH {
        return 1;
    }

    unsafe {
        DisableThreadLibraryCalls(hinstDLL);
    }

    return 1;
}

pub(crate) fn load_unixlib() -> bool {
    if __wine_unixlib_handle.load(Ordering::Relaxed) == 0 {
        unsafe {
            let img_base = &raw const __ImageBase;

            let mut wine_unixlib_handle: u64 = 0;
            let ret = NtQueryVirtualMemory(
                GetCurrentProcess(),
                img_base,
                1000,
                &raw mut wine_unixlib_handle as *mut c_void,
                std::mem::size_of_val(&wine_unixlib_handle),
                std::ptr::null_mut(),
            );
            if ret != 0 {
                return false;
            }

            eprintln!("__wine_unixlib_handle = 0x{wine_unixlib_handle:016x}");
            __wine_unixlib_handle.store(wine_unixlib_handle, Ordering::Relaxed);

            let unixlib_ver = get_version();
            if unixlib_ver != tolk2spd_abi::ABI_VERSION {
                eprintln!(
                    "ERROR: tolk2spd ABI mismatch, got {} expected {}",
                    unixlib_ver,
                    tolk2spd_abi::ABI_VERSION
                );
                __wine_unixlib_handle.store(0, Ordering::Relaxed);
                return false;
            }
        }
    }

    true
}

pub fn get_version() -> u32 {
    unsafe {
        __wine_unix_call_dispatcher(
            __wine_unixlib_handle.load(Ordering::Relaxed),
            tolk2spd_abi::Syscalls::GetVersion as u32,
            ptr::null(),
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ConnectionHandle(pub u64, PhantomData<*mut ()>);
impl fmt::Debug for ConnectionHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}

pub fn connect() -> Option<ConnectionHandle> {
    unsafe {
        let mut args = tolk2spd_abi::ArgsConnect { out_connection: 0 };

        let ret = __wine_unix_call_dispatcher(
            __wine_unixlib_handle.load(Ordering::Relaxed),
            tolk2spd_abi::Syscalls::Connect as u32,
            &mut args as *mut tolk2spd_abi::ArgsConnect as *const c_void,
        );
        if ret != 0 {
            return None;
        }

        Some(ConnectionHandle(args.out_connection, PhantomData))
    }
}

pub unsafe fn disconnect(conn: ConnectionHandle) {
    unsafe {
        let mut args = tolk2spd_abi::ArgsDisconnect {
            in_connection: conn.0,
        };

        __wine_unix_call_dispatcher(
            __wine_unixlib_handle.load(Ordering::Relaxed),
            tolk2spd_abi::Syscalls::Disconnect as u32,
            &mut args as *mut tolk2spd_abi::ArgsDisconnect as *const c_void,
        );
    }
}
