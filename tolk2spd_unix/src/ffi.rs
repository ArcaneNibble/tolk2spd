use std::ffi::c_void;

#[repr(transparent)]
pub struct WineUnixlibFnPtr(pub extern "C" fn(*const c_void) -> u32);
unsafe impl Sync for WineUnixlibFnPtr {}
