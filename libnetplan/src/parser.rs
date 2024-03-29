#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]


use std::ffi::CString;

use crate::libnetplan::error_get_message;
use crate::libnetplan::netplan_parser_new;
use crate::libnetplan::netplan_parser_clear;
use crate::libnetplan::netplan_parser_load_yaml_hierarchy;
use crate::libnetplan::NetplanParser;
use crate::libnetplan::NetplanResult;
use crate::libnetplan::NetplanError;
use crate::libnetplan::LibNetplanError;

pub struct Parser {
    pub(crate) parser: *mut NetplanParser,
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
                if ! error_message.is_null() {
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
