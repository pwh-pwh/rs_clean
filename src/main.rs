use std::env::args;
use std::fs;
use std::path::Path;
use std::process::exit;
use std::time::Instant;

static EXCLUDE_DIR: &[&str] = &[
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

fn main() {
    let mut args = args();
    let _ = args.next().unwrap();
    let base_dir = args.next().unwrap();
    let mut cmd = std::process::Command::new("cargo");
    cmd.args(["clean"]);
    let start = Instant::now();
    do_clean(Path::new(&base_dir), &mut cmd);
    let elapsed = start.elapsed();
    println!("\n\x1B[32mdo_clean took {} seconds\x1B[0m", elapsed.as_secs_f64());
}

fn do_clean(dir: &Path, cmd: &mut std::process::Command) {
    if dir.is_dir() {
        if let Some(dir_name) = dir.file_name() {
            if EXCLUDE_DIR.contains(&dir_name.to_str().unwrap()) {
                return;
            }
            if dir_name.to_str().unwrap().starts_with(".") {
                return;
            }
        }
        let cargo_toml_path = dir.join("Cargo.toml");
        if cargo_toml_path.exists() {
            println!("\x1B[31mclean {}\x1B[0m", dir.display());
            cmd.current_dir(dir);
            let _ = cmd.output().map_err(|e| {
                eprintln!("{dir:?} > {e:?}");
                exit(1)
            });
        } else {
            for entry in fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                    do_clean(&path, cmd);
                }
            }
        }
    }
}
