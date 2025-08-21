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

pub struct Cmd<'a> {
    pub name: &'a str,
    pub related_files: Vec<&'a str>,
}

impl<'a> Cmd<'a> {
    pub fn new(cmd_str: &'a str, related_files: Vec<&'a str>) -> Self {
        Self {
            name: cmd_str,
            related_files,
        }
    }

    pub async fn run_clean(&self, dir: &Path) -> Result<(), CleanError> {
        if self.name == "nodejs" {
            return self.clean_nodejs_project(dir).await;
        }

        let mut command = Command::new(self.name);
        #[cfg(target_os = "windows")]
        {
            if self.name == "flutter" {
                command = Command::new("flutter.bat");
            }
        }
        command.arg("clean");
        command.current_dir(dir);

        command.output().await.map(|_| ()).map_err(|source| CleanError::CommandExecutionFailed {
            command: format!("{} clean", self.name),
            path: dir.display().to_string(),
            source,
        })
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constant::get_cmd_map;
    use crate::utils::command_exists;

    #[test]
    fn test_cmd_creation() {
        let cmd = Cmd::new("cargo", vec!["Cargo.toml"]);
        assert_eq!(cmd.name, "cargo");
        assert_eq!(cmd.related_files, vec!["Cargo.toml"]);
    }

    #[test]
    fn test_cmd_list_initialization() {
        let map = get_cmd_map();
        let cmd_list: Vec<_> = map
            .iter()
            .filter(|(key, _)| command_exists(key))
            .map(|(key, value)| Cmd::new(key, value.clone()))
            .collect();

        // Depending on the test environment, the number of available commands may vary.
        // We expect at least 'cargo' to be present.
        assert!(!cmd_list.is_empty());
        assert!(cmd_list.iter().any(|cmd| cmd.name == "cargo"));
    }
}
