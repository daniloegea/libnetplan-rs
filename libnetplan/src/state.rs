#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::CString;
use std::fs::File;
use std::io::{prelude::*, Read, SeekFrom};
use std::os::fd::FromRawFd;
use std::os::unix::io::AsRawFd;

use nix::sys::memfd::{memfd_create, MemFdCreateFlag};

use crate::libnetplan::_netplan_netdef_pertype_iter_next;
use crate::libnetplan::_netplan_state_new_netdef_pertype_iter;
use crate::libnetplan::error_get_message;
use crate::libnetplan::netdef_get_id;
use crate::libnetplan::netdef_pertype_iter;
use crate::libnetplan::netplan_state_clear;
use crate::libnetplan::netplan_state_dump_yaml;
use crate::libnetplan::netplan_state_import_parser_results;
use crate::libnetplan::netplan_state_new;
use crate::libnetplan::netplan_util_dump_yaml_subtree;
use crate::libnetplan::LibNetplanError;
use crate::libnetplan::Netdef;
use crate::libnetplan::NetplanError;
use crate::libnetplan::NetplanResult;
use crate::libnetplan::NetplanState;
use crate::parser::Parser;

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

    pub fn import_parser_state(&self, parser: Parser) -> NetplanResult<()> {
        unsafe {
            let mut error_message = ::std::ptr::null_mut::<NetplanError>();
            let error =
                netplan_state_import_parser_results(self.state, parser.parser, &mut error_message);
            if error == 0 {
                if !error_message.is_null() {
                    if let Ok(message) = error_get_message(error_message) {
                        return Err(LibNetplanError::NetplanValidationError(message));
                    } else {
                        return Err(LibNetplanError::NetplanValidationError(
                            "import parser state error".to_string(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn dump_yaml(&self) -> NetplanResult<String> {
        let mem_file = memfd_create(
            &CString::new("netplan_yaml").unwrap(),
            MemFdCreateFlag::MFD_CLOEXEC,
        )
        .expect("Cannot create memory file");
        unsafe {
            netplan_state_dump_yaml(self.state, mem_file.as_raw_fd(), ::std::ptr::null_mut());
        }
        let mut file = unsafe { File::from_raw_fd(mem_file.as_raw_fd()) };
        _ = file.seek(SeekFrom::Start(0));
        let mut yaml = String::new();
        file.read_to_string(&mut yaml)
            .expect("Cannot read from memory file");
        Ok(yaml)
    }

    pub fn dump_yaml_subtree(&self, subtree: &str) -> NetplanResult<String> {
        let input_file = memfd_create(
            &CString::new("netplan_input_yaml").unwrap(),
            MemFdCreateFlag::MFD_CLOEXEC,
        )
        .expect("Cannot create memory file");
        let output_file = memfd_create(
            &CString::new("netplan_output_yaml").unwrap(),
            MemFdCreateFlag::MFD_CLOEXEC,
        )
        .expect("Cannot create memory file");
        unsafe {
            netplan_state_dump_yaml(self.state, input_file.as_raw_fd(), ::std::ptr::null_mut());
        }
        let mut file = unsafe { File::from_raw_fd(input_file.as_raw_fd()) };
        _ = file.seek(SeekFrom::Start(0));

        let mut subtree_components: Vec<&str> = subtree.split('.').collect();
        if subtree_components[0] != "network" {
            subtree_components.insert(0, "network");
        }
        let subtree_string = CString::new(subtree_components.join("\t")).unwrap();

        unsafe {
            netplan_util_dump_yaml_subtree(
                subtree_string.as_ptr(),
                input_file.as_raw_fd(),
                output_file.as_raw_fd(),
                ::std::ptr::null_mut(),
            );
        }

        file = unsafe { File::from_raw_fd(output_file.as_raw_fd()) };
        _ = file.seek(SeekFrom::Start(0));
        let mut yaml = String::new();
        file.read_to_string(&mut yaml)
            .expect("Cannot read from memory file");
        Ok(yaml)
    }
}

impl Iterator for State {
    type Item = Netdef;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iter == ::std::ptr::null_mut() {
            self.iter = unsafe {
                _netplan_state_new_netdef_pertype_iter(self.state, ::std::ptr::null_mut())
            };
        }

        let netdef = unsafe { _netplan_netdef_pertype_iter_next(self.iter) };

        if netdef.is_null() {
            return None;
        }

        let name_string = netdef_get_id(netdef).unwrap();

        Some(Netdef { name: name_string })
    }
}

impl Drop for State {
    fn drop(&mut self) {
        unsafe { netplan_state_clear(&mut self.state) };
    }
}
