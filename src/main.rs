use clap::Parser;
use colored::*;
use rs_clean::cmd::Cmd;
use rs_clean::constant::get_cmd_map;
use rs_clean::do_clean_all;
use rs_clean::utils::command_exists;
use rs_clean::{get_cpu_core_count, MAX_DIRECTORY_DEPTH, MAX_FILES_PER_PROJECT};
use std::path::PathBuf;
use std::time::Instant;

/// A fast and simple tool to clean build artifacts from various projects.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to the directory to clean. Defaults to the current directory.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Exclude certain project types from cleaning.
    #[arg(short = 't', long = "exclude-type", value_name = "TYPE")]
    exclude_types: Vec<String>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let start = Instant::now();

    let map = get_cmd_map();
    let mut cmd_list = vec![];
    for (key, value) in map {
        if command_exists(key) && !cli.exclude_types.contains(&key.to_string()) {
            cmd_list.push(Cmd::new(key, value.clone()));
        }
    }

    let init_cmd: Vec<String> = cmd_list.iter().map(|cmd| cmd.name.to_string()).collect();
    println!(
        "Found supported clean commands: {}",
        init_cmd.join(", ").blue()
    );
    
    // 显示并发限制和安全信息
    let cpu_cores = get_cpu_core_count();
    println!(
        "Using {} concurrent worker{} (CPU cores: {})",
        cpu_cores,
        if cpu_cores > 1 { "s" } else { "" },
        cpu_cores
    );
    println!(
        "Safety limits: max depth {}, max files {}",
        MAX_DIRECTORY_DEPTH,
        MAX_FILES_PER_PROJECT
    );

    let count = do_clean_all(&cli.path, &cmd_list).await;
    let elapsed = start.elapsed();

    println!(
        "\n{}",
        format!(
            "rs_clean cleaned {} packages in {:.2} seconds",
            count,
            elapsed.as_secs_f64()
        )
        .green()
    );
}
