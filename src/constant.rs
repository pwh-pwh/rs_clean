use std::collections::HashMap;
use std::sync::OnceLock;
use crate::cmd::CommandType; // 引入 CommandType

pub const DEFAULT_MAX_DIRECTORY_DEPTH: usize = 5;
pub const DEFAULT_MAX_FILES_PER_PROJECT: usize = 10000;


static CMD_MAP: OnceLock<HashMap<CommandType, Vec<&'static str>>> = OnceLock::new();

pub fn get_cmd_map() -> &'static HashMap<CommandType, Vec<&'static str>> {
    CMD_MAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert(CommandType::Cargo, vec!["Cargo.toml"]);
        m.insert(CommandType::Go, vec!["go.mod"]);
        m.insert(CommandType::Gradle, vec!["build.gradle", "build.gradle.kts"]);
        m.insert(CommandType::NodeJs, vec!["package.json"]); // 统一使用 nodejs 标识符
        m.insert(CommandType::Flutter, vec!["pubspec.yaml"]);
        m.insert(CommandType::Python, vec!["requirements.txt", "pyproject.toml"]); // Python projects
        #[cfg(not(target_os = "windows"))]
        {
            m.insert(CommandType::Maven, vec!["pom.xml"]);
        }
        #[cfg(target_os = "windows")]
        {
            m.insert(CommandType::MavenCmd, vec!["pom.xml"]);
        }
        m
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_constants() {
        assert_eq!(DEFAULT_MAX_DIRECTORY_DEPTH, 5);
        assert_eq!(DEFAULT_MAX_FILES_PER_PROJECT, 10000);
    }

    #[test]
    fn test_get_cmd_map() {
        let map = get_cmd_map();

        // 测试 Rust 命令
        assert_eq!(map.get(&CommandType::Cargo), Some(&vec!["Cargo.toml"]));

        // 测试 Go 命令
        assert_eq!(map.get(&CommandType::Go), Some(&vec!["go.mod"]));

        // 测试 Gradle 命令
        assert_eq!(
            map.get(&CommandType::Gradle),
            Some(&vec!["build.gradle", "build.gradle.kts"])
        );

        // 测试 Node.js 命令
        assert_eq!(map.get(&CommandType::NodeJs), Some(&vec!["package.json"]));

        // 测试 Maven 命令（平台相关）
        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!(map.get(&CommandType::Maven), Some(&vec!["pom.xml"]));
        }
        #[cfg(target_os = "windows")]
        {
            assert_eq!(map.get(&CommandType::MavenCmd), Some(&vec!["pom.xml"]));
        }

        // 验证总数
        assert_eq!(map.len(), 7);
    }
}
