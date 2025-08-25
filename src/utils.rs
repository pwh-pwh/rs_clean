use which::which;
use std::path::{Path, PathBuf};
use crate::config::ConfigError; // 引入 ConfigError

pub fn command_exists(cmd: &str) -> bool {
    which(cmd).is_ok()
}

/// Validate and sanitize a path to prevent directory traversal attacks
pub fn validate_and_sanitize_path(path_str: &str) -> Result<PathBuf, ConfigError> {
    let path = Path::new(path_str);
    
    // Check for empty path
    if path_str.is_empty() {
        return Err(ConfigError::InvalidConfig(
            "Path cannot be empty".to_string()
        ));
    }

    // Normalize path separators for cross-platform checking
    let path_str_normalized = path_str.replace('\\', "/");
    
    // Check for path traversal patterns (cross-platform)
    if path_str_normalized.contains("../") || path_str_normalized.contains("~/") {
        return Err(ConfigError::InvalidConfig(
            "Path traversal (../) or home directory (~) not allowed".to_string()
        ));
    }

    // Check for absolute paths that might be dangerous
    if path.is_absolute() {
        // For security, we'll restrict absolute paths to common safe directories
        let path_str_lower = path_str_normalized.to_lowercase();
        let dangerous_patterns = [
            // Unix system directories
            "/etc", "/usr", "/bin", "/sbin", "/lib", "/boot", "/dev", "/proc",
            "/sys", "/root", "/var", "/opt", "/tmp", "/home", "/mnt", "/media",
            // Windows system directories
            "c:/windows", "c:\\windows", "d:/windows", "d:\\windows",
            "c:/program files", "c:\\program files", "c:/program files (x86)", "c:\\program files (x86)",
            "c:/system32", "c:\\system32", "c:/winnt", "c:\\winnt",
            // macOS system directories
            "/applications", "/library", "/system", "/users",
            // Common dangerous paths
            "/windows", "/program files", "/system32", "/winnt",
        ];
        
        // Check for exact matches and prefix matches
        for pattern in &dangerous_patterns {
            let pattern_lower = pattern.to_lowercase();
            if path_str_lower.starts_with(&pattern_lower) {
                // Check if it's exactly the pattern or pattern followed by a separator
                let path_len = path_str_lower.len();
                let pattern_len = pattern_lower.len();
                if path_len == pattern_len ||
                   path_str_lower.chars().nth(pattern_len).map_or(false, |c| c == '/' || c == '\\') {
                    return Err(ConfigError::InvalidConfig(
                        format!("Access to system directory '{}' not allowed", pattern)
                    ));
                }
            }
        }
    }

    // Always attempt canonicalization for security - this is critical
    let canonical_path = match path.canonicalize() {
        Ok(path) => path,
        Err(_) => {
            return Err(ConfigError::InvalidConfig(
                "Path does not exist or cannot be accessed".to_string()
            ));
        }
    };

    // Ensure path doesn't escape current working directory
    if let Ok(current_dir) = std::env::current_dir() {
        if let Ok(current_canonical) = current_dir.canonicalize() {
            if !canonical_path.starts_with(&current_canonical) {
                return Err(ConfigError::InvalidConfig(
                    "Path must be within the current directory tree".to_string()
                ));
            }
        }
    }

    Ok(canonical_path)
}

/// Validate exclude directory names to prevent injection attacks
pub fn validate_exclude_dir_name(dir_name: &str) -> Result<(), ConfigError> {
    if dir_name.is_empty() {
        return Err(ConfigError::InvalidConfig(
            "Exclude directory name cannot be empty".to_string()
        ));
    }

    // Check for path traversal attempts
    if dir_name.contains("..") || dir_name.contains('/') || dir_name.contains('\\') {
        return Err(ConfigError::InvalidConfig(
            format!("Invalid exclude directory name: '{}'", dir_name)
        ));
    }

    // Check for reserved names
    if dir_name == "." || dir_name == ".." {
        return Err(ConfigError::InvalidConfig(
            format!("Reserved directory name cannot be excluded: '{}'", dir_name)
        ));
    }

    // Check for Windows reserved names (case-insensitive)
    let dir_name_lower = dir_name.to_lowercase();
    let windows_reserved = [
        "con", "prn", "aux", "nul",
        "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8", "com9",
        "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
    ];
    
    if windows_reserved.contains(&dir_name_lower.as_str()) {
        return Err(ConfigError::InvalidConfig(
            format!("Windows reserved name cannot be used as exclude directory: '{}'", dir_name)
        ));
    }

    // Check length limit
    if dir_name.len() > 255 {
        return Err(ConfigError::InvalidConfig(
            "Exclude directory name too long (max 255 characters)".to_string()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_command_exists() {
        // cargo should always be available in the test environment
        assert!(command_exists("cargo"));

        // Test for a command that is unlikely to exist
        assert!(!command_exists("a-command-that-does-not-exist"));
    }

    // Security tests for path validation
    #[test]
    fn test_validate_and_sanitize_path_reject_traversal() {
        // Test various path traversal attempts
        assert!(validate_and_sanitize_path("../etc/passwd").is_err());
        assert!(validate_and_sanitize_path("../../etc").is_err());
        assert!(validate_and_sanitize_path("../../../usr/bin").is_err());
        assert!(validate_and_sanitize_path("file/../etc/passwd").is_err());
        assert!(validate_and_sanitize_path("dir/../../etc").is_err());
    }

    #[test]
    fn test_validate_and_sanitize_path_reject_home_directory() {
        // Test home directory access attempts
        assert!(validate_and_sanitize_path("~/etc/passwd").is_err());
        assert!(validate_and_sanitize_path("~/").is_err());
        assert!(validate_and_sanitize_path("~/Documents").is_err());
    }

    #[test]
    fn test_validate_and_sanitize_path_reject_system_directories() {
        // Test Unix system directories
        assert!(validate_and_sanitize_path("/etc/passwd").is_err());
        assert!(validate_and_sanitize_path("/usr/bin").is_err());
        assert!(validate_and_sanitize_path("/bin/sh").is_err());
        assert!(validate_and_sanitize_path("/etc/").is_err());
        
        // Test Windows system directories
        assert!(validate_and_sanitize_path("C:\\Windows\\System32").is_err());
        assert!(validate_and_sanitize_path("C:/Windows/System32").is_err());
        assert!(validate_and_sanitize_path("C:\\Program Files").is_err());
        
        // Test macOS system directories
        assert!(validate_and_sanitize_path("/Applications").is_err());
        assert!(validate_and_sanitize_path("/System/Library").is_err());
    }

    #[test]
    fn test_validate_and_sanitize_path_reject_windows_traversal() {
        // Test Windows-style path traversal
        assert!(validate_and_sanitize_path("..\\..\\Windows").is_err());
        assert!(validate_and_sanitize_path("file\\..\\etc").is_err());
        assert!(validate_and_sanitize_path("dir\\..\\..\\etc").is_err());
    }

    #[test]
    fn test_validate_and_sanitize_path_allow_valid_paths() {
        // Test valid paths (these should work if they exist)
        assert!(validate_and_sanitize_path(".").is_ok());
        
        // Create a temporary directory within current working directory
        let temp_dir = tempfile::TempDir::new_in(".").unwrap();
        let temp_path = temp_dir.path();
        
        // Convert to relative path for validation
        let current_dir = std::env::current_dir().unwrap();
        let relative_path = temp_path.strip_prefix(&current_dir).unwrap_or(temp_path);
        
        assert!(validate_and_sanitize_path(relative_path.to_str().unwrap()).is_ok());
        
        // Test relative path that exists
        let test_file = temp_path.join("test_file");
        std::fs::write(&test_file, "test").unwrap();
        let relative_test_file = test_file.strip_prefix(&current_dir).unwrap_or(&test_file);
        assert!(validate_and_sanitize_path(relative_test_file.to_str().unwrap()).is_ok());
    }

    #[test]
    fn test_validate_and_sanitize_path_reject_nonexistent() {
        // Test that non-existent paths are rejected
        assert!(validate_and_sanitize_path("/nonexistent/path").is_err());
        assert!(validate_and_sanitize_path("nonexistent_dir").is_err());
        assert!(validate_and_sanitize_path("../nonexistent").is_err());
    }

    #[test]
    fn test_validate_and_sanitize_path_empty_path() {
        // Test empty path
        assert!(validate_and_sanitize_path("").is_err());
    }

    #[test]
    fn test_validate_exclude_dir_name_valid() {
        // Test valid exclude directory names
        assert!(validate_exclude_dir_name("node_modules").is_ok());
        assert!(validate_exclude_dir_name("target").is_ok());
        assert!(validate_exclude_dir_name("build").is_ok());
        assert!(validate_exclude_dir_name("dist").is_ok());
        assert!(validate_exclude_dir_name("vendor").is_ok());
        assert!(validate_exclude_dir_name("custom_dir").is_ok());
    }

    #[test]
    fn test_validate_exclude_dir_name_invalid() {
        // Test invalid exclude directory names
        assert!(validate_exclude_dir_name("").is_err());
        assert!(validate_exclude_dir_name(".").is_err());
        assert!(validate_exclude_dir_name("..").is_err());
        assert!(validate_exclude_dir_name("../malicious").is_err());
        assert!(validate_exclude_dir_name("../../etc").is_err());
        assert!(validate_exclude_dir_name("dir/../etc").is_err());
        assert!(validate_exclude_dir_name("path/with/slashes").is_err());
        assert!(validate_exclude_dir_name("path\\with\\backslashes").is_err());
    }

    #[test]
    fn test_validate_exclude_dir_name_reserved_names() {
        // Test Windows reserved names
        assert!(validate_exclude_dir_name("con").is_err());
        assert!(validate_exclude_dir_name("prn").is_err());
        assert!(validate_exclude_dir_name("aux").is_err());
        assert!(validate_exclude_dir_name("nul").is_err());
        assert!(validate_exclude_dir_name("com1").is_err());
        assert!(validate_exclude_dir_name("lpt1").is_err());
    }

    #[test]
    fn test_validate_exclude_dir_name_too_long() {
        // Test name length limit
        let long_name = "a".repeat(256);
        assert!(validate_exclude_dir_name(&long_name).is_err());
        
        // Test name at length limit
        let max_name = "a".repeat(255);
        assert!(validate_exclude_dir_name(&max_name).is_ok());
    }
}
