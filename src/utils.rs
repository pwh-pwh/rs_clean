use std::process::Command;

pub fn command_exists(cmd: &str) -> bool {
    Command::new(cmd).args(["--version"]).output().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_exists() {
        assert!(command_exists("gradle"));
    }
}