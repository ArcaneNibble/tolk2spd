#![no_std]

use core::slice;

pub const ABI_VERSION: u32 = 0x00000001;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct FFIStringBorrow {
    ptr: *const u8,
    len: u64,
}
impl From<&str> for FFIStringBorrow {
    fn from(value: &str) -> Self {
        Self {
            ptr: value.as_ptr(),
            len: value.len() as u64,
        }
    }
}
impl From<&FFIStringBorrow> for &str {
    fn from(value: &FFIStringBorrow) -> Self {
        unsafe { str::from_utf8_unchecked(slice::from_raw_parts(value.ptr, value.len as usize)) }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct ArgsConnect {
    pub in_exename: FFIStringBorrow,

    pub out_connection: u64,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct ArgsDisconnect {
    pub in_connection: u64,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct ArgsSpeak {
    pub in_connection: u64,
    pub in_msg: FFIStringBorrow,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct ArgsStopSpeaking {
    pub in_connection: u64,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[repr(u32)]
pub enum Syscalls {
    GetVersion,
    Connect,
    Disconnect,
    Speak,
    StopSpeaking,
}
