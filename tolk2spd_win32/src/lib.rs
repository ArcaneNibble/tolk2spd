use windows_sys::Win32::Foundation::HINSTANCE;
use windows_sys::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows_sys::core::BOOL;

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "system" fn DllMain(hinstDLL: HINSTANCE, fdwReason: u32, _: *mut ()) -> BOOL {
    if fdwReason != DLL_PROCESS_ATTACH {
        return 1;
    }

    dbg!("tolk2spd DllMain!");

    // unsafe {
    //     DisableThreadLibraryCalls(hinstDLL);

    //     let img_base = &raw const __ImageBase;
    //     dbg!(img_base);

    //     let mut wine_unixlib_handle: u64 = 0;
    //     let ret = NtQueryVirtualMemory(
    //         GetCurrentProcess(),
    //         img_base,
    //         1000,
    //         &raw mut wine_unixlib_handle as *mut c_void,
    //         std::mem::size_of_val(&wine_unixlib_handle),
    //         std::ptr::null_mut(),
    //     );

    //     if ret != 0 {
    //         return 0;
    //     }
    //     println!("__wine_unixlib_handle = 0x{wine_unixlib_handle:016x}");
    //     __wine_unixlib_handle.store(wine_unixlib_handle, Ordering::Relaxed);

    //     dbg!(__wine_unix_call_dispatcher);
    // }

    return 1;
}
