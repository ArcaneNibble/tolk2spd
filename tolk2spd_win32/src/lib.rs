use std::ptr;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};

use windows_strings::w;

mod ffi;

struct LeakedPCWSTR(*const u16);
unsafe impl Send for LeakedPCWSTR {}
unsafe impl Sync for LeakedPCWSTR {}

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
static CONNECTION: AtomicU64 = AtomicU64::new(0);

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Load() {
    eprintln!("Tolk_Load");

    // Load the Wine unixlib, if it hasn't already
    if !ffi::load_unixlib() {
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

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_IsLoaded() -> bool {
    CONNECTION.load(Ordering::Relaxed) != 0
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Unload() {
    eprintln!("Tolk_Unload");
    // Close the connection
    let conn = CONNECTION.swap(9, Ordering::Relaxed);
    let conn = ffi::ConnectionHandle::from(conn);
    unsafe {
        ffi::disconnect(conn);
    }

    // We can't actually unload the unixlib
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_TrySAPI(try_sapi: bool) {
    eprintln!("Tolk_TrySAPI {}", try_sapi);
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_PreferSAPI(prefer_sapi: bool) {
    eprintln!("Tolk_PreferSAPI {}", prefer_sapi);
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_DetectScreenReader() -> *const u16 {
    eprintln!("Tolk_DetectScreenReader");
    if !Tolk_IsLoaded() {
        return ptr::null();
    }
    EMULATED_SCREENREADER_NAME.0
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_HasSpeech() -> bool {
    if !Tolk_IsLoaded() {
        return false;
    }
    // We support _only_ speech
    true
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_HasBraille() -> bool {
    // We do not support braille
    false
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Output(str_: *const u16, interrupt: bool) -> bool {
    // Since we only support speech, redirect
    Tolk_Speak(str_, interrupt)
}

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

    // TODO: Actually do something
    // for now we pretend that it worked
    true
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Braille(str_: *const u16) -> bool {
    eprintln!("Tolk_Braille {:?}", str_);
    // We do not support braille
    false
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_IsSpeaking() -> bool {
    eprintln!("Tolk_IsSpeaking");

    // Technically, it is possible to implement this
    // However, in order to massively simplify the code
    // (no async notification handling), we don't.
    // This matches what many of the drivers in fact do
    false
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Silence() -> bool {
    eprintln!("Tolk_Silence");
    if !Tolk_IsLoaded() {
        return false;
    }

    // TODO: Actually do something
    // for now we pretend that it worked
    true
}
