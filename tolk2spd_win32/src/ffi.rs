//! FFI glue for Wine unixlib

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
    /// Use the linker to get our [HMODULE](windows_sys::Win32::Foundation::HMODULE)
    ///
    /// This is used in lieu of saving the `HMODULE` in `DllMain`
    static __ImageBase: std::ffi::c_void;
}
/// Handle to the *nix `.so` side of our library
#[allow(non_upper_case_globals)]
static __wine_unixlib_handle: AtomicU64 = AtomicU64::new(0);
#[link(name = "ntdll", kind = "raw-dylib")]
unsafe extern "system" {
    /// Wine special NTDLL export for calling across the boundary
    ///
    /// Note that this isn't a function, it's a function _pointer_.
    /// We have declared it to Rust s.t. Rust understands this.
    pub static __wine_unix_call_dispatcher:
        unsafe extern "system" fn(unixlib_handle: u64, code: u32, ptr: *const c_void) -> u32;
}

/// The DLL entry point
///
/// For efficiency, ask to not get thread messages we don't care about
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

/// Attempt to load the *nix `.so` library
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

/// Calls [tolk2spd_abi::Syscalls::GetVersion]
pub fn get_version() -> u32 {
    unsafe {
        __wine_unix_call_dispatcher(
            __wine_unixlib_handle.load(Ordering::Relaxed),
            tolk2spd_abi::Syscalls::GetVersion as u32,
            ptr::null(),
        )
    }
}

/// Wrap a connection object/handle
///
/// This cleans up the debug printing and makes sure it has the correct variance in Rust.
/// This isn't super _useful_, but it may help in case this code gets reused further.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ConnectionHandle(pub u64, PhantomData<*mut ()>);
impl fmt::Debug for ConnectionHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}
impl From<u64> for ConnectionHandle {
    fn from(value: u64) -> Self {
        Self(value, PhantomData)
    }
}

/// Calls [tolk2spd_abi::Syscalls::Connect]
pub fn connect() -> Option<ConnectionHandle> {
    // Get the EXE name, specifically from the Win32 side
    // (it's not useful to have the *nix side return that the executable is the preloader)
    let exe_pathbuf;
    let exe = if let Ok(exe) = std::env::current_exe() {
        exe_pathbuf = exe;
        if let Some(exe) = exe_pathbuf.file_stem() {
            &exe.to_string_lossy()
        } else {
            "tolk2spd"
        }
    } else {
        "tolk2spd"
    };

    unsafe {
        let mut args = tolk2spd_abi::ArgsConnect {
            in_exename: exe.into(),
            out_connection: 0,
        };

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

/// Calls [tolk2spd_abi::Syscalls::Disconnect]
///
/// Unsafe because the `ConnectionHandle` does expose the raw integer value.
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

/// Calls [tolk2spd_abi::Syscalls::Speak]
///
/// Unsafe because the `ConnectionHandle` does expose the raw integer value.
pub unsafe fn speak(conn: ConnectionHandle, msg: &str) -> bool {
    unsafe {
        let mut args = tolk2spd_abi::ArgsSpeak {
            in_connection: conn.0,
            in_msg: msg.into(),
        };

        let ret = __wine_unix_call_dispatcher(
            __wine_unixlib_handle.load(Ordering::Relaxed),
            tolk2spd_abi::Syscalls::Speak as u32,
            &mut args as *mut tolk2spd_abi::ArgsSpeak as *const c_void,
        );
        if ret != 0 {
            return false;
        }

        true
    }
}

/// Calls [tolk2spd_abi::Syscalls::StopSpeaking]
///
/// Unsafe because the `ConnectionHandle` does expose the raw integer value.
pub unsafe fn stop_speaking(conn: ConnectionHandle) -> bool {
    unsafe {
        let mut args = tolk2spd_abi::ArgsStopSpeaking {
            in_connection: conn.0,
        };

        let ret = __wine_unix_call_dispatcher(
            __wine_unixlib_handle.load(Ordering::Relaxed),
            tolk2spd_abi::Syscalls::StopSpeaking as u32,
            &mut args as *mut tolk2spd_abi::ArgsStopSpeaking as *const c_void,
        );
        if ret != 0 {
            return false;
        }

        true
    }
}
