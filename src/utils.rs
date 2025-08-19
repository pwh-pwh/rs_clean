use std::process::Command;

pub fn command_exists(cmd: &str) -> bool {
    Command::new(cmd).args(["--version"]).output().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_exists() {
        // 测试系统中可用的命令
        // cargo 应该总是可用的（因为我们在 Rust 环境中）
        assert!(command_exists("cargo"));
        
        // 其他命令可能不可用，所以只测试它们不会panic
        let _ = command_exists("go");
        let _ = command_exists("gradle");
        let _ = command_exists("npm");
        let _ = command_exists("yarn");
        let _ = command_exists("pnpm");
        let _ = command_exists("mvn");
    }
}