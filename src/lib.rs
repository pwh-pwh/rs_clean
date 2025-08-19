pub mod constant;
pub mod cmd;
pub mod utils;

use crate::constant::{EXCLUDE_DIR, COLOR_RED, COLOR_GRAY, COLOR_RESET};
use std::fs;
use std::path::Path;
use std::process::{exit, Command};


pub fn do_clean_all(dir: &Path,cmd_list: &mut Vec<cmd::Cmd>) -> u32 {
    let mut count = 0;
    if dir.is_dir() {
        if let Some(dir_name) = dir.file_name() {
            // Handle invalid UTF-8 characters safely
            if let Some(dir_str) = dir_name.to_str() {
                if EXCLUDE_DIR.contains(&dir_str) {
                    return count;
                }
                if dir_str.starts_with(".") {
                    return count;
                }
            } else {
                // Skip directories with invalid UTF-8 names
                return count;
            }
        }
        //定义变量flag 记录是否存在符合条件的文件
        let mut flag = false;
        
        // Check for node_modules directory
        let node_modules_path = dir.join("node_modules");
        if node_modules_path.exists() && node_modules_path.is_dir() {
            count += 1;
            flag = true;
            println!("{}remove:{} node_modules{} {}", COLOR_GRAY, COLOR_RESET, COLOR_RED, dir.display());
            if let Err(e) = fs::remove_dir_all(&node_modules_path) {
                eprintln!("{dir:?} > {e:?}");
                exit(1)
            }
        }
        
        cmd_list.iter_mut().for_each(|cmd| {
            cmd.related_files.clone().iter().for_each(|file| {
                if dir.join(file).exists() {
                    count += 1;
                    flag = true;
                    println!("{}run:{} {} clean{} {}", COLOR_GRAY, COLOR_RESET, COLOR_RED, COLOR_RESET, dir.display());
                    
                    // 检查是否为需要特殊处理的命令
                    if cmd.is_special_clean_command() {
                        // 执行特殊清理逻辑
                        if let Err(e) = cmd.run_special_clean(dir) {
                            eprintln!("{dir:?} > {e:?}");
                            exit(1)
                        }
                    } else {
                        // 执行标准的 clean 命令
                        cmd.current_dir(dir);
                        let _ = cmd.run().map_err(|e| {
                            eprintln!("{dir:?} > {e:?}");
                            exit(1)
                        });
                    }
                }
            })
        });
        if !flag {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        count += do_clean_all(&path, cmd_list);
                    }
                }
            }
        }
    }
    count
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
            // Use safe error handling instead of unwrap()
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        do_clean(&path, cmd);
                    }
                }
            }
        }
    }
}
