use std::ffi::c_void;

mod ffi;

extern "C" fn testfunc(_arg: *const c_void) -> u32 {
    dbg!("Called testfunc!");
    12345
}

#[unsafe(no_mangle)]
static __wine_unix_call_funcs: [ffi::WineUnixlibFnPtr; 1] = [ffi::WineUnixlibFnPtr(testfunc)];
