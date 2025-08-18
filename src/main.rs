use rs_clean::cmd::Cmd;
use rs_clean::constant::get_cmd_map;
use rs_clean::{do_clean_all, COUNT};
use rs_clean::utils::command_exists;
use std::env::args;
use std::path::Path;
use std::time::Instant;

fn main() {
    let mut args = args();
    let _program_name = args.next(); // Skip program name, safe to ignore
    let base_dir = args.next().unwrap_or_else(|| ".".to_string());
    let start = Instant::now();
    let map = get_cmd_map();
    let mut cmd_list = vec![];
    for (key, value) in map {
        if command_exists(key) {
            cmd_list.push(Cmd::new(key, value.clone()));
        }
    }
    let init_cmd:Vec<String> = cmd_list.iter().map(|cmd| {
        cmd.name.to_string()
    }).collect();
    println!("find supports clean command is \x1B[34m{:?}\x1B[0m", init_cmd);
    do_clean_all(Path::new(&base_dir), &mut cmd_list);
    let elapsed = start.elapsed();
    unsafe {
        println!(
            "\n\x1B[32m rs_clean clean {} packages took {} seconds\x1B[0m", COUNT,
            elapsed.as_secs_f64()
        );
    }
}
