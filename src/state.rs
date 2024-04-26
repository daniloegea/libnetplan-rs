use std::ffi::CString;
use std::fs::File;
use std::io::{prelude::*, Read, SeekFrom};
use std::os::fd::FromRawFd;
use std::os::unix::io::AsRawFd;
use std::ptr::null_mut;

use crate::libnetplan::netdef_pertype_iter;
use crate::libnetplan::netplan_state_clear;
use crate::libnetplan::netplan_state_dump_yaml;
use crate::libnetplan::netplan_state_import_parser_results;
use crate::libnetplan::netplan_state_new;
use crate::libnetplan::netplan_util_dump_yaml_subtree;
use crate::libnetplan::LibNetplanError;
use crate::libnetplan::NetplanError;
use crate::libnetplan::NetplanResult;
use crate::libnetplan::NetplanState;
use crate::libnetplan::{_netplan_netdef_pertype_iter_next, netplan_memfd_create};
use crate::libnetplan::{
    _netplan_state_new_netdef_pertype_iter, netplan_state_update_yaml_hierarchy,
};
use crate::libnetplan::{error_get_message, netplan_state_write_yaml_file};
use crate::netdef::{Netdef, NetdefType};
use crate::parser::Parser;

pub struct State {
    pub(crate) state: *mut NetplanState,
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
        let mem_file = netplan_memfd_create().expect("Cannot create memory file");
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
        let input_file = netplan_memfd_create().expect("Cannot create memory file");
        let output_file = netplan_memfd_create().expect("Cannot create memory file");
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

    pub fn update_yaml_hierarchy(
        &self,
        default_filename: &str,
        root_dir: &str,
    ) -> NetplanResult<()> {
        let default_filename_cstr = CString::new(default_filename).unwrap();
        let root_dir_cstr = CString::new(root_dir).unwrap();

        unsafe {
            let ret = netplan_state_update_yaml_hierarchy(
                self.state,
                default_filename_cstr.as_ptr(),
                root_dir_cstr.as_ptr(),
                null_mut(),
            );

            if ret == 0 {
                return Err(LibNetplanError::NetplanFileError(
                    "update_yaml_hierarchy failed".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub fn write_yaml_file(&self, filename: &str, root_dir: &str) -> NetplanResult<()> {
        let filename_cstr = CString::new(filename).unwrap();
        let rootdir_cstr = CString::new(root_dir).unwrap();
        unsafe {
            let mut error_message = ::std::ptr::null_mut::<NetplanError>();
            let error = netplan_state_write_yaml_file(
                self.state,
                filename_cstr.as_ptr(),
                rootdir_cstr.as_ptr(),
                &mut error_message,
            );
            if error == 0 {
                if !error_message.is_null() {
                    if let Ok(message) = error_get_message(error_message) {
                        return Err(LibNetplanError::NetplanFileError(message));
                    } else {
                        return Err(LibNetplanError::NetplanFileError(
                            "load hierarchy error".to_string(),
                        ));
                    }
                }
            }
        }
        Ok(())
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

        let netdef = Netdef::from_raw_netdef(netdef);
        Some(netdef)
    }
}

impl Drop for State {
    fn drop(&mut self) {
        unsafe { netplan_state_clear(&mut self.state) };
    }
}

impl TryFrom<Parser> for State {
    type Error = LibNetplanError;
    fn try_from(value: Parser) -> Result<Self, Self::Error> {
        let state = State::new();
        match state.import_parser_state(value) {
            Ok(_) => Ok(state),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    fn create_parser(yaml: &str) -> Parser {
        let root_dir = tempdir().expect("Cannot create tempdir for test");
        let filename = root_dir.path().join("10-config.yaml");

        let mut tmp_file = File::create(filename).expect("Cannot create tempfile for test");
        let mut perms = tmp_file.metadata().unwrap().permissions();
        perms.set_mode(0o0600);
        tmp_file
            .set_permissions(perms)
            .expect("Cannot set permission to 600");

        let yaml = yaml.as_bytes();

        tmp_file
            .write(yaml)
            .expect("Cannot write YAML content for test");

        let parser = Parser::new();

        let filename_str = root_dir
            .path()
            .join("10-config.yaml")
            .to_str()
            .unwrap()
            .to_string();

        let parser_result = parser.load_yaml(&filename_str);

        assert!(parser_result.is_ok());

        fs::remove_file(root_dir.path().join("10-config.yaml")).expect("Cannot remove file");
        root_dir.close().expect("Cannot close directory");

        parser
    }

    #[test]
    fn test_create_a_state() {
        let state = State::new();
        assert!(state.state != ::std::ptr::null_mut());
    }

    #[test]
    fn test_import_parser_results_validation_error() {
        let yaml = r"
network:
  vrfs:
    vrf0:
      table: 1000
      routes:
        - to: 192.168.0.0/24
          via: 1.2.3.4
          table: 2000
  ethernets:
    eth0:
      dhcp4: true
    eth1: {}
    eth2:
      addresses:
        - 192.168.0.1/24
        - 10.20.30.40/24
      nameservers:
        addresses:
          - 192.168.0.254
        search:
          - mydomain.local";

        let parser = create_parser(yaml);

        let state = State::new();

        if let Err(error) = state.import_parser_state(parser) {
            if let LibNetplanError::NetplanValidationError(msg) = error {
                assert_eq!(msg, "vrf0: VRF routes table mismatch (1000 != 2000)");
            }
        } else {
            assert!(false, "This test should have passed but didn't");
        }
    }

    #[test]
    fn test_import_parser_results() {
        let yaml = r"
network:
  ethernets:
    eth0:
      dhcp4: true
    eth1: {}
    eth2:
      addresses:
        - 192.168.0.1/24
        - 10.20.30.40/24
      routes:
        - to: default
          via: 192.168.0.254
      nameservers:
        addresses:
          - 192.168.0.254
        search:
          - mydomain.local";

        let parser = create_parser(yaml);

        let state = State::new();

        if let Err(_) = state.import_parser_state(parser) {
            assert!(false, "load parser results failed");
        }
    }

    #[test]
    fn test_dump_yaml() {
        let yaml = r"
network:
  ethernets:
    eth0:
      dhcp4: true";

        let parser = create_parser(yaml);

        let state = State::new();

        if let Err(_) = state.import_parser_state(parser) {
            assert!(false, "load parser results failed");
        }

        let dump = state.dump_yaml();

        assert_eq!(
            "network:\n  version: 2\n  ethernets:\n    eth0:\n      dhcp4: true\n",
            dump.unwrap()
        );
    }

    #[test]
    fn test_state_iterator() {
        let yaml = r"
network:
  ethernets:
    eth0:
      dhcp4: true
    eth1: {}
    eth2:
      addresses:
        - 192.168.0.1/24
        - 10.20.30.40/24
      routes:
        - to: default
          via: 192.168.0.254
      nameservers:
        addresses:
          - 192.168.0.254
        search:
          - mydomain.local";

        let parser = create_parser(yaml);

        let state = State::new();
        state.import_parser_state(parser).unwrap();

        let mut netdef_ids = Vec::new();
        let netdef_ids_expected = vec!["eth0", "eth1", "eth2"];

        for netdef in state {
            assert!(matches!(netdef.r#type, NetdefType::Ethernet));
            netdef_ids.push(netdef.id.clone());
        }

        for expected in netdef_ids_expected {
            assert!(netdef_ids.contains(&expected.to_string()));
        }
    }

    #[test]
    fn test_state_try_from() {
        let yaml = r"
network:
  ethernets:
    eth0:
      dhcp4: true";

        let parser = create_parser(yaml);

        if let Err(_) = State::try_from(parser) {
            assert!(false, "load parser results failed");
        }
    }
}
