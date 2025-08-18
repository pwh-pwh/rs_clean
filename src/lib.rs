pub mod constant;
pub mod cmd;
pub mod utils;

use crate::constant::{EXCLUDE_DIR, COLOR_RED, COLOR_GRAY, COLOR_RESET};
use std::fs;
use std::path::Path;
use std::process::{exit, Command};


pub static mut COUNT: u32 = 0;

pub fn do_clean_all(dir: &Path,cmd_list: &mut Vec<cmd::Cmd>) {
    if dir.is_dir() {
        if let Some(dir_name) = dir.file_name() {
            // Handle invalid UTF-8 characters safely
            if let Some(dir_str) = dir_name.to_str() {
                if EXCLUDE_DIR.contains(&dir_str) {
                    return;
                }
                if dir_str.starts_with(".") {
                    return;
                }
            } else {
                // Skip directories with invalid UTF-8 names
                return;
            }
        }
        //定义变量flag 记录是否存在符合条件的文件
        let mut flag = false;
        cmd_list.iter_mut().for_each(|cmd| {
            cmd.related_files.clone().iter().for_each(|file| unsafe {
                if dir.join(file).exists() {
                    COUNT += 1;
                    flag = true;
                    println!("{}run:{} {} clean{} {}", COLOR_GRAY, COLOR_RESET, COLOR_RED, COLOR_RESET, dir.display());
                    cmd.current_dir(dir);
                    let _ = cmd.run().map_err(|e| {
                        eprintln!("{dir:?} > {e:?}");
                        exit(1)
                    });
                }
            })
        });
        if !flag {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_dir() {
                            do_clean_all(&path, cmd_list);
                        }
                    }
                }
            }
        }
    }
}

pub fn do_clean(dir: &Path, cmd: &mut Command) {
    if dir.is_dir() {
        if let Some(dir_name) = dir.file_name() {
            // Handle invalid UTF-8 characters safely
            if let Some(dir_str) = dir_name.to_str() {
                if EXCLUDE_DIR.contains(&dir_str) {
                    return;
                }
                if dir_str.starts_with(".") {
                    return;
                }
            } else {
                // Skip directories with invalid UTF-8 names
                return;
            }
        }
        let cargo_toml_path = dir.join("Cargo.toml");
        if cargo_toml_path.exists() {
            println!("{}clean{} {}", COLOR_RED, COLOR_RESET, dir.display());
            cmd.current_dir(dir);
            let _ = cmd.output().map_err(|e| {
                eprintln!("{dir:?} > {e:?}");
                exit(1)
            });
        } else {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_dir() {
                            do_clean(&path, cmd);
                        }
                    }
                }
            }
        }
    }
}
