use std::ffi::c_void;

mod ffi;

extern "C" fn get_version(_arg: *const c_void) -> u32 {
    tolk2spd_abi::ABI_VERSION
}

#[unsafe(no_mangle)]
static __wine_unix_call_funcs: [ffi::WineUnixlibFnPtr; 1] = [ffi::WineUnixlibFnPtr(get_version)];
