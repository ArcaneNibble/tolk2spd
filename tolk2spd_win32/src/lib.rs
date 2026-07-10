use std::ptr;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};

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
static IS_WORKING: AtomicBool = AtomicBool::new(false);

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Load() {
    eprintln!("Tolk_Load");

    // Load the Wine unixlib, if it hasn't already
    if !ffi::load_unixlib() {
        return;
    }

    // This is set only if a connection is properly established
    IS_WORKING.store(true, Ordering::Relaxed);
    // Force the emulated screenreader name to be initialized now
    let _ = *EMULATED_SCREENREADER_NAME;
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_IsLoaded() -> bool {
    eprintln!("Tolk_IsLoaded");
    IS_WORKING.load(Ordering::Relaxed)
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Unload() {
    eprintln!("Tolk_Unload");
    // Unloading the unixlib doesn't actually work, so just update the dummy variable
    IS_WORKING.store(false, Ordering::Relaxed);
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
    eprintln!("Tolk_HasSpeech");
    if !Tolk_IsLoaded() {
        return false;
    }
    // We support _only_ speech
    true
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_HasBraille() -> bool {
    eprintln!("Tolk_HasBraille");
    // We do not support braille
    false
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Output(str_: *const u16, interrupt: bool) -> bool {
    eprintln!("Tolk_Output {:?} {}", str_, interrupt);
    // Since we only support speech, redirect
    Tolk_Speak(str_, interrupt)
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Speak(str_: *const u16, interrupt: bool) -> bool {
    eprintln!("Tolk_Speak {:?} {}", str_, interrupt);
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
