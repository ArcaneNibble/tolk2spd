fn main() {
    println!("Hello, world!");

    let our_dll = unsafe {
        windows_sys::Win32::System::LibraryLoader::LoadLibraryA(b"lib\\Tolk.dll\x00".as_ptr())
    };
    dbg!(our_dll);
    assert!(!our_dll.is_null(), "failed to load unixlib");
}
