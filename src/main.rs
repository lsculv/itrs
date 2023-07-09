use std::fs::File;
use std::io::{self, stdout, BufRead, BufReader, BufWriter, Write};
use std::process;
use std::str::FromStr;

use clap::{Arg, Command};
use colored::Colorize;

#[derive(Debug)]
enum IterTool {
    Trim,
    TrimStart,
    TrimEnd,
    ToUppercase,
    ToLowercase,
}

impl IterTool {
    pub fn apply_to<'a>(&'a self, buf: &'a str) -> String {
        match self {
            IterTool::Trim => buf.trim().to_string(),
            IterTool::TrimStart => buf.trim_start().to_string(),
            IterTool::TrimEnd => buf.trim_end().to_string(),
            IterTool::ToUppercase => buf.to_uppercase(),
            IterTool::ToLowercase => buf.to_lowercase(),
        }
    }

    pub fn removes_newline(&self) -> bool {
        matches!(self, IterTool::Trim | IterTool::TrimEnd)
    }
}

impl FromStr for IterTool {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "trim" => Ok(IterTool::Trim),
            "trim_start" => Ok(IterTool::TrimStart),
            "trim_end" => Ok(IterTool::TrimEnd),
            "to_uppercase" => Ok(IterTool::ToUppercase),
            "to_lowercase" => Ok(IterTool::ToLowercase),
            _ => Err(()),
        }
    }
}

fn run(config: Config) -> anyhow::Result<()> {
    #[inline(always)]
    fn make_reader(path: &str) -> anyhow::Result<Box<dyn BufRead>> {
        match path {
            "-" => Ok(Box::new(BufReader::new(io::stdin().lock()))),
            _ => Ok(Box::new(BufReader::new(File::open(path)?))),
        }
    }

    let mut readers: Vec<Box<dyn BufRead>> = Vec::with_capacity(config.files.len());
    for file in config.files {
        readers.push(make_reader(&file)?);
    }

    let command = config.command;

    let mut buf_stdout = BufWriter::new(stdout().lock());
    let mut buf = String::with_capacity(255);

    for mut reader in readers {
        loop {
            let bytes = reader.read_line(&mut buf)?;
            if bytes == 0 {
                buf.clear();
                break;
            }
            write!(buf_stdout, "{}", command.apply_to(&buf))?;
            if command.removes_newline() {
                writeln!(buf_stdout)?;
            }

            buf.clear();
        }
    }
    buf_stdout.flush()?;

    Ok(())
}

#[derive(Debug)]
struct Config {
    files: Vec<String>,
    command: IterTool,
}

fn parse_args() -> anyhow::Result<Config> {
    let matches = Command::new("it")
        .version("0.1.0")
        .author("Lucas Culverhouse")
        .about("Provides command-line access to several useful Rust itertools and string methods")
        .subcommand_required(true)
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .help("Input file(s)")
                .default_value("-")
                .num_args(1..)
                .global(true),
        )
        .subcommand(Command::new("trim"))
        .subcommand(Command::new("trim_start").visible_alias("trim_left"))
        .subcommand(Command::new("trim_end").visible_alias("trim_right"))
        .subcommand(Command::new("to_uppercase").visible_aliases(["upper", "uppercase"]))
        .subcommand(Command::new("to_lowercase").visible_aliases(["lower", "lowercase"]))
        .get_matches();

    let (subcommand, subargs) = matches
        .subcommand()
        .expect("subcommand should be required by clap");

    let files: Vec<String> = subargs
        .get_many("files")
        .expect("files should at least contain STDIN")
        .cloned()
        .collect();

    let command: IterTool =
        IterTool::from_str(subcommand).expect("clap should catch invalid subcommands being passed");

    Ok(Config { files, command })
}

fn main() {
    if let Err(e) = parse_args().and_then(run) {
        eprintln!("{} {}", "error:".bright_red(), e);
        process::exit(1);
    }
}
