use std::io;
use std::path::Path;
use tokio::fs;
use tokio::process::Command;

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

    pub async fn run_clean(&self, dir: &Path) -> io::Result<()> {
        if self.name == "nodejs" {
            return self.clean_nodejs_project(dir).await;
        }

        let mut command = Command::new(self.name);
        command.arg("clean");
        command.current_dir(dir);

        command.output().await.map(|_| ()).map_err(|e| {
            eprintln!(
                "Failed to execute '{} clean' in {}: {}",
                self.name,
                dir.display(),
                e
            );
            e
        })
    }

    async fn clean_nodejs_project(&self, dir: &Path) -> io::Result<()> {
        let node_modules = dir.join("node_modules");
        if node_modules.exists() {
            fs::remove_dir_all(&node_modules).await?;
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
