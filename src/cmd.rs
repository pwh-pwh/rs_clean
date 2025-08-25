use std::io;
use std::path::Path;
use tokio::fs;
use tokio::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CleanError {
    #[error("Failed to execute command '{command}' in '{path}': {source}")]
    CommandExecutionFailed {
        command: String,
        path: String,
        #[source]
        source: io::Error,
    },
    #[error("Failed to remove directory '{path}': {source}")]
    DirectoryRemovalFailed {
        path: String,
        #[source]
        source: io::Error,
    },
    #[error("Unknown error: {0}")]
    Unknown(#[from] io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandType {
    Cargo,
    Go,
    Gradle,
    NodeJs,
    Flutter,
    Python,
    Maven,
    MavenCmd, // For Windows specific mvn.cmd
}

impl CommandType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CommandType::Cargo => "cargo",
            CommandType::Go => "go",
            CommandType::Gradle => "gradle",
            CommandType::NodeJs => "nodejs",
            CommandType::Flutter => "flutter",
            CommandType::Python => "python",
            CommandType::Maven => "mvn",
            CommandType::MavenCmd => "mvn.cmd",
        }
    }
}

impl From<&str> for CommandType {
    fn from(s: &str) -> Self {
        match s {
            "cargo" => CommandType::Cargo,
            "go" => CommandType::Go,
            "gradle" => CommandType::Gradle,
            "nodejs" => CommandType::NodeJs,
            "flutter" => CommandType::Flutter,
            "python" => CommandType::Python,
            "mvn" => CommandType::Maven,
            "mvn.cmd" => CommandType::MavenCmd,
            _ => panic!("Unknown command type: {}", s), // Should not happen with validated input
        }
    }
}

pub struct Cmd {
    pub command_type: CommandType,
    pub related_files: Vec<&'static str>,
}

impl Cmd {
    pub fn new(command_type: CommandType, related_files: Vec<&'static str>) -> Self {
        Self {
            command_type,
            related_files,
        }
    }

    pub async fn run_clean(&self, dir: &Path) -> Result<(), CleanError> {
        match self.command_type {
            CommandType::NodeJs => self.clean_nodejs_project(dir).await,
            CommandType::Python => self.clean_python_project(dir).await,
            _ => {
                let cmd_name = self.command_type.as_str();
                let mut command = Command::new(cmd_name);

                #[cfg(target_os = "windows")]
                {
                    if self.command_type == CommandType::Flutter {
                        command = Command::new("flutter.bat");
                    }
                }
                command.arg("clean");
                command.current_dir(dir);

                command.output().await.map(|_| ()).map_err(|source| CleanError::CommandExecutionFailed {
                    command: format!("{} clean", cmd_name),
                    path: dir.display().to_string(),
                    source,
                })
            }
        }
    }

    async fn clean_nodejs_project(&self, dir: &Path) -> Result<(), CleanError> {
        let common_node_dirs = vec![
            "node_modules",
            "dist",
            "build",
            ".next", // Next.js build output
            "out",   // Common build output or Parcel
            "coverage", // Test coverage reports
            ".cache", // General cache directory
        ];

        for sub_dir_name in common_node_dirs {
            let path_to_clean = dir.join(sub_dir_name);
            self.remove_dir_if_exists(&path_to_clean).await?;
        }
        Ok(())
    }

    async fn remove_dir_if_exists(&self, path: &Path) -> Result<(), CleanError> {
        if path.exists() {
            fs::remove_dir_all(path).await.map_err(|source| CleanError::DirectoryRemovalFailed {
                path: path.display().to_string(),
                source,
            })?;
        }
        Ok(())
    }

    async fn clean_python_project(&self, dir: &Path) -> Result<(), CleanError> {
        let common_python_dirs = vec![
            "__pycache__",
            "build",
            "dist",
            ".eggs",
            "*.egg-info", // This is a glob pattern, needs special handling or direct removal if possible
            ".pytest_cache",
            "htmlcov",
            ".mypy_cache",
            "venv", // Common virtual environment name
            ".venv", // Common virtual environment name
        ];

        for sub_dir_name in common_python_dirs {
            // For glob patterns like "*.egg-info", we need to list and remove
            if sub_dir_name.contains('*') {
                let pattern = dir.join(sub_dir_name).to_string_lossy().into_owned();
                for entry in glob::glob(&pattern).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))? {
                    if let Ok(path) = entry {
                        if path.is_dir() {
                            self.remove_dir_if_exists(&path).await?;
                        }
                    }
                }
            } else {
                let path_to_clean = dir.join(sub_dir_name);
                self.remove_dir_if_exists(&path_to_clean).await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constant::get_cmd_map;
    use crate::utils::command_exists;

    #[test]
    fn test_cmd_creation() {
        let cmd = Cmd::new(CommandType::Cargo, vec!["Cargo.toml"]);
        assert_eq!(cmd.command_type, CommandType::Cargo);
        assert_eq!(cmd.related_files, vec!["Cargo.toml"]);
    }

    #[test]
    fn test_cmd_list_initialization() {
        let map = get_cmd_map();
        let cmd_list: Vec<_> = map
            .iter()
            .filter(|(key, _)| command_exists(key.as_str()))
            .map(|(key, value)| Cmd::new(*key, value.clone()))
            .collect();

        // Depending on the test environment, the number of available commands may vary.
        // We expect at least 'cargo' to be present.
        assert!(!cmd_list.is_empty());
        assert!(cmd_list.iter().any(|cmd| cmd.command_type == CommandType::Cargo));
    }
}
