use std::{ffi::c_void, ptr};

use crate::SPDConnection;

#[repr(transparent)]
struct WineUnixlibFnPtr(pub extern "C" fn(*const c_void) -> u32);
unsafe impl Sync for WineUnixlibFnPtr {}

extern "C" fn bridge_connect(arg: *const c_void) -> u32 {
    unsafe {
        let arg = arg as *mut tolk2spd_abi::ArgsConnect;

        let conn = Box::new(SPDConnection::new());
        let conn = Box::into_raw(conn);
        (*arg).out_connection = conn.expose_provenance() as u64;

        0
    }
}

extern "C" fn bridge_disconnect(arg: *const c_void) -> u32 {
    unsafe {
        let arg = arg as *mut tolk2spd_abi::ArgsDisconnect;

        let conn = (*arg).in_connection as usize;
        let conn: *mut SPDConnection = ptr::with_exposed_provenance_mut(conn);
        let _conn = Box::from_raw(conn);

        0
    }
}

#[unsafe(no_mangle)]
static __wine_unix_call_funcs: [WineUnixlibFnPtr; 3] = [
    WineUnixlibFnPtr(crate::get_version),
    WineUnixlibFnPtr(bridge_connect),
    WineUnixlibFnPtr(bridge_disconnect),
];
