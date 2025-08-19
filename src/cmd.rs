use std::path::Path;
use std::process::Command;
use std::fs;
use std::io;

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
    
    // 检查是否为需要特殊处理的命令
    pub fn is_special_clean_command(&self) -> bool {
        matches!(self.name, "npm" | "yarn" | "pnpm")
    }
    
    // 执行特殊清理逻辑
    pub fn run_special_clean(&self, dir: &Path) -> io::Result<()> {
        match self.name {
            "npm" | "yarn" | "pnpm" => self.clean_nodejs_project(dir),
            _ => Ok(())
        }
    }
    
    // 清理 Node.js 项目
    fn clean_nodejs_project(&self, dir: &Path) -> io::Result<()> {
        let mut cleaned_count = 0;
        
        // 删除 node_modules 文件夹
        let node_modules = dir.join("node_modules");
        if node_modules.exists() {
            if let Err(e) = fs::remove_dir_all(&node_modules) {
                eprintln!("Failed to remove {}: {}", node_modules.display(), e);
            } else {
                cleaned_count += 1;
                println!("Removed node_modules/");
            }
        }
        
        // 删除 package-lock.json
        let package_lock = dir.join("package-lock.json");
        if package_lock.exists() {
            if let Err(e) = fs::remove_file(&package_lock) {
                eprintln!("Failed to remove {}: {}", package_lock.display(), e);
            } else {
                cleaned_count += 1;
                println!("Removed package-lock.json");
            }
        }
        
        // 删除 yarn.lock (如果是 yarn 项目)
        if self.name == "yarn" {
            let yarn_lock = dir.join("yarn.lock");
            if yarn_lock.exists() {
                if let Err(e) = fs::remove_file(&yarn_lock) {
                    eprintln!("Failed to remove {}: {}", yarn_lock.display(), e);
                } else {
                    cleaned_count += 1;
                    println!("Removed yarn.lock");
                }
            }
        }
        
        // 删除 pnpm-lock.yaml (如果是 pnpm 项目)
        if self.name == "pnpm" {
            let pnpm_lock = dir.join("pnpm-lock.yaml");
            if pnpm_lock.exists() {
                if let Err(e) = fs::remove_file(&pnpm_lock) {
                    eprintln!("Failed to remove {}: {}", pnpm_lock.display(), e);
                } else {
                    cleaned_count += 1;
                    println!("Removed pnpm-lock.yaml");
                }
            }
        }
        
        // 删除 .npm 缓存文件夹
        let npm_cache = dir.join(".npm");
        if npm_cache.exists() {
            if let Err(e) = fs::remove_dir_all(&npm_cache) {
                eprintln!("Failed to remove {}: {}", npm_cache.display(), e);
            } else {
                cleaned_count += 1;
                println!("Removed .npm/");
            }
        }
        
        // 删除 .yarn 缓存文件夹 (如果是 yarn 项目)
        if self.name == "yarn" {
            let yarn_cache = dir.join(".yarn");
            if yarn_cache.exists() {
                if let Err(e) = fs::remove_dir_all(&yarn_cache) {
                    eprintln!("Failed to remove {}: {}", yarn_cache.display(), e);
                } else {
                    cleaned_count += 1;
                    println!("Removed .yarn/");
                }
            }
        }
        
        if cleaned_count > 0 {
            println!("Cleaned {} Node.js artifacts", cleaned_count);
        }
        
        Ok(())
    }
    
}

#[cfg(test)]
mod tests {
    use crate::constant::get_cmd_map;
    use crate::utils::command_exists;
    use super::*;
    
    #[test]
    fn test_cmd() {
        let cmd = Cmd::new("cargo", vec!["Cargo.toml"]);
        assert_eq!(cmd.name, "cargo");
        assert_eq!(cmd.related_files, vec!["Cargo.toml"]);
    }
    
    #[test]
    fn test_init_cmd_list() {
        let map = get_cmd_map();
        let mut cmd_list = vec![];
        //遍历map
        for (key, value) in map {
            if command_exists(key) {
                cmd_list.push(Cmd::new(key, value.clone()));
            }
        }
        // 现在有7个命令：cargo, go, gradle, npm, yarn, pnpm, mvn/mvn.cmd
        // 但测试环境中可能只有部分命令可用，所以检查总数而不是固定值
        assert!(cmd_list.len() >= 1); // 至少应该有cargo可用
        assert!(cmd_list.len() <= 7); // 最多7个命令
    }
    
    #[test]
    fn test_special_clean_commands() {
        let npm_cmd = Cmd::new("npm", vec!["package.json"]);
        let yarn_cmd = Cmd::new("yarn", vec!["package.json"]);
        let pnpm_cmd = Cmd::new("pnpm", vec!["package.json"]);
        let cargo_cmd = Cmd::new("cargo", vec!["Cargo.toml"]);
        
        assert!(npm_cmd.is_special_clean_command());
        assert!(yarn_cmd.is_special_clean_command());
        assert!(pnpm_cmd.is_special_clean_command());
        assert!(!cargo_cmd.is_special_clean_command());
    }
}