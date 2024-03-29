#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::CStr;
use std::result;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[derive(Debug)]
pub enum LibNetplanError {
    NetplanParserError,
    NetplanValidationError(String),
    NetplanFileError(String),
}

pub type NetplanResult<T> = result::Result<T, LibNetplanError>;

pub(crate) fn netdef_get_id(netdef: *const NetplanNetDefinition) -> Result<String, String> {
    let mut size = 128;
    loop {
        let mut name: Vec<u8> = vec![b'\0'; size];
        let copied =
            unsafe { netplan_netdef_get_id(netdef, name.as_mut_ptr() as *mut i8, name.len()) }
                as isize;

        if copied == 0 {
            println!("copied is zero");
            return Err("copied is zero".to_string());
        }

        if copied == -2 {
            size *= 2;

            if size > 1048576 {
                return Err("data is too big".to_string());
            }
            continue;
        }

        let name_raw = CStr::from_bytes_until_nul(&name).unwrap();
        let name_string = name_raw.to_string_lossy().to_string();

        return Ok(name_string);
    }
}

pub fn error_get_message(error: *mut NetplanError) -> Result<String, String> {
    let mut size = 128;
    loop {
        let mut error_msg: Vec<u8> = vec![b'\0'; size];
        let copied = unsafe {
            netplan_error_message(error, error_msg.as_mut_ptr() as *mut i8, error_msg.len())
        } as isize;

        if copied == 0 {
            println!("copied is zero");
            return Err("copied is zero".to_string());
        }

        if copied == -2 {
            size *= 2;

            if size > 1048576 {
                return Err("data is too big".to_string());
            }
            continue;
        }

        let error_msg_raw = CStr::from_bytes_until_nul(&error_msg).unwrap();
        let error_msg_string = error_msg_raw.to_string_lossy().to_string();

        return Ok(error_msg_string);
    }
}

pub struct Netdef {
    pub name: String,
}
