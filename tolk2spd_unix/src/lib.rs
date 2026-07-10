use std::ffi::c_void;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

mod ffi;
mod whoami;

extern "C" fn get_version(_arg: *const c_void) -> u32 {
    tolk2spd_abi::ABI_VERSION
}

#[derive(Debug)]
pub struct SPDConnection {
    stream: UnixStream,
}

impl SPDConnection {
    pub fn new(exename: &str) -> Option<Self> {
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
        let stream = UnixStream::connect(&sock_path).ok()?;
        let mut ret = Self { stream };

        // Send the client name
        let username = whoami::get_username();
        let result = ret
            .send_cmd(&format!(
                "set self client_name {username}:{exename}:tolk2spd"
            ))
            .ok()?;
        if !(result.0 >= 200 && result.0 <= 299) {
            eprintln!("warn: setting ssip client_name failed! {}", result.1);
        }

        Some(ret)
    }

    fn send_cmd(&mut self, cmd: &str) -> io::Result<(u32, String)> {
        write!(self.stream, "{}\r\n", cmd)?;

        let mut response_code = None;
        let mut response_msg = String::new();
        let mut r = BufReader::new(&self.stream);
        loop {
            let start_idx = response_msg.len();
            let nbytes = r.read_line(&mut response_msg)?;
            if nbytes == 0 {
                return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
            }

            let l = &response_msg[start_idx..start_idx + nbytes];
            dbg!(l);

            if response_code.is_none() {
                response_code = Some(l[..3].parse::<u32>().unwrap_or_default());
            }

            if l.as_bytes()[3] != b'-' {
                break;
            }
        }

        Ok((response_code.unwrap_or_default(), response_msg))
    }
}

impl Drop for SPDConnection {
    fn drop(&mut self) {
        dbg!("drop conn!");
    }
}
