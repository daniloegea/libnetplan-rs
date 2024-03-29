#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::CString;

use crate::libnetplan::error_get_message;
use crate::libnetplan::netplan_parser_clear;
use crate::libnetplan::netplan_parser_load_keyfile;
use crate::libnetplan::netplan_parser_load_yaml;
use crate::libnetplan::netplan_parser_load_yaml_hierarchy;
use crate::libnetplan::netplan_parser_new;
use crate::libnetplan::LibNetplanError;
use crate::libnetplan::NetplanError;
use crate::libnetplan::NetplanParser;
use crate::libnetplan::NetplanResult;

pub struct Parser {
    pub(crate) parser: *mut NetplanParser,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            parser: unsafe { netplan_parser_new() },
        }
    }

    // TODO: implement all the possible errors it can return
    pub fn load_yaml_hierarchy(&self, root_dir: &str) -> NetplanResult<()> {
        let path = CString::new(root_dir).unwrap();
        unsafe {
            let mut error_message = ::std::ptr::null_mut::<NetplanError>();
            let error =
                netplan_parser_load_yaml_hierarchy(self.parser, path.as_ptr(), &mut error_message);
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

    pub fn load_yaml(&self, filename: &str) -> NetplanResult<()> {
        let path = CString::new(filename).unwrap();

        unsafe {
            let mut error_message = ::std::ptr::null_mut::<NetplanError>();
            let error = netplan_parser_load_yaml(self.parser, path.as_ptr(), &mut error_message);
            if error == 0 {
                if !error_message.is_null() {
                    if let Ok(message) = error_get_message(error_message) {
                        return Err(LibNetplanError::NetplanFileError(message));
                    } else {
                        return Err(LibNetplanError::NetplanFileError(
                            "load YAML error".to_string(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn load_keyfile(&self, filename: &str) -> NetplanResult<()> {
        let path = CString::new(filename).unwrap();

        unsafe {
            let mut error_message = ::std::ptr::null_mut::<NetplanError>();
            let error = netplan_parser_load_keyfile(self.parser, path.as_ptr(), &mut error_message);
            if error == 0 {
                if !error_message.is_null() {
                    if let Ok(message) = error_get_message(error_message) {
                        return Err(LibNetplanError::NetplanFileError(message));
                    } else {
                        return Err(LibNetplanError::NetplanFileError(
                            "load YAML error".to_string(),
                        ));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::DirBuilder;
    use std::fs::{self, File};
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    #[test]
    fn test_create_a_parser() {
        let parser = Parser::new();
        assert!(parser.parser != ::std::ptr::null_mut());
    }

    #[test]
    fn test_load_hierarchy_ok() {
        let root_dir = tempdir().expect("Cannot create tempdir for test");
        let filename = root_dir.path().join("etc/netplan/10-config.yaml");

        let _dirbuilder = DirBuilder::new()
            .recursive(true)
            .create(root_dir.path().join("etc/netplan"))
            .unwrap();

        let mut tmp_file = File::create(filename).expect("Cannot create tempfile for test");
        let mut perms = tmp_file.metadata().unwrap().permissions();
        perms.set_mode(0o0600);
        tmp_file
            .set_permissions(perms)
            .expect("Cannot set permission to 600");

        let yaml = r"
network:
  ethernets:
    eth0:
      dhcp4: true"
            .as_bytes();

        tmp_file
            .write(yaml)
            .expect("Cannot write YAML content for test");

        let parser = Parser::new();

        let root_dir_string = root_dir.path().to_str().unwrap().to_string();

        let parser_result = parser.load_yaml_hierarchy(&root_dir_string);

        assert!(parser_result.is_ok());

        fs::remove_file(root_dir.path().join("etc/netplan/10-config.yaml"))
            .expect("Cannot remove file");
        fs::remove_dir(root_dir.path().join("etc/netplan"))
            .expect("Cannot remove directory after test");
        fs::remove_dir(root_dir.path().join("etc")).expect("Cannot remove directory after test");
        root_dir.close().expect("Cannot close directory");
    }

    #[test]
    fn test_load_hierarchy_err() {
        let root_dir = tempdir().expect("Cannot create tempdir for test");
        let filename = root_dir.path().join("etc/netplan/10-config.yaml");

        let _dirbuilder = DirBuilder::new()
            .recursive(true)
            .create(root_dir.path().join("etc/netplan"))
            .unwrap();

        let mut tmp_file = File::create(filename).expect("Cannot create tempfile for test");
        let mut perms = tmp_file.metadata().unwrap().permissions();
        perms.set_mode(0o0600);
        tmp_file
            .set_permissions(perms)
            .expect("Cannot set permission to 600");

        let yaml = r"
network:
  ethernets:
    eth0:
      dhcp4: badvalue"
            .as_bytes();

        tmp_file
            .write(yaml)
            .expect("Cannot write YAML content for test");

        let parser = Parser::new();

        let root_dir_string = root_dir.path().to_str().unwrap().to_string();

        let parser_result = parser.load_yaml_hierarchy(&root_dir_string);

        assert!(parser_result.is_err());

        if let Err(error) = parser_result {
            if let LibNetplanError::NetplanFileError(error_message) = error {
                assert!(error_message
                    .contains("Error in network definition: invalid boolean value 'badvalue'"));
            }
        }

        fs::remove_file(root_dir.path().join("etc/netplan/10-config.yaml"))
            .expect("Cannot remove file");
        fs::remove_dir(root_dir.path().join("etc/netplan"))
            .expect("Cannot remove directory after test");
        fs::remove_dir(root_dir.path().join("etc")).expect("Cannot remove directory after test");
        root_dir.close().expect("Cannot close directory");
    }

    #[test]
    fn test_load_yaml_ok() {
        let root_dir = tempdir().expect("Cannot create tempdir for test");
        let filename = root_dir.path().join("10-config.yaml");

        let mut tmp_file = File::create(filename).expect("Cannot create tempfile for test");
        let mut perms = tmp_file.metadata().unwrap().permissions();
        perms.set_mode(0o0600);
        tmp_file
            .set_permissions(perms)
            .expect("Cannot set permission to 600");

        let yaml = r"
network:
  ethernets:
    eth0:
      dhcp4: true"
            .as_bytes();

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
    }

    #[test]
    fn test_load_yaml_err() {
        let root_dir = tempdir().expect("Cannot create tempdir for test");
        let filename = root_dir.path().join("10-config.yaml");

        let mut tmp_file = File::create(filename).expect("Cannot create tempfile for test");
        let mut perms = tmp_file.metadata().unwrap().permissions();
        perms.set_mode(0o0600);
        tmp_file
            .set_permissions(perms)
            .expect("Cannot set permission to 600");

        let yaml = r"
network:
  ethernets:
    eth0:
      dhcp4: badvalue"
            .as_bytes();

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

        assert!(parser_result.is_err());

        if let Err(error) = parser_result {
            if let LibNetplanError::NetplanFileError(error_message) = error {
                assert!(error_message
                    .contains("Error in network definition: invalid boolean value 'badvalue'"));
            }
        }

        fs::remove_file(root_dir.path().join("10-config.yaml")).expect("Cannot remove file");
        root_dir.close().expect("Cannot close directory");
    }

    #[test]
    fn test_load_keyfile_ok() {
        let root_dir = tempdir().expect("Cannot create tempdir for test");
        let file = "enp3s0.nmconnection";
        let filename = root_dir.path().join(file);

        let mut tmp_file = File::create(&filename).expect("Cannot create tempfile for test");

        let keyfile = r"[connection]
id=netplan-enp3s0
type=ethernet
interface-name=enp3s0
uuid=6352c897-174c-4f61-9623-556eddad05b2
[ipv4]
method=manual
address1=10.100.1.39/24"
            .as_bytes();

        tmp_file
            .write(keyfile)
            .expect("Cannot write YAML content for test");

        let parser = Parser::new();

        let filename_str = filename.to_str().unwrap().to_string();

        let parser_result = parser.load_keyfile(&filename_str);

        assert!(parser_result.is_ok());

        fs::remove_file(filename).expect("Cannot remove file");
        root_dir.close().expect("Cannot close directory");
    }

    #[test]
    fn test_load_keyfile_err() {
        let root_dir = tempdir().expect("Cannot create tempdir for test");
        let file = "enp3s0.nmconnection";
        let filename = root_dir.path().join(file);

        let mut tmp_file = File::create(&filename).expect("Cannot create tempfile for test");

        let keyfile = r"[connection]
id=netplan-enp3s0
type=ethernet
interface-name=enp3s0
[ipv4]
method=manual
address1=10.100.1.39/24"
            .as_bytes();

        tmp_file
            .write(keyfile)
            .expect("Cannot write nmconnection content for test");

        let parser = Parser::new();

        let filename_str = filename.to_str().unwrap().to_string();

        let parser_result = parser.load_keyfile(&filename_str);

        assert!(parser_result.is_err());

        if let Err(error) = parser_result {
            if let LibNetplanError::NetplanFileError(error_message) = error {
                assert!(error_message.contains("Keyfile: cannot find connection.uuid"));
            }
        }

        fs::remove_file(filename).expect("Cannot remove file");
        root_dir.close().expect("Cannot close directory");
    }
}
