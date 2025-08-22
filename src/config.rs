use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file '{path}': {source}")]
    FileReadError {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to parse config file '{path}': {source}")]
    ParseError {
        path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// Default path to clean (defaults to current directory)
    pub default_path: Option<String>,
    /// Project types to exclude by default
    pub exclude_types: Option<Vec<String>>,
    /// Directory names to exclude by default
    pub exclude_dirs: Option<Vec<String>>,
    /// Maximum concurrent workers (defaults to CPU core count)
    pub max_concurrent: Option<usize>,
    /// Maximum directory depth to scan (defaults to 50)
    pub max_depth: Option<usize>,
    /// Maximum files per project (defaults to 10000)
    pub max_files: Option<usize>,
    /// Whether to show detailed output
    pub verbose: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_path: None,
            exclude_types: None,
            exclude_dirs: None,
            max_concurrent: None,
            max_depth: None,
            max_files: None,
            verbose: None,
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|source| ConfigError::FileReadError {
            path: path.display().to_string(),
            source,
        })?;

        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        
        match extension {
            "json" => {
                serde_json::from_str(&content).map_err(|source| ConfigError::ParseError {
                    path: path.display().to_string(),
                    source: Box::new(source),
                })
            }
            "toml" => {
                toml::from_str(&content).map_err(|source| ConfigError::ParseError {
                    path: path.display().to_string(),
                    source: Box::new(source),
                })
            }
            _ => Err(ConfigError::InvalidConfig(
                format!("Unsupported config file format: {}", extension)
            )),
        }
    }

    /// Find and load configuration from default locations
    pub async fn load() -> Result<Self, ConfigError> {
        let current_dir = std::env::current_dir()
            .map_err(|e| ConfigError::InvalidConfig(
                format!("Failed to get current directory: {}", e)
            ))?;
        
        let config_filenames = vec![
            "rs_clean.json",
            "rs_clean.toml",
            ".rs_clean.json",
            ".rs_clean.toml",
        ];

        // Check user config directory first (highest priority)
        if let Ok(user_config_dir) = Self::get_user_config_dir() {
            for filename in &config_filenames {
                let path = user_config_dir.join(filename);
                if path.exists() {
                    return Self::from_file(&path).await;
                }
            }
        }

        // Then check current directory
        for filename in &config_filenames {
            let path = current_dir.join(filename);
            if path.exists() {
                return Self::from_file(&path).await;
            }
        }

        // Then check config subdirectory
        for filename in &config_filenames {
            let path = current_dir.join("config").join(filename);
            if path.exists() {
                return Self::from_file(&path).await;
            }
        }

        // Return default config if no config file found
        Ok(Self::default())
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate max_concurrent with reasonable bounds
        if let Some(max_concurrent) = self.max_concurrent {
            if max_concurrent == 0 {
                return Err(ConfigError::InvalidConfig(
                    "max_concurrent must be greater than 0".to_string()
                ));
            }
            if max_concurrent > 64 {
                return Err(ConfigError::InvalidConfig(
                    format!("max_concurrent must be <= 64, got {}", max_concurrent)
                ));
            }
        }

        // Validate max_depth with reasonable bounds
        if let Some(max_depth) = self.max_depth {
            if max_depth == 0 {
                return Err(ConfigError::InvalidConfig(
                    "max_depth must be greater than 0".to_string()
                ));
            }
            if max_depth > 1000 {
                return Err(ConfigError::InvalidConfig(
                    format!("max_depth must be <= 1000, got {}", max_depth)
                ));
            }
        }

        // Validate max_files with reasonable bounds
        if let Some(max_files) = self.max_files {
            if max_files == 0 {
                return Err(ConfigError::InvalidConfig(
                    "max_files must be greater than 0".to_string()
                ));
            }
            if max_files > 100000 {
                return Err(ConfigError::InvalidConfig(
                    format!("max_files must be <= 100000, got {}", max_files)
                ));
            }
        }

        // Validate default_path exists if specified
        if let Some(ref path_str) = self.default_path {
            let path = std::path::Path::new(path_str);
            if !path.exists() {
                return Err(ConfigError::InvalidConfig(
                    format!("default_path does not exist: {}", path_str)
                ));
            }
        }

        // Validate exclude_types against known project types
        if let Some(ref exclude_types) = self.exclude_types {
            let valid_types = ["cargo", "go", "gradle", "nodejs", "flutter", "python", "mvn"];
            for exclude_type in exclude_types {
                if !valid_types.contains(&exclude_type.as_str()) {
                    return Err(ConfigError::InvalidConfig(
                        format!("Unknown exclude type: {}. Valid types: {}", exclude_type, valid_types.join(", "))
                    ));
                }
            }
        }

        Ok(())
    }

    /// Merge with CLI arguments (CLI args take precedence)
    pub fn merge_with_cli(&self, cli_path: &Option<PathBuf>, cli_exclude_types: &[String], cli_exclude_dirs: &[String]) -> MergedConfig {
        MergedConfig {
            path: cli_path.clone()
                .or_else(|| self.default_path.as_ref().map(|p| PathBuf::from(p)))
                .unwrap_or_else(|| PathBuf::from(".")),
            exclude_types: if cli_exclude_types.is_empty() {
                self.exclude_types.clone().unwrap_or_default()
            } else {
                cli_exclude_types.to_vec()
            },
            exclude_dirs: if cli_exclude_dirs.is_empty() {
                self.exclude_dirs.clone().unwrap_or_default()
            } else {
                cli_exclude_dirs.to_vec()
            },
            max_concurrent: self.max_concurrent,
            max_depth: self.max_depth,
            max_files: self.max_files,
            verbose: self.verbose.unwrap_or(false),
        }
    }

    /// Get the path to the user's config directory
    /// 
    /// Returns platform-appropriate config directory:
    /// - Windows: %APPDATA%\rs_clean (or %LOCALAPPDATA%\rs_clean)
    /// - Unix-like: ~/.rs_clean
    pub fn get_user_config_dir() -> Result<PathBuf, ConfigError> {
        dirs::home_dir()
            .ok_or_else(|| ConfigError::InvalidConfig(
                "Failed to get home directory".to_string()
            ))
            .map(|home| {
                // Use platform-appropriate config directory
                if cfg!(target_os = "windows") {
                    // On Windows, use AppData\Roaming for better compatibility
                    dirs::data_dir()
                        .unwrap_or_else(|| home.clone())
                        .join("rs_clean")
                } else {
                    // On Unix-like systems, use hidden directory in home
                    home.join(".rs_clean")
                }
            })
    }

    /// Get the path to the user's config file
    pub fn get_user_config_path() -> Result<PathBuf, ConfigError> {
        Self::get_user_config_dir().map(|dir| dir.join("rs_clean.toml"))
    }

    /// Initialize a default config file in the user's home directory
    pub async fn init_user_config() -> Result<PathBuf, ConfigError> {
        let config_path = Self::get_user_config_path()?;
        let config_dir = config_path.parent().unwrap();

        // Create config directory if it doesn't exist with better error handling
        if let Err(e) = fs::create_dir_all(config_dir) {
            return Err(ConfigError::FileReadError {
                path: config_dir.display().to_string(),
                source: e,
            });
        }

        // Create default config
        let default_config = Self::default();
        let toml_content = toml::to_string_pretty(&default_config).map_err(|source| ConfigError::ParseError {
            path: config_path.display().to_string(),
            source: Box::new(source),
        })?;

        // Write config file with Windows-compatible handling
        if let Err(e) = fs::write(&config_path, toml_content) {
            return Err(ConfigError::FileReadError {
                path: config_path.display().to_string(),
                source: e,
            });
        }

        Ok(config_path)
    }

    /// Set a configuration value in the user's config file
    pub async fn set_user_config_value(key: &str, value: &str) -> Result<(), ConfigError> {
        let config_path = Self::get_user_config_path()?;
        
        // Load existing config or create default
        let mut config = if config_path.exists() {
            Self::from_file(&config_path).await?
        } else {
            Self::default()
        };

        // Set the value based on key
        match key {
            "default_path" => config.default_path = Some(value.to_string()),
            "exclude_types" => {
                let types: Vec<String> = value.split(',').map(|s| s.trim().to_string()).collect();
                config.exclude_types = Some(types);
            },
            "exclude_dirs" => {
                let dirs: Vec<String> = value.split(',').map(|s| s.trim().to_string()).collect();
                config.exclude_dirs = Some(dirs);
            },
            "max_concurrent" => {
                let max_concurrent = value.parse().map_err(|_| ConfigError::InvalidConfig(
                    format!("Invalid value for max_concurrent: {}", value)
                ))?;
                config.max_concurrent = Some(max_concurrent);
            },
            "max_depth" => {
                let max_depth = value.parse().map_err(|_| ConfigError::InvalidConfig(
                    format!("Invalid value for max_depth: {}", value)
                ))?;
                config.max_depth = Some(max_depth);
            },
            "max_files" => {
                let max_files = value.parse().map_err(|_| ConfigError::InvalidConfig(
                    format!("Invalid value for max_files: {}", value)
                ))?;
                config.max_files = Some(max_files);
            },
            "verbose" => {
                let verbose = value.parse().map_err(|_| ConfigError::InvalidConfig(
                    format!("Invalid value for verbose: {}", value)
                ))?;
                config.verbose = Some(verbose);
            },
            _ => return Err(ConfigError::InvalidConfig(
                format!("Unknown configuration key: {}", key)
            )),
        }

        // Save the updated config
        let toml_content = toml::to_string_pretty(&config).map_err(|source| ConfigError::ParseError {
            path: config_path.display().to_string(),
            source: Box::new(source),
        })?;

        fs::write(&config_path, toml_content).map_err(|source| ConfigError::FileReadError {
            path: config_path.display().to_string(),
            source,
        })?;

        Ok(())
    }

    /// Get current user configuration
    pub async fn get_user_config() -> Result<Self, ConfigError> {
        let config_path = Self::get_user_config_path()?;
        if config_path.exists() {
            Self::from_file(&config_path).await
        } else {
            Ok(Self::default())
        }
    }
}

/// Merged configuration combining config file and CLI arguments
#[derive(Debug, Clone)]
pub struct MergedConfig {
    pub path: PathBuf,
    pub exclude_types: Vec<String>,
    pub exclude_dirs: Vec<String>,
    pub max_concurrent: Option<usize>,
    pub max_depth: Option<usize>,
    pub max_files: Option<usize>,
    pub verbose: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_load_json_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
        {
            "default_path": "/test/path",
            "exclude_types": ["cargo", "go"],
            "exclude_dirs": ["node_modules"],
            "max_concurrent": 8,
            "verbose": true
        }
        "#;
        
        temp_file.write_all(config_content.as_bytes()).unwrap();
        
        // Create a new path with .json extension
        let json_path = temp_file.path().with_extension("json");
        fs::rename(temp_file.path(), &json_path).unwrap();
        
        let config = Config::from_file(&json_path).await.unwrap();
        
        assert_eq!(config.default_path, Some("/test/path".to_string()));
        assert_eq!(config.exclude_types, Some(vec!["cargo".to_string(), "go".to_string()]));
        assert_eq!(config.exclude_dirs, Some(vec!["node_modules".to_string()]));
        assert_eq!(config.max_concurrent, Some(8));
        assert_eq!(config.verbose, Some(true));
    }

    #[tokio::test]
    async fn test_load_toml_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
        default_path = "/test/path"
        exclude_types = ["cargo", "go"]
        exclude_dirs = ["node_modules"]
        max_concurrent = 8
        verbose = true
        "#;
        
        temp_file.write_all(config_content.as_bytes()).unwrap();
        
        // Create a new path with .toml extension
        let toml_path = temp_file.path().with_extension("toml");
        fs::rename(temp_file.path(), &toml_path).unwrap();
        
        let config = Config::from_file(&toml_path).await.unwrap();
        
        assert_eq!(config.default_path, Some("/test/path".to_string()));
        assert_eq!(config.exclude_types, Some(vec!["cargo".to_string(), "go".to_string()]));
        assert_eq!(config.exclude_dirs, Some(vec!["node_modules".to_string()]));
        assert_eq!(config.max_concurrent, Some(8));
        assert_eq!(config.verbose, Some(true));
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        
        // Valid config should pass
        assert!(config.validate().is_ok());
        
        // Invalid max_concurrent should fail
        config.max_concurrent = Some(0);
        assert!(config.validate().is_err());
        
        // Reset and test max_depth
        config.max_concurrent = Some(8);
        config.max_depth = Some(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_merge_with_cli() {
        let config = Config {
            default_path: Some("/config/path".to_string()),
            exclude_types: Some(vec!["cargo".to_string(), "go".to_string()]),
            exclude_dirs: Some(vec!["node_modules".to_string()]),
            max_concurrent: Some(8),
            max_depth: Some(50),
            max_files: Some(10000),
            verbose: Some(true),
        };
        
        // Test with CLI args - CLI should take precedence
        let cli_path = Some(PathBuf::from("/cli/path"));
        let cli_exclude_types = vec!["python".to_string()];
        let cli_exclude_dirs = vec!["target".to_string()];
        
        let merged = config.merge_with_cli(&cli_path, &cli_exclude_types, &cli_exclude_dirs);
        
        assert_eq!(merged.path, PathBuf::from("/cli/path"));
        assert_eq!(merged.exclude_types, vec!["python".to_string()]);
        assert_eq!(merged.exclude_dirs, vec!["target".to_string()]);
        assert_eq!(merged.max_concurrent, Some(8));
        assert_eq!(merged.verbose, true);
    }
}