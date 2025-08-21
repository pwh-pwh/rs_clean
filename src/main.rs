use clap::Parser;
use colored::*;
use rs_clean::cmd::Cmd;
use rs_clean::config::Config;
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
    path: Option<PathBuf>,

    /// Exclude certain project types from cleaning.
    #[arg(short = 't', long = "exclude-type", value_name = "TYPE")]
    exclude_types: Vec<String>,

    /// Exclude specific directory names from cleaning.
    #[arg(short = 'd', long = "exclude-dir", value_name = "DIR_NAME")]
    exclude_dirs: Vec<String>,

    /// Configuration file path (optional)
    #[arg(short = 'c', long = "config", value_name = "FILE")]
    config: Option<PathBuf>,

    /// Show detailed output
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let start = Instant::now();

    // Load configuration
    let config = if let Some(config_path) = &cli.config {
        match Config::from_file(config_path).await {
            Ok(config) => config,
            Err(e) => {
                eprintln!("{} Failed to load config file: {}", "Error:".red(), e);
                std::process::exit(1);
            }
        }
    } else {
        match Config::load().await {
            Ok(config) => config,
            Err(e) => {
                eprintln!("{} Failed to load configuration: {}", "Error:".red(), e);
                std::process::exit(1);
            }
        }
    };

    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("{} Invalid configuration: {}", "Error:".red(), e);
        std::process::exit(1);
    }

    // Merge configuration with CLI arguments
    let merged_config = config.merge_with_cli(&cli.path, &cli.exclude_types, &cli.exclude_dirs);

    if cli.verbose || merged_config.verbose {
        println!("{} Using configuration:", "Info:".blue());
        println!("  Path: {}", merged_config.path.display());
        if !merged_config.exclude_types.is_empty() {
            println!("  Exclude types: {}", merged_config.exclude_types.join(", "));
        }
        if !merged_config.exclude_dirs.is_empty() {
            println!("  Exclude dirs: {}", merged_config.exclude_dirs.join(", "));
        }
        if let Some(max_concurrent) = merged_config.max_concurrent {
            println!("  Max concurrent: {}", max_concurrent);
        }
        println!();
    }

    let map = get_cmd_map();
    let mut cmd_list = vec![];
    for (key, value) in map {
        if command_exists(key) && !merged_config.exclude_types.contains(&key.to_string()) {
            cmd_list.push(Cmd::new(key, value.clone()));
        }
    }

    let init_cmd: Vec<String> = cmd_list.iter().map(|cmd| cmd.name.to_string()).collect();
    println!(
        "Found supported clean commands: {}",
        init_cmd.join(", ").blue()
    );
    
    // 显示并发限制和安全信息
    let cpu_cores = merged_config.max_concurrent.unwrap_or_else(get_cpu_core_count);
    println!(
        "Using {} concurrent worker{} (CPU cores: {})",
        cpu_cores,
        if cpu_cores > 1 { "s" } else { "" },
        get_cpu_core_count()
    );
    let max_depth = merged_config.max_depth.unwrap_or(MAX_DIRECTORY_DEPTH);
    let max_files = merged_config.max_files.unwrap_or(MAX_FILES_PER_PROJECT);
    println!(
        "Safety limits: max depth {}, max files {}",
        max_depth,
        max_files
    );

    let count = do_clean_all(&merged_config.path, &cmd_list, &merged_config.exclude_dirs).await;
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
