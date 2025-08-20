pub mod cmd;
pub mod constant;
pub mod utils;

use crate::cmd::Cmd;
use crate::constant::EXCLUDE_DIR;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn get_dir_size(path: &Path) -> u64 {
    let mut total_size = 0;
    if path.exists() {
        if let Ok(metadata) = fs::metadata(path) {
            if metadata.is_dir() {
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries.flatten() {
                        if let Ok(metadata) = entry.metadata() {
                            if metadata.is_file() {
                                total_size += metadata.len();
                            } else if metadata.is_dir() {
                                total_size += get_dir_size(&entry.path());
                            }
                        }
                    }
                }
            } else {
                total_size += metadata.len();
            }
        }
    }
    total_size
}

pub async fn do_clean_all(dir: &Path, cmd_list: &Vec<Cmd<'_>>) -> u32 {
    let entries: Vec<_> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .collect();

    let cleaning_tasks: Vec<_> = entries
        .iter()
        .filter_map(|entry| {
            let path = entry.path();
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                if EXCLUDE_DIR.contains(&dir_name) || dir_name.starts_with('.') {
                    return None;
                }
            }

            let mut tasks_for_dir = vec![];
            for cmd in cmd_list.iter() {
                if cmd
                    .related_files
                    .iter()
                    .any(|file| path.join(file).exists())
                {
                    tasks_for_dir.push((path.to_path_buf(), cmd.name));
                }
            }
            if tasks_for_dir.is_empty() {
                None
            } else {
                Some(tasks_for_dir)
            }
        })
        .flatten()
        .collect();

    if cleaning_tasks.is_empty() {
        println!("{}", "No projects found to clean".yellow());
        return 0;
    }

    let total_tasks = cleaning_tasks.len();
    let pb = ProgressBar::new(total_tasks as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .expect("Failed to set progress template")
            .progress_chars("#>-"),
    );

    pb.set_message("Scanning projects...");

    let mut total_size_before = 0;
    let mut project_info = Vec::new();

    for (path, cmd_name) in &cleaning_tasks {
        let size_before = get_dir_size(path);
        total_size_before += size_before;
        project_info.push((path.clone(), cmd_name.to_string(), size_before));
    }

    if total_size_before > 0 {
        pb.set_message(format!(
            "Total cache size: {}",
            format_size(total_size_before)
        ));
    }

    let mut results = vec![];
    for (path, cmd_name, size_before) in project_info {
        pb.inc(1);
        pb.set_message(format!("Cleaning {} ({})", path.display(), cmd_name));

        let cmd = cmd_list.iter().find(|c| c.name == cmd_name).unwrap();
        let success = cmd.run_clean(&path).await.is_ok();

        if success {
            let size_after = get_dir_size(&path);
            let cleaned_size = size_before.saturating_sub(size_after);
            if cleaned_size > 0 {
                pb.println(format!(
                    "✓ {} {} - {}",
                    "Cleaned".green(),
                    path.display(),
                    format_size(cleaned_size).cyan()
                ));
            } else {
                pb.println(format!(
                    "✓ {} {} - {}",
                    "Cleaned".green(),
                    path.display(),
                    "No files removed".yellow()
                ));
            }
            results.push(1);
        } else {
            pb.println(format!(
                "✗ {} {} - {}",
                "Failed".red(),
                path.display(),
                cmd_name
            ));
            results.push(0);
        }
    }

    pb.finish_with_message("Cleaning complete!");

    let total_cleaned = results.iter().sum::<u32>();
    if total_size_before > 0 {
        let total_size_after = cleaning_tasks
            .iter()
            .map(|(path, _)| get_dir_size(path))
            .sum::<u64>();
        let total_freed = total_size_before.saturating_sub(total_size_after);

        println!(
            "Total space freed: {}",
            format_size(total_freed).green().bold()
        );
    }

    total_cleaned
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}
