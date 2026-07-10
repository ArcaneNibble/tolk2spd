use std::ptr;

mod ffi;

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Load() {
    eprintln!("Tolk_Load");
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_IsLoaded() -> bool {
    eprintln!("Tolk_IsLoaded");
    false
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C" fn Tolk_Unload() {
    eprintln!("Tolk_Unload");
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
    ptr::null()
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
