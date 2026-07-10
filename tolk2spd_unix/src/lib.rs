mod ffi;

#[unsafe(no_mangle)]
static __wine_unix_call_funcs: [ffi::WineUnixlibFnPtr; 0] = [];
