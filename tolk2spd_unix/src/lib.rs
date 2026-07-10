use std::ffi::c_void;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

mod ffi;

extern "C" fn get_version(_arg: *const c_void) -> u32 {
    tolk2spd_abi::ABI_VERSION
}

#[derive(Debug)]
pub struct SPDConnection {
    stream: UnixStream,
}

impl SPDConnection {
    pub fn new(_exename: &str) -> Option<Self> {
        // Try to find the socket path
        let mut sock_path = if let Some(x) = std::env::var_os("XDG_RUNTIME_DIR") {
            PathBuf::from(x)
        } else if let Some(x) = std::env::var_os("XDG_CACHE_HOME") {
            PathBuf::from(x)
        } else {
            if let Some(mut home) = std::env::home_dir() {
                home.push(".cache");
                home
            } else {
                // Give up
                return None;
            }
        };
        sock_path.push("speech-dispatcher/speechd.sock");
        dbg!(&sock_path);

        // Try to connect to the socket
        let mut stream = UnixStream::connect(&sock_path).ok()?;
        dbg!(&stream);

        write!(stream, "speak\r\nthis is a test\r\n.\r\n").ok()?;

        Some(Self { stream })
    }
}

impl Drop for SPDConnection {
    fn drop(&mut self) {
        dbg!("drop conn!");
    }
}
