//! Tolk to Speech Dispatcher Win32 code
//!
//! This is a Wine "builtin" DLL making use of "unixlib" functionality.
//! The DLL's internal name (in its exports section) controls the
//! name of the .so file that Wine will load, so the Rust _crate_ name
//! must be `tolk` in order for Wine to look for `tolk.so`.
//!
//! This DLL should export the same interface as the "real" Tolk library.
//! None of the functions are visible to Rust, because it's not intended
//! to be consumed _by Rust code_, only by _C_ code.

use std::ptr;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};

use windows_strings::w;

mod ffi;

/// A read-only string which is allocated only once, inside a [LazyLock], and never freed.
struct LeakedPCWSTR(*const u16);
unsafe impl Send for LeakedPCWSTR {}
unsafe impl Sync for LeakedPCWSTR {}

/// What name should we report ourselves as?
///
/// Normally, we are honest and report "Speech Dispatcher (Wine)".
/// However, in case it's necessary, the `TOLK2SPD_SPOOF` environment variable
/// can override this in order to emulate e.g. NVDA.
static EMULATED_SCREENREADER_NAME: LazyLock<LeakedPCWSTR> =
    LazyLock::new(|| match std::env::var_os("TOLK2SPD_SPOOF") {
        Some(spoof) => {
            let spoof = spoof.to_string_lossy();
            let mut spoof_u16 = spoof.encode_utf16().collect::<Vec<_>>();
            // Add a null terminator
            spoof_u16.push(0);
            LeakedPCWSTR(spoof_u16.into_raw_parts().0)
        }
        None => LeakedPCWSTR(w!("Speech Dispatcher (Wine)").0),
    });
/// Represents the active connection handle.
///
/// There is only one, and Tolk isn't thread-safe.
///
/// A null value (0) is never valid (implicit ABI contract)
/// and represents a closed/failed connection.
static CONNECTION: AtomicU64 = AtomicU64::new(0);

/// Attempt to connect to the server
///
/// Also loads the unixlib .so if we haven't already
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Load() {
    eprintln!("Tolk_Load");

    // Load the Wine unixlib, if it hasn't already
    if !ffi::load_unixlib() {
        return;
    }

    // If we're already connected, don't try to connect again
    if Tolk_IsLoaded() {
        return;
    }

    // Try to establish a connection
    if let Some(conn) = ffi::connect() {
        dbg!(conn);
        // This is set to non-null only if a connection is properly established
        CONNECTION.store(conn.0, Ordering::Relaxed);
    } else {
        return;
    }

    // Force the emulated screenreader name to be initialized now
    let _ = *EMULATED_SCREENREADER_NAME;
}

/// Check if we have an active connection
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_IsLoaded() -> bool {
    CONNECTION.load(Ordering::Relaxed) != 0
}

/// Disconnect from the server
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Unload() {
    eprintln!("Tolk_Unload");

    if !Tolk_IsLoaded() {
        return;
    }

    // Close the connection
    // SAFETY: We must have one open and active connection
    // SAFETY: We make sure to null out the pointer to consume and free it
    unsafe {
        let conn = CONNECTION.swap(0, Ordering::Relaxed);
        let conn = ffi::ConnectionHandle::from(conn);
        ffi::disconnect(conn);
    }

    // We can't actually unload the unixlib
}

/// Dummy function
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_TrySAPI(try_sapi: bool) {
    eprintln!("Tolk_TrySAPI {}", try_sapi);
}

/// Dummy function
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_PreferSAPI(prefer_sapi: bool) {
    eprintln!("Tolk_PreferSAPI {}", prefer_sapi);
}

/// Returns if we're connected, and possibly spoof another software
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_DetectScreenReader() -> *const u16 {
    eprintln!("Tolk_DetectScreenReader");
    if !Tolk_IsLoaded() {
        return ptr::null();
    }
    EMULATED_SCREENREADER_NAME.0
}

/// Dummy function
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_HasSpeech() -> bool {
    if !Tolk_IsLoaded() {
        return false;
    }
    // We support _only_ speech
    true
}

/// Dummy function
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_HasBraille() -> bool {
    // We do not support braille
    false
}

/// Speak some message
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Output(str_: *const u16, interrupt: bool) -> bool {
    // Since we only support speech, redirect
    Tolk_Speak(str_, interrupt)
}

/// Speak some message
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Speak(str_: *const u16, interrupt: bool) -> bool {
    if !Tolk_IsLoaded() {
        return false;
    }
    if str_.is_null() {
        return false;
    }

    // Convert to something which doesn't suck
    // SAFETY: str_ is valid (non-null), and the caller is _required_
    // to pass in a null-terminated UTF-16 string
    let str_ = windows_strings::PCWSTR(str_);
    let str_ = unsafe {
        match str_.to_string() {
            Ok(s) => s,
            Err(e) => {
                dbg!(e);
                return false;
            }
        }
    };
    eprintln!("Tolk_Speak {} {}", str_, interrupt);

    // SAFETY: We must have one open and active connection
    unsafe {
        let conn = CONNECTION.load(Ordering::Relaxed);
        let conn = ffi::ConnectionHandle::from(conn);

        // If message should interrupt, stop speaking
        // Ignore errors that might occur
        if interrupt {
            ffi::stop_speaking(conn);
        }

        ffi::speak(conn, &str_)
    }
}

/// Dummy function
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Braille(str_: *const u16) -> bool {
    eprintln!("Tolk_Braille {:?}", str_);
    // We do not support braille
    false
}

/// Dummy function (for now)
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_IsSpeaking() -> bool {
    eprintln!("Tolk_IsSpeaking");

    // Technically, it is possible to implement this
    // However, in order to massively simplify the code
    // (no async notification handling), we don't.
    // This matches what many of the drivers in fact do.
    false
}

/// Stop speaking
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Silence() -> bool {
    eprintln!("Tolk_Silence");
    if !Tolk_IsLoaded() {
        return false;
    }

    // SAFETY: We must have one open and active connection
    unsafe {
        let conn = CONNECTION.load(Ordering::Relaxed);
        let conn = ffi::ConnectionHandle::from(conn);
        ffi::stop_speaking(conn)
    }
}
