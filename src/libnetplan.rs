#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]

use std::ffi::{CStr, CString};
use std::os::fd::{FromRawFd, OwnedFd};
use std::result;

use crate::netdef::NetdefType;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[derive(Debug)]
pub enum NetplanErrorDomains {
    NetplanGenericError,
    NetplanParserError(String),
    NetplanValidationError(String),
    NetplanFileError(String),
}

impl NetplanErrorDomains {
    pub(crate) fn from_libnetplan_error(error: &LibNetplanError) -> Self {
        match error.domain {
            1 => NetplanErrorDomains::NetplanParserError(error.message.clone()),
            2 => NetplanErrorDomains::NetplanValidationError(error.message.clone()),
            3 => NetplanErrorDomains::NetplanFileError(error.message.clone()),
            _ => NetplanErrorDomains::NetplanGenericError,
        }
    }
}

pub(crate) struct LibNetplanError {
    pub(crate) code: u32,
    pub(crate) domain: u32,
    pub(crate) message: String,
}

impl LibNetplanError {
    pub fn try_from_raw_error(error: *mut NetplanError) -> Option<Self> {
        let message = error_get_message(error)?;
        let error_code = error_get_code(error)?;

        let domain = (error_code >> 32) as u32;
        let code = error_code as u32;

        Some(Self {
            code,
            domain,
            message,
        })
    }
}

pub type NetplanResult<T> = result::Result<T, NetplanErrorDomains>;

pub(crate) fn netdef_get_id(netdef: *const NetplanNetDefinition) -> Result<String, String> {
    let name_string = unsafe {
        copy_string_realloc_call(
            |netdef, buffer, len| {
                netplan_netdef_get_id(netdef as *const netplan_net_definition, buffer, len)
            },
            netdef as *const i8,
        )
        .unwrap()
    };
    Ok(name_string)
}

pub(crate) fn error_get_message(error: *mut GError) -> Option<String> {
    let name_string = unsafe {
        copy_string_realloc_call(
            |error, buffer, len| netplan_error_message(error as *mut GError, buffer, len),
            error as *const i8,
        )
    };

    match name_string {
        Ok(message) => Some(message),
        Err(_) => None,
    }
}

pub(crate) fn error_get_code(error: *mut GError) -> Option<u64> {
    Some(unsafe { netplan_error_code(error as *mut GError) })
}

fn copy_string_realloc_call<F>(call: F, ptr: *const i8) -> Result<String, String>
where
    F: FnOnce(*const i8, *mut i8, usize) -> isize + Copy,
{
    let mut size = 128;
    loop {
        let mut name: Vec<u8> = vec![b'\0'; size];
        let copied = call(ptr, name.as_mut_ptr() as *mut i8, name.len());

        if copied == 0 {
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

pub(crate) fn netdef_get_type(netdef: *const NetplanNetDefinition) -> NetdefType {
    let netdef_type = unsafe { netplan_netdef_get_type(netdef) };

    match netdef_type {
        NetplanDefType_NETPLAN_DEF_TYPE_ETHERNET => NetdefType::Ethernet,
        NetplanDefType_NETPLAN_DEF_TYPE_WIFI => NetdefType::Wifi,
        NetplanDefType_NETPLAN_DEF_TYPE_MODEM => NetdefType::Modem,
        NetplanDefType_NETPLAN_DEF_TYPE_BRIDGE => NetdefType::Bridge,
        NetplanDefType_NETPLAN_DEF_TYPE_BOND => NetdefType::Bond,
        NetplanDefType_NETPLAN_DEF_TYPE_VLAN => NetdefType::Vlan,
        NetplanDefType_NETPLAN_DEF_TYPE_TUNNEL => NetdefType::Tunnel,
        NetplanDefType_NETPLAN_DEF_TYPE_PORT => NetdefType::Port,
        NetplanDefType_NETPLAN_DEF_TYPE_VRF => NetdefType::Vrf,
        NetplanDefType_NETPLAN_DEF_TYPE_NM => NetdefType::Nm,
        NetplanDefType_NETPLAN_DEF_TYPE_DUMMY => NetdefType::Dummy,
        NetplanDefType_NETPLAN_DEF_TYPE_VETH => NetdefType::Veth,
        _ => NetdefType::None,
    }
}

/* Simple wrapper around libc's memfd_create to avoid importing other crates
   memfd_create() is defined in netplan.h
*/
pub(crate) fn netplan_memfd_create() -> Result<OwnedFd, String> {
    unsafe {
        let name_cstr = CString::new("netplan_memfd").unwrap();
        let ret = memfd_create(name_cstr.as_ptr(), 0);

        if ret >= 0 {
            return Ok(OwnedFd::from_raw_fd(ret));
        } else {
            return Err("memfd_create failed".to_string());
        }
    }
}
