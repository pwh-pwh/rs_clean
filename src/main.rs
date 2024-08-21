use rs_clean::do_clean;
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
    do_clean(Path::new(&base_dir), &mut cmd);
    let elapsed = start.elapsed();
    println!(
        "\n\x1B[32mdo_clean took {} seconds\x1B[0m",
        elapsed.as_secs_f64()
    );
}
