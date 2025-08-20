pub mod constant;
pub mod cmd;
pub mod utils;

use crate::cmd::Cmd;
use crate::constant::EXCLUDE_DIR;
use colored::*;
use futures::future;
use std::path::Path;
use walkdir::WalkDir;

pub async fn do_clean_all(dir: &Path, cmd_list: &Vec<Cmd<'_>>) -> u32 {
    let entries: Vec<_> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .collect();

    let cleaning_futures: Vec<_> = entries
        .iter()
        .filter_map(|entry| {
            let path = entry.path();
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                if EXCLUDE_DIR.contains(&dir_name) || dir_name.starts_with('.') {
                    return None;
                }
            }

            let mut futures_for_dir = vec![];
            for cmd in cmd_list.iter() {
                if cmd
                    .related_files
                    .iter()
                    .any(|file| path.join(file).exists())
                {
                    let path_buf = path.to_path_buf();
                    let future = async move {
                        println!(
                            "{} {} {}",
                            "Cleaning".bright_black(),
                            path_buf.display(),
                            format!("({})", cmd.name).blue()
                        );
                        if cmd.run_clean(&path_buf).await.is_ok() {
                            1
                        } else {
                            0
                        }
                    };
                    futures_for_dir.push(future);
                }
            }
            Some(futures_for_dir)
        })
        .flatten()
        .collect();

    let results = future::join_all(cleaning_futures).await;
    results.into_iter().sum()
}
