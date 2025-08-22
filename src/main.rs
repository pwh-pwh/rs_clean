use clap::{Parser, Subcommand};
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

    /// Configuration management commands
    #[command(subcommand)]
    command: Option<ConfigCommand>,
}

#[derive(Subcommand, Debug)]
enum ConfigCommand {
    /// Initialize a default configuration file
    Init,
    /// Set a configuration value
    Set {
        /// Configuration key to set
        key: String,
        /// Configuration value
        value: String,
    },
    /// Show current configuration
    Show,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Handle config commands
    if let Some(command) = cli.command {
        match command {
            ConfigCommand::Init => {
                match Config::init_user_config().await {
                    Ok(path) => {
                        println!("{} Configuration file created at: {}", "Success:".green(), path.display());
                        println!("You can now edit this file to customize your settings.");
                    }
                    Err(e) => {
                        eprintln!("{} Failed to create config file: {}", "Error:".red(), e);
                        std::process::exit(1);
                    }
                }
            }
            ConfigCommand::Set { key, value } => {
                match Config::set_user_config_value(&key, &value).await {
                    Ok(()) => {
                        println!("{} Configuration updated: {} = {}", "Success:".green(), key, value);
                    }
                    Err(e) => {
                        eprintln!("{} Failed to set configuration: {}", "Error:".red(), e);
                        std::process::exit(1);
                    }
                }
            }
            ConfigCommand::Show => {
                match Config::get_user_config().await {
                    Ok(config) => {
                        println!("{} Current configuration:", "Info:".blue());
                        println!("  Config file: {}", Config::get_user_config_path().unwrap_or_default().display());
                        if let Some(path) = &config.default_path {
                            println!("  default_path: {}", path);
                        }
                        if let Some(types) = &config.exclude_types {
                            println!("  exclude_types: {}", types.join(", "));
                        }
                        if let Some(dirs) = &config.exclude_dirs {
                            println!("  exclude_dirs: {}", dirs.join(", "));
                        }
                        if let Some(max_concurrent) = &config.max_concurrent {
                            println!("  max_concurrent: {}", max_concurrent);
                        }
                        if let Some(max_depth) = &config.max_depth {
                            println!("  max_depth: {}", max_depth);
                        }
                        if let Some(max_files) = &config.max_files {
                            println!("  max_files: {}", max_files);
                        }
                        if let Some(verbose) = &config.verbose {
                            println!("  verbose: {}", verbose);
                        }
                    }
                    Err(e) => {
                        eprintln!("{} Failed to load configuration: {}", "Error:".red(), e);
                        std::process::exit(1);
                    }
                }
            }
        }
        return;
    }

    // Normal cleaning operation
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
