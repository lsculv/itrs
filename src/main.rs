use colored::Colorize;
use std::process;

mod it;

fn main() {
    if let Err(e) = it::parse_args().and_then(it::run) {
        eprintln!("{} {}", "error:".bright_red().bold(), e);
        process::exit(1);
    }
}
