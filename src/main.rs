use rs_clean::cmd::Cmd;
use rs_clean::constant::get_cmd_map;
use rs_clean::utils::command_exists;
use rs_clean::do_clean_all;
use std::env::args;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

fn main() {
    let mut args = args();
    let _ = args.next().unwrap();
    let base_dir = args.next().unwrap();
    let mut cmd = Command::new("cargo");
    cmd.args(["clean"]);
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
    println!(
        "\n\x1B[32mdo_clean took {} seconds\x1B[0m",
        elapsed.as_secs_f64()
    );
}
