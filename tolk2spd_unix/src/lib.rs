use std::ffi::c_void;

mod ffi;

extern "C" fn get_version(_arg: *const c_void) -> u32 {
    tolk2spd_abi::ABI_VERSION
}

#[derive(Debug)]
pub struct SPDConnection {
    _test: u32,
}

impl SPDConnection {
    pub fn new() -> Self {
        dbg!("make conn");

        Self { _test: 12345 }
    }
}

impl Drop for SPDConnection {
    fn drop(&mut self) {
        dbg!("drop conn!", self._test);
    }
}
