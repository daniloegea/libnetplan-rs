#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]


use std::ffi::{CString, CStr};
use std::os::unix::io::AsRawFd;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub struct Parser {
    parser: *mut NetplanParser,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            parser: unsafe { netplan_parser_new() },
        }
    }

    pub fn load_yaml_hierarchy(&self, root_dir: &str) {
        let path = CString::new(root_dir).unwrap();
        unsafe {
            netplan_parser_load_yaml_hierarchy(self.parser, path.as_ptr(), ::std::ptr::null_mut());
        }
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

    pub fn import_parser_state(&self, parser: &Parser) {
        unsafe {
            netplan_state_import_parser_results(self.state, parser.parser, ::std::ptr::null_mut());
        }
    }

    pub fn dump_yaml(&self) {
        unsafe {
            netplan_state_dump_yaml(self.state, std::io::stdout().as_raw_fd(), ::std::ptr::null_mut());
        }
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
