//! Try to determine the current username
//!
//! This is something that SSIP would like to have as part of its CLIENT_NAME

use std::alloc::Layout;
use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::ptr;

use libc::*;

/// Return the current username
///
/// This looks at the current EUID and attempts to convert it to a username.
/// If the conversion fails, it returns the numeric ID converted to string.
pub fn get_username() -> String {
    unsafe {
        let uid = geteuid();

        let pwdb_max_sz = sysconf(_SC_GETPW_R_SIZE_MAX);
        if pwdb_max_sz < 0 {
            return format!("{}", uid);
        }
        let pwdb_max_sz = pwdb_max_sz as usize;
        let buf_layout = Layout::from_size_align(pwdb_max_sz, 1).unwrap();

        let pwdb_buf = std::alloc::alloc_zeroed(buf_layout);
        let mut passwd = MaybeUninit::<passwd>::zeroed().assume_init();
        let mut _useless = ptr::null_mut();
        let ret = getpwuid_r(
            uid,
            &mut passwd,
            pwdb_buf as *mut c_char,
            pwdb_max_sz,
            &mut _useless,
        );
        if ret < 0 {
            return format!("{}", uid);
        }

        let username = CStr::from_ptr(passwd.pw_name);
        let ret = username.to_string_lossy().into_owned();

        std::alloc::dealloc(pwdb_buf, buf_layout);

        ret
    }
}
