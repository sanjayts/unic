use std::env::args;
use std::process::exit;
use unic::{parse_config, run};

fn main() {
    let cmd_args = args().collect();
    if let Err(e) = parse_config(cmd_args).and_then(|c| run(&c)) {
        eprintln!("{}", e);
        exit(1);
    }
}
