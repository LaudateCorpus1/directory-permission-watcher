use file_mode::{ModeError, ModePath, ProtectionBit, User};
use std::io::{Error as IOError, ErrorKind as IOErrorKind};
use std::path::{Path, PathBuf};

fn should_log_io_error(err: &IOError) -> bool {
    match err.kind() {
        // This is probably a temporary file
        IOErrorKind::NotFound => {
            return false;
        }
        _ => {
            return true;
        }
    }
}

pub fn check_permission(p: &Path) {
    match p.mode() {
        Ok(mut mode) => {
            if let Some(file_type) = mode.file_type() {
                if file_type.is_directory() || file_type.is_regular_file() {
                    let mut should_update_path = false;

                    let mut owner_protection = mode.user_protection(User::Owner);
                    let mut group_protection = mode.user_protection(User::Group);
                    let mut other_protection = mode.user_protection(User::Other);

                    // Owner permissions
                    if !owner_protection.is_read_set() {
                        should_update_path = true;
                        owner_protection.set(ProtectionBit::Read)
                    }
                    if !owner_protection.is_write_set() {
                        should_update_path = true;
                        owner_protection.set(ProtectionBit::Write)
                    }

                    // Group permissions
                    if !group_protection.is_read_set() {
                        should_update_path = true;
                        group_protection.set(ProtectionBit::Read)
                    }
                    if !group_protection.is_write_set() {
                        should_update_path = true;
                        group_protection.set(ProtectionBit::Write)
                    }

                    // Other permissions
                    if !other_protection.is_read_set() {
                        should_update_path = true;
                        other_protection.set(ProtectionBit::Read)
                    }

                    // Directory needs execute permissions as well
                    if file_type.is_directory() {
                        if !owner_protection.is_execute_set() {
                            should_update_path = true;
                            owner_protection.set(ProtectionBit::Execute)
                        }
                        if !group_protection.is_execute_set() {
                            should_update_path = true;
                            group_protection.set(ProtectionBit::Execute)
                        }
                        if !other_protection.is_execute_set() {
                            should_update_path = true;
                            other_protection.set(ProtectionBit::Execute)
                        }
                    }

                    if should_update_path {
                        mode.set_protection(User::Owner, &owner_protection);
                        mode.set_protection(User::Group, &group_protection);
                        mode.set_protection(User::Other, &other_protection);

                        match p.set_mode(mode) {
                            Ok(_) => {
                                if cfg!(debug_assertions) {
                                    println!("Updated permissions of {:?}", p)
                                }
                            }
                            Err(err) => match err {
                                ModeError::IoError(err) => {
                                    if should_log_io_error(&err) {
                                        println!("Could not update permissions {:?}, {:?}", p, err)
                                    } else if cfg!(debug_assertions) {
                                        println!(
                                            "[DEBUG-ERROR] update permissions: {:?}, {:?}",
                                            p, err
                                        );
                                    }
                                }
                                _ => {
                                    println!("Invalid permissions {:?}", err)
                                }
                            },
                        }
                    }
                }
            }
        }
        Err(err) => {
            if should_log_io_error(&err) {
                println!("Could not load permissions of {:?}, {:?}", p, err)
            } else if cfg!(debug_assertions) {
                println!("[DEBUG-ERROR] load permissions: {:?}, {:?}", p, err);
            }
        }
    }
}

pub fn check_permissions(paths: Vec<PathBuf>) {
    for path in paths.clone().iter() {
        let p = path.as_path();

        if cfg!(debug_assertions) {
            println!("Validating file permissions of {:?}", p);
        }

        check_permission(p);
    }
}