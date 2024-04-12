use std::{
    ffi::CString,
    fs::File,
    io::{Read, Seek},
    os::fd::AsRawFd,
    ptr::null_mut,
};

use crate::libnetplan::{netplan_memfd_create, netplan_util_create_yaml_patch};

pub fn netplan_create_yaml_patch(conf_obj_path: &str, obj_payload: &str) -> Result<String, String> {
    let output = netplan_memfd_create().unwrap();

    let patch = conf_obj_path.split(".").collect::<Vec<&str>>().join("\t");
    let patch_cstr = CString::new(patch).unwrap();
    let obj_payload_cstr = CString::new(obj_payload).unwrap();

    unsafe {
        let res = netplan_util_create_yaml_patch(
            patch_cstr.as_ptr(),
            obj_payload_cstr.as_ptr(),
            output.as_raw_fd(),
            null_mut(),
        );
        if res == 0 {
            return Err("create_yaml_patch failed".to_string());
        }
    }

    let mut file = File::from(output);
    let _ = file.rewind();
    let mut yaml_patch = String::new();
    let _ = file.read_to_string(&mut yaml_patch);

    Ok(yaml_patch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_yaml_patch() {
        let a = netplan_create_yaml_patch("network.ethernets.eth0.dhcp4", "false");

        if let Ok(output) = a {
            assert_eq!(
                output,
                "network:\n  ethernets:\n    eth0:\n      dhcp4: false\n"
            );
        } else {
            assert!(false);
        }
    }
}
