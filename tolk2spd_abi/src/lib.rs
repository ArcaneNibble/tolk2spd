//! ABI information for interfacing between the Wine/Windows world and *nix
//!
//! This crate contains only type definitions and associated helpers.
//!
//! The _same_ code must work correctly between Windows and *nix.
//! (This is generally easier than if it were written in C, since
//! Rust doesn't have LP64 vs ILP64 footguns.)
//!
//! The _same_ code also must work correctly between 32-bit and 64-bit,
//! and more care is needed here. In general, we deal with this by
//! always treating pointers as 64-bits, zero-extending where relevant.

#![no_std]

use core::{ptr, slice};

/// ABI version number, to help check for mismatched files
///
/// This needs to be _manually_ bumped when incompatible changes happen.
/// The ABI stability policy (forwards vs backwards compat) is TBD.
/// There is also no way to automatically test this,
/// so it currently depends on human domain expertise.
pub const ABI_VERSION: u32 = 0x00000001;

/// Represents a borrowed string with transient validity
///
/// Validity is roughly "for the current call".
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct FFIStringBorrow {
    ptr: u64,
    len: u64,
}
impl From<&str> for FFIStringBorrow {
    fn from(value: &str) -> Self {
        Self {
            ptr: value.as_ptr().expose_provenance() as u64,
            len: value.len() as u64,
        }
    }
}
impl From<&FFIStringBorrow> for &str {
    fn from(value: &FFIStringBorrow) -> Self {
        let ptr: *const u8 = ptr::with_exposed_provenance(value.ptr as usize);
        // SAFETY: Because fields are private, this struct can only be constructed
        // by going through the above conversion implementation.
        //
        // This is only really safe for the 32-bit --> 64-bit direction
        // (the reverse needs a special memory allocator), but this can't easily be checked.
        unsafe { str::from_utf8_unchecked(slice::from_raw_parts(ptr, value.len as usize)) }
    }
}

/// The arguments for [Syscalls::Connect]
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct ArgsConnect {
    /// The name of the current executable
    ///
    /// This is passed in to SSIP as part of the CLIENT_NAME
    pub in_exename: FFIStringBorrow,

    /// The returned connection handle
    ///
    /// This has `&mut` semantics
    pub out_connection: u64,
}

/// The arguments for [Syscalls::Disconnect]
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct ArgsDisconnect {
    /// The input connection handle
    ///
    /// This consumes ownership
    pub in_connection: u64,
}

/// The arguments for [Syscalls::Speak]
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct ArgsSpeak {
    /// The input connection handle
    ///
    /// This has `&mut` semantics
    pub in_connection: u64,
    /// The input message to speak
    pub in_msg: FFIStringBorrow,
}

/// The arguments for [Syscalls::StopSpeaking]
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct ArgsStopSpeaking {
    /// The input connection handle
    ///
    /// This has `&mut` semantics
    pub in_connection: u64,
}

/// unixlib interface function indices
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[repr(u32)]
#[non_exhaustive]
pub enum Syscalls {
    GetVersion,
    Connect,
    Disconnect,
    Speak,
    StopSpeaking,
}
