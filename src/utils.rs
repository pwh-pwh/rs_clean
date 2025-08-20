use which::which;

pub fn command_exists(cmd: &str) -> bool {
    which(cmd).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_exists() {
        // cargo should always be available in the test environment
        assert!(command_exists("cargo"));

        // Test for a command that is unlikely to exist
        assert!(!command_exists("a-command-that-does-not-exist"));
    }
}
