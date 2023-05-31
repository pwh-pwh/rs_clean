use std::env::args;
use std::fs;
use std::process::exit;

fn main() {
    let mut args = args();
    let _ = args.next().unwrap();
    let base_dir = args.next().unwrap();
    let dir = fs::read_dir(base_dir).unwrap();
    let mut cmd = std::process::Command::new("cargo");
    cmd.args(&["clean"]);
    for file in dir {
        let file = file.unwrap();
        let file_type = file.file_type().unwrap();
        if file_type.is_dir() {
            println!("clean {:?}", file.path().display().to_string());
            cmd.current_dir(file.path());
            let file = file.path().display().to_string();
            let _ = cmd.output().map_err(|e| {
                eprintln!("{file} > {e:?}");
                exit(1)
            });
        }
    }
}
