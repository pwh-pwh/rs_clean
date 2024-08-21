mod constant;
mod cmd;

use crate::constant::EXCLUDE_DIR;
use std::fs;
use std::path::Path;
use std::process::{exit, Command};

pub fn command_exists(cmd: &str) -> bool {
    Command::new(cmd).args(["--version"]).output().is_ok()
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
