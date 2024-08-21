use std::path::Path;
use std::process::Command;

pub struct Cmd<'a> {
    pub name: &'a str,
    pub cmd: Command,
    pub related_files: Vec<&'a str>,
}

impl<'a> Cmd<'a> {
    pub fn new(cmd_str: &'a str, related_files: Vec<&'a str>) -> Self {
        let mut command = Command::new(cmd_str);
        command.args(["clean"]);
        Self {
            name: cmd_str,
            cmd: command,
            related_files,
        }
    }
    
    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) {
        self.cmd.current_dir(dir);
    }
    
    pub fn run(&mut self) -> std::io::Result<std::process::Output> {
        self.cmd.output()
    }
    
}

#[cfg(test)]
mod tests {
    use crate::constant::get_cmd_map;
    use super::*;
    
    #[test]
    fn test_cmd() {
        let mut cmd = Cmd::new("cargo", vec!["Cargo.toml"]);
        assert_eq!(cmd.name, "cargo");
        assert_eq!(cmd.related_files, vec!["Cargo.toml"]);
    }
    
    #[test]
    fn test_init_cmd_list() {
        let map = get_cmd_map();
        let mut cmd_list = vec![];
        //遍历map
        for (key, value) in map {
            for v in value {
                cmd_list.push(Cmd::new(key, vec![v]));
            }
        }
        assert_eq!(cmd_list.len(), 1);
    }
}