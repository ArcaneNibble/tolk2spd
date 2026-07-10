use std::ffi::c_void;
use std::sync::atomic::AtomicU64;

use windows_sys::Win32::Foundation::HINSTANCE;
use windows_sys::Win32::System::LibraryLoader::DisableThreadLibraryCalls;
use windows_sys::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows_sys::core::BOOL;

unsafe extern "C" {
    pub(crate) static __ImageBase: std::ffi::c_void;
}
#[allow(non_upper_case_globals)]
pub(crate) static __wine_unixlib_handle: AtomicU64 = AtomicU64::new(0);
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
