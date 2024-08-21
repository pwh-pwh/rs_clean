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