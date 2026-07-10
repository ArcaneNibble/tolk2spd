use std::error::Error;
use std::ffi::c_void;
use std::fmt;
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

#[derive(Debug)]
pub struct SSIPRespose {
    pub code: u32,
    pub message: String,
}
impl SSIPRespose {
    pub fn is_ok(&self) -> bool {
        self.code >= 200 && self.code <= 299
    }
}

#[derive(Debug)]
pub enum SSIPError {
    IoError(io::Error),
    SSIPError(SSIPRespose),
}
impl Error for SSIPError {
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            SSIPError::IoError(e) => Some(e),
            _ => None,
        }
    }
}
impl From<io::Error> for SSIPError {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}
impl fmt::Display for SSIPError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SSIPError::IoError(e) => e.fmt(f),
            SSIPError::SSIPError(e) => write!(f, "{}", e.message),
        }
    }
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
        if !result.is_ok() {
            eprintln!("warn: setting ssip client_name failed! {}", result.message);
        }

        ret.speak(".\r\nthis is\r\n.\r\n a test\r\n.\r\n");

        Some(ret)
    }

    fn send_cmd(&mut self, cmd: &str) -> io::Result<SSIPRespose> {
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

        Ok(SSIPRespose {
            code: response_code.unwrap_or_default(),
            message: response_msg,
        })
    }

    pub fn speak(&mut self, to_speak: &str) -> Result<SSIPRespose, SSIPError> {
        // Replace potential badness
        let mut fixed = String::with_capacity(to_speak.len());
        let to_speak = if let Some(rest) = to_speak.strip_prefix(".") {
            fixed.push_str("..");
            rest
        } else {
            to_speak
        };

        for (i, x) in to_speak.split("\r\n.").enumerate() {
            if i != 0 {
                fixed.push_str("\r\n..");
            }
            fixed.push_str(x);
        }
        dbg!(&fixed);

        let resp = self.send_cmd("speak")?;
        if !resp.is_ok() {
            return Err(SSIPError::SSIPError(resp));
        }

        write!(self.stream, "{}", fixed)?;

        // Send terminating line
        let resp = self.send_cmd("\r\n.")?;
        if !resp.is_ok() {
            return Err(SSIPError::SSIPError(resp));
        }
        Ok(resp)
    }
}

impl Drop for SPDConnection {
    fn drop(&mut self) {
        dbg!("drop conn!");
    }
}
