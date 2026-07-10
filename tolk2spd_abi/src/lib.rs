#![no_std]

pub const ABI_VERSION: u32 = 0x00000001;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct ArgsConnect {
    pub out_connection: u64,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct ArgsDisconnect {
    pub in_connection: u64,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[repr(u32)]
pub enum Syscalls {
    GetVersion,
    Connect,
    Disconnect,
}
