use std::ffi::CStr;

use windows_strings::PCWSTR;
use windows_sys::Win32::Foundation::HMODULE;
use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};

#[inline]
fn gpa<F>(dll: HMODULE, name: &CStr) -> F {
    unsafe {
        let fn_ = GetProcAddress(dll, name.as_ptr() as *const u8).expect("failed to load import");
        std::mem::transmute_copy(&fn_)
    }
}

#[allow(non_snake_case)]
fn main() {
    println!("Hello, world!");

    let our_dll = unsafe { LoadLibraryA(b"lib\\Tolk.dll\x00".as_ptr()) };
    dbg!(our_dll);
    assert!(!our_dll.is_null(), "failed to load unixlib");

    let Tolk_Load: extern "C" fn() = gpa(our_dll, c"Tolk_Load");
    let Tolk_DetectScreenReader: extern "C" fn() -> *const u16 =
        gpa(our_dll, c"Tolk_DetectScreenReader");
    // dbg!(Tolk_Load, Tolk_DetectScreenReader);

    Tolk_Load();
    let sr = PCWSTR::from_raw(Tolk_DetectScreenReader());
    unsafe {
        eprintln!("Screen reader: {}", sr.display());
    }
}
