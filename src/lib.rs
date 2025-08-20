pub mod constant;
pub mod cmd;
pub mod utils;

use colored::Colorize;
use crate::cmd::Cmd;
use crate::constant::EXCLUDE_DIR;
use rayon::prelude::*;
use std::path::Path;
use walkdir::WalkDir;

pub fn do_clean_all(dir: &Path, cmd_list: &Vec<Cmd>) -> u32 {
    let entries: Vec<_> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .collect();

    let count: u32 = entries
        .par_iter()
        .map(|entry| {
            let path = entry.path();
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                if EXCLUDE_DIR.contains(&dir_name) || dir_name.starts_with('.') {
                    return 0;
                }
            }

            let mut cleaned_in_dir = 0;
            for cmd in cmd_list.iter() {
                if cmd
                    .related_files
                    .iter()
                    .any(|file| path.join(file).exists())
                {
                    println!(
                        "{} {} {}",
                        "Cleaning".black(),
                        path.display(),
                        format!("({})", cmd.name).blue()
                    );
                    if cmd.run_clean(path).is_ok() {
                        cleaned_in_dir += 1;
                    }
                }
            }
            cleaned_in_dir
        })
        .sum();

    count
}
