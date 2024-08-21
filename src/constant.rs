use std::collections::HashMap;
use std::sync::Once;

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

static INIT: Once = Once::new();
static mut CMD_MAP: Option<HashMap<&'static str, Vec<&'static str>>> = None;

pub fn get_cmd_map() -> &'static HashMap<&'static str, Vec<&'static str>> {
    unsafe {
        INIT.call_once(|| {
            let mut m = HashMap::new();
            m.insert("cargo", vec!["Cargo.toml"]);
            m.insert("go", vec!["go.mod"]);
            m.insert("gradle", vec!["build.gradle","build.gradle.kts"]);
            #[cfg(not(target_os = "windows"))]
            {
                m.insert("mvn", vec!["pom.xml"]);
            }
            #[cfg(target_os = "windows")]
            {
                m.insert("mvn.cmd", vec!["pom.xml"]);
            }
            CMD_MAP = Some(m);
        });
        CMD_MAP.as_ref().expect("CMD_MAP was not initialized")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_cmd_map() {
        assert_eq!(get_cmd_map().get("cargo"), Some(&vec!["Cargo.toml"]));
    }
}