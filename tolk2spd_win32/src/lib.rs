use std::ffi::c_void;
use std::ptr;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};

use windows_strings::w;
use windows_sys::Wdk::Storage::FileSystem::NtQueryVirtualMemory;
use windows_sys::Win32::System::Threading::GetCurrentProcess;

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
static DUMMY_IS_LOADED: AtomicBool = AtomicBool::new(false);

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Load() {
    eprintln!("Tolk_Load");

    // Load the Wine unixlib
    unsafe {
        if ffi::__wine_unixlib_handle.load(Ordering::Relaxed) != 0 {
            // unixlib already loaded
            return;
        }

        let img_base = &raw const ffi::__ImageBase;

        let mut wine_unixlib_handle: u64 = 0;
        let ret = NtQueryVirtualMemory(
            GetCurrentProcess(),
            img_base,
            1000,
            &raw mut wine_unixlib_handle as *mut c_void,
            std::mem::size_of_val(&wine_unixlib_handle),
            std::ptr::null_mut(),
        );
        if ret != 0 {
            return;
        }

        eprintln!("__wine_unixlib_handle = 0x{wine_unixlib_handle:016x}");
        ffi::__wine_unixlib_handle.store(wine_unixlib_handle, Ordering::Relaxed);

        let returned = ffi::__wine_unix_call_dispatcher(
            ffi::__wine_unixlib_handle.load(Ordering::Relaxed),
            0,
            ptr::null(),
        );
        dbg!(returned);
    }

    // We track a dummy "is loaded" state
    DUMMY_IS_LOADED.store(true, Ordering::Relaxed);
    // Force the emulated screenreader name to be initialized now
    let _ = *EMULATED_SCREENREADER_NAME;
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_IsLoaded() -> bool {
    eprintln!("Tolk_IsLoaded");
    DUMMY_IS_LOADED.load(Ordering::Relaxed)
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Unload() {
    eprintln!("Tolk_Unload");
    // Unloading the unixlib doesn't actually work, so just update the dummy variable
    DUMMY_IS_LOADED.store(false, Ordering::Relaxed);
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
    EMULATED_SCREENREADER_NAME.0
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_HasSpeech() -> bool {
    eprintln!("Tolk_HasSpeech");
    false
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_HasBraille() -> bool {
    eprintln!("Tolk_HasBraille");
    false
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Output(str_: *const u16, interrupt: bool) -> bool {
    eprintln!("Tolk_Output {:?} {}", str_, interrupt);
    false
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Speak(str_: *const u16, interrupt: bool) -> bool {
    eprintln!("Tolk_Speak {:?} {}", str_, interrupt);
    false
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Braille(str_: *const u16) -> bool {
    eprintln!("Tolk_Braille {:?}", str_);
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
    false
}
