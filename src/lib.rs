pub mod cmd;
pub mod config;
pub mod constant;
pub mod utils;


use crate::cmd::Cmd;
use crate::constant::EXCLUDE_DIR;
use colored::*;
use futures::future;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::sync::Arc;
use tokio::{fs, sync::Semaphore};
use walkdir::WalkDir;

// 安全配置常量
pub const MAX_DIRECTORY_DEPTH: usize = 50;
pub const MAX_FILES_PER_PROJECT: usize = 10_000;

async fn get_dir_size_async(path: &Path) -> u64 {
    use std::collections::VecDeque;

    let mut total_size = 0;
    let mut file_count = 0;
    let mut dirs_to_visit = VecDeque::new();

    if path.exists() {
        dirs_to_visit.push_back((path.to_path_buf(), 0)); // (path, depth)

        while let Some((current_dir, depth)) = dirs_to_visit.pop_front() {
            // 检查目录深度限制
            if depth > MAX_DIRECTORY_DEPTH {
                eprintln!("{} Warning: Maximum directory depth ({}) exceeded for {}. Size calculation might be incomplete.",
                         "SKIP".yellow(), MAX_DIRECTORY_DEPTH, current_dir.display());
                continue;
            }

            if let Ok(mut entries) = fs::read_dir(&current_dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    // 检查文件数量限制
                    if file_count > MAX_FILES_PER_PROJECT {
                        eprintln!("{} Warning: Maximum file count ({}) exceeded for {}. Size calculation might be incomplete.",
                                 "SKIP".yellow(), MAX_FILES_PER_PROJECT, current_dir.display());
                        return total_size;
                    }

                    if let Ok(metadata) = entry.metadata().await {
                        if metadata.is_file() {
                            total_size += metadata.len();
                            file_count += 1;
                        } else if metadata.is_dir() {
                            dirs_to_visit.push_back((entry.path(), depth + 1));
                        }
                    }
                }
            }
        }
    }

    total_size
}

// 获取CPU逻辑核心数
pub fn get_cpu_core_count() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4) // 默认4个核心
}

pub async fn do_clean_all(dir: &Path, commands: &Vec<Cmd<'_>>, exclude_dirs: &Vec<String>, max_concurrent: Option<usize>) -> u32 {
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
                if EXCLUDE_DIR.contains(&dir_name) || dir_name.starts_with('.') || exclude_dirs.contains(&dir_name.to_string()) {
                    return None;
                }
            }

            let mut tasks_for_dir = vec![];
            for cmd in commands.iter() {
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
    let pb = Arc::new(ProgressBar::new(total_tasks as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .expect("Failed to set progress template")
            .progress_chars("#>-"),
    );

    pb.set_message("Scanning projects...");

    // 使用配置的并发限制或默认值
    let max_concurrent_limit = max_concurrent.unwrap_or_else(get_cpu_core_count);
    let semaphore = Arc::new(Semaphore::new(max_concurrent_limit));
    
    // 并行计算所有项目的初始大小（带并发限制）
    let size_futures: Vec<_> = cleaning_tasks
        .iter()
        .map(|(path, _)| {
            let semaphore = Arc::clone(&semaphore);
            async move {
                let _permit = semaphore.acquire().await.unwrap();
                get_dir_size_async(path).await
            }
        })
        .collect();

    let sizes_before = future::join_all(size_futures).await;
    let total_size_before: u64 = sizes_before.iter().sum();

    if total_size_before > 0 {
        pb.set_message(format!(
            "Total cache size: {}",
            format_size(total_size_before)
        ));
    }

    // 准备并行执行的任务（带并发限制）
    let cleaning_futures: Vec<_> = cleaning_tasks
        .into_iter()
        .zip(sizes_before.into_iter())
        .map(|((path, cmd_name), size_before)| {
            let pb = Arc::clone(&pb);
            let semaphore = Arc::clone(&semaphore);

            async move {
                let _permit = semaphore.acquire().await.unwrap();
                pb.inc(1);
                pb.set_message(format!("Cleaning {} ({})", path.display(), cmd_name));

                let cmd = commands.iter().find(|c| c.name == cmd_name).unwrap();
                match cmd.run_clean(&path).await {
                    Ok(_) => {
                        let size_after = get_dir_size_async(&path).await;
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
                        (1, size_before, size_after)
                    }
                    Err(e) => {
                        pb.println(format!(
                            "✗ {} {} - {} (Error: {})",
                            "Failed".red(),
                            path.display(),
                            cmd_name,
                            e
                        ));
                        (0, size_before, 0)
                    }
                }
            }
        })
        .collect();

    // 并行执行所有清理任务
    let results = future::join_all(cleaning_futures).await;

    pb.finish_with_message("Cleaning complete!");

    // 计算总结果
    let total_cleaned: u32 = results.iter().map(|(count, _, _)| count).sum();
    let total_size_after: u64 = results.iter().map(|(_, _, after)| after).sum();
    let total_freed = total_size_before.saturating_sub(total_size_after);

    if total_size_before > 0 {
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
