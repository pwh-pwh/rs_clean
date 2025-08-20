use std::collections::HashMap;
use std::sync::OnceLock;

pub static EXCLUDE_DIR: &[&str] = &[
    "node_modules",
    "target",
    "build",
    "dist",
    "bin",
    "pkg",
    "src",
    "tests",
    "test",
];

// ANSI color codes for terminal output
pub const COLOR_BLUE: &str = "\x1B[34m";
pub const COLOR_RED: &str = "\x1B[31m";
pub const COLOR_GREEN: &str = "\x1B[32m";
pub const COLOR_GRAY: &str = "\x1B[90m";
pub const COLOR_RESET: &str = "\x1B[0m";

static CMD_MAP: OnceLock<HashMap<&'static str, Vec<&'static str>>> = OnceLock::new();

pub fn get_cmd_map() -> &'static HashMap<&'static str, Vec<&'static str>> {
    CMD_MAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("cargo", vec!["Cargo.toml"]);
        m.insert("go", vec!["go.mod"]);
        m.insert("gradle", vec!["build.gradle","build.gradle.kts"]);
        m.insert("nodejs", vec!["package.json"]); // 统一使用 nodejs 标识符
        #[cfg(not(target_os = "windows"))]
        {
            m.insert("mvn", vec!["pom.xml"]);
        }
        #[cfg(target_os = "windows")]
        {
            m.insert("mvn.cmd", vec!["pom.xml"]);
        }
        m
    })
}

// 定义需要特殊处理的命令（不执行 clean 子命令，而是直接删除文件/文件夹）
pub fn get_special_clean_commands() -> &'static [&'static str] {
    &["nodejs"]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_cmd_map() {
        let map = get_cmd_map();
        
        // 测试 Rust 命令
        assert_eq!(map.get("cargo"), Some(&vec!["Cargo.toml"]));
        
        // 测试 Go 命令
        assert_eq!(map.get("go"), Some(&vec!["go.mod"]));
        
        // 测试 Gradle 命令
        assert_eq!(map.get("gradle"), Some(&vec!["build.gradle", "build.gradle.kts"]));
        
        // 测试 Node.js 命令
        assert_eq!(map.get("nodejs"), Some(&vec!["package.json"]));
        
        // 测试 Maven 命令（平台相关）
        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!(map.get("mvn"), Some(&vec!["pom.xml"]));
        }
        #[cfg(target_os = "windows")]
        {
            assert_eq!(map.get("mvn.cmd"), Some(&vec!["pom.xml"]));
        }
        
        // 验证总数
        assert_eq!(map.len(), 5);
    }
}