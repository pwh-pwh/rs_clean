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

        // First check current directory
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
        if let Some(max_concurrent) = self.max_concurrent {
            if max_concurrent == 0 {
                return Err(ConfigError::InvalidConfig(
                    "max_concurrent must be greater than 0".to_string()
                ));
            }
        }

        if let Some(max_depth) = self.max_depth {
            if max_depth == 0 {
                return Err(ConfigError::InvalidConfig(
                    "max_depth must be greater than 0".to_string()
                ));
            }
        }

        if let Some(max_files) = self.max_files {
            if max_files == 0 {
                return Err(ConfigError::InvalidConfig(
                    "max_files must be greater than 0".to_string()
                ));
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