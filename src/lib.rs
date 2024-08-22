pub mod constant;
pub mod cmd;
pub mod utils;

use crate::constant::EXCLUDE_DIR;
use std::fs;
use std::path::Path;
use std::process::{exit, Command};


pub static mut COUNT: u32 = 0;

pub fn do_clean_all(dir: &Path,cmd_list: &mut Vec<cmd::Cmd>) {
    if dir.is_dir() {
        if let Some(dir_name) = dir.file_name() {
            if EXCLUDE_DIR.contains(&dir_name.to_str().unwrap()) {
                return;
            }
            if dir_name.to_str().unwrap().starts_with(".") {
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
                    println!("\x1B[90mrun:\x1B[0m \x1B[31m {} clean\x1B[0m {}",cmd.name, dir.display());
                    cmd.current_dir(dir);
                    let _ = cmd.run().map_err(|e| {
                        eprintln!("{dir:?} > {e:?}");
                        exit(1)
                    });
                }
            })
        });
        if !flag {
            for entry in fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                    do_clean_all(&path, cmd_list);
                }
            }
        }
    }
}

pub fn do_clean(dir: &Path, cmd: &mut Command) {
    if dir.is_dir() {
        if let Some(dir_name) = dir.file_name() {
            if EXCLUDE_DIR.contains(&dir_name.to_str().unwrap()) {
                return;
            }
            if dir_name.to_str().unwrap().starts_with(".") {
                return;
            }
        }
        let cargo_toml_path = dir.join("Cargo.toml");
        if cargo_toml_path.exists() {
            println!("\x1B[31mclean {}\x1B[0m", dir.display());
            cmd.current_dir(dir);
            let _ = cmd.output().map_err(|e| {
                eprintln!("{dir:?} > {e:?}");
                exit(1)
            });
        } else {
            for entry in fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                    do_clean(&path, cmd);
                }
            }
        }
    }
}
