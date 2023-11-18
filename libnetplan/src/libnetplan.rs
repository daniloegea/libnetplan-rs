#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]


use std::ffi::{CString, CStr};
use std::fs::File;
use std::os::fd::FromRawFd;
use std::os::unix::io::AsRawFd;
use std::result;
use std::io::{prelude::*, Read, SeekFrom};

use nix::sys::memfd::{memfd_create, MemFdCreateFlag};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[derive(Debug)]
pub enum LibNetplanError {
   NetplanParserError,
   NetplanValidationError(String),
   NetplanFileError(String),
}

pub type NetplanResult<T> = result::Result<T, LibNetplanError>;

pub struct Parser {
    parser: *mut NetplanParser,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            parser: unsafe { netplan_parser_new() },
        }
    }

    pub fn load_yaml_hierarchy(&self, root_dir: &str) -> NetplanResult<()>{
        let path = CString::new(root_dir).unwrap();
        unsafe {
            let mut error_message = ::std::ptr::null_mut::<NetplanError>();
            let error = netplan_parser_load_yaml_hierarchy(self.parser, path.as_ptr(), &mut error_message);
            if error == 0 {
                if error_message != ::std::ptr::null_mut() {
                    if let Ok(message) = error_get_message(error_message) {
                        return Err(LibNetplanError::NetplanFileError(message));
                    } else {
                        return Err(LibNetplanError::NetplanFileError("load hierarchy error".to_string()));
                    }
                }
            }
        }
        Ok(())
    }
}

impl Drop for Parser {
    fn drop(&mut self) {
        unsafe { netplan_parser_clear(&mut self.parser) };
    }
}

pub struct State {
    state: *mut NetplanState,
    iter: *mut netdef_pertype_iter,
}

impl State {
    pub fn new() -> Self {
        State {
            state: unsafe { netplan_state_new() },
            iter: ::std::ptr::null_mut(),
        }
    }

    pub fn import_parser_state(&self, parser: &Parser) -> NetplanResult<()> {
        unsafe {
            let mut error_message = ::std::ptr::null_mut::<NetplanError>();
            let error = netplan_state_import_parser_results(self.state, parser.parser, &mut error_message);
            if error == 0 {
                if error_message != ::std::ptr::null_mut() {
                    if let Ok(message) = error_get_message(error_message) {
                        return Err(LibNetplanError::NetplanValidationError(message));
                    } else {
                        return Err(LibNetplanError::NetplanValidationError("import parser state error".to_string()));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn dump_yaml(&self) -> NetplanResult<String>{
        let mem_file = memfd_create(&CString::new("netplan_yaml").unwrap(), MemFdCreateFlag::MFD_CLOEXEC).expect("Cannot create memory file");
        let mut file;
        unsafe {
            netplan_state_dump_yaml(self.state, mem_file.as_raw_fd(), ::std::ptr::null_mut());
            file = File::from_raw_fd(mem_file.as_raw_fd());
        }
        _ = file.seek(SeekFrom::Start(0));
        let mut yaml = String::new();
        file.read_to_string(&mut yaml).expect("Cannot read from memory file");
        Ok(yaml)
    }

}

pub fn netdef_get_id(netdef: *const NetplanNetDefinition) -> Result<String, String> {
    let mut size = 128;
    loop {
        let mut name: Vec<u8> = vec![b'\0'; size];
        let copied = unsafe { netplan_netdef_get_id(netdef, name.as_mut_ptr() as *mut i8, name.len()) } as isize;

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
        let copied = unsafe { netplan_error_message(error, error_msg.as_mut_ptr() as *mut i8, error_msg.len()) } as isize;

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

impl Iterator for State {
    type Item = Netdef;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iter == ::std::ptr::null_mut() {
            self.iter = unsafe { _netplan_state_new_netdef_pertype_iter(self.state, ::std::ptr::null_mut()) };
        }

        let netdef = unsafe {_netplan_netdef_pertype_iter_next(self.iter) };

        if netdef == ::std::ptr::null_mut() {
            return None;
        }
        
        let name_string = netdef_get_id(netdef).unwrap();

        Some(Netdef { name: name_string})
    }
}

impl Drop for State {
    fn drop(&mut self) {
        unsafe { netplan_state_clear(&mut self.state) };
    }
}

pub struct Netdef {
    pub name: String,
}
