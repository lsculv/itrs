use std::fmt;
use std::fs::File;
use std::io::{self, stdout, BufRead, BufReader, BufWriter, Write};
use std::str::FromStr;

use clap::{Arg, ColorChoice, Command};
use colored::control::SHOULD_COLORIZE;
use itertools::Itertools;
use thiserror::Error;

#[derive(Debug)]
enum IterTool {
    Trim,
    TrimStart,
    TrimEnd,
    ToUppercase,
    ToLowercase,
    Unique,
    Sum,
}

impl IterTool {
    // PERF: Can very likely eliminate allocations due to the `to_string()`
    // method for several of these subcommands. Per some sketchy profiling
    // alloactions take up ~1/3 of the runtime on large files.
    pub fn apply_to<'a>(&'a self, buf: &'a str) -> anyhow::Result<String> {
        match self {
            IterTool::Trim => Ok(buf.trim().to_string()),
            IterTool::TrimStart => Ok(buf.trim_start().to_string()),
            IterTool::TrimEnd => Ok(buf.trim_end().to_string()),
            IterTool::ToUppercase => Ok(buf.to_uppercase()),
            IterTool::ToLowercase => Ok(buf.to_lowercase()),
            IterTool::Unique => Ok(buf.lines().unique().join("\n")),
            IterTool::Sum => {
                let mut sum: i64 = 0;
                for line in buf.lines() {
                    sum += line.parse::<i64>()?;
                }
                Ok(sum.to_string())
            }
        }
    }

    pub fn is_by_lines(&self) -> bool {
        matches!(
            self,
            IterTool::Trim
                | IterTool::TrimStart
                | IterTool::TrimEnd
                | IterTool::ToUppercase
                | IterTool::ToLowercase
        )
    }

    pub fn removes_newline(&self) -> bool {
        matches!(self, IterTool::Trim | IterTool::TrimEnd)
    }
}

// TODO: This custom error type and error message could be removed using some
// of clap's features around making subcommands with enums. However I don't
// know yet how to do it through the builder API.
#[derive(Debug, Clone, Error)]
struct IterToolEnumParseError {
    attempted: String,
}

impl fmt::Display for IterToolEnumParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Attempted to parse \"{}\" as an IterTool variant.",
            self.attempted
        )?;
        write!(f, "       This likely means this variant was added as a clap subcommand but not to the `from_str` implementation of IterTool")?;
        Ok(())
    }
}

impl FromStr for IterTool {
    type Err = IterToolEnumParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "trim" => Ok(IterTool::Trim),
            "trim_start" => Ok(IterTool::TrimStart),
            "trim_end" => Ok(IterTool::TrimEnd),
            "to_uppercase" => Ok(IterTool::ToUppercase),
            "to_lowercase" => Ok(IterTool::ToLowercase),
            "unique" => Ok(IterTool::Unique),
            "sum" => Ok(IterTool::Sum),
            // This Err should never be reached in a release version as clap
            // should exclude invalid subcommands at the args parsing step.
            // The only exception is in the case specified by IterToolEnumParseError.
            _ => Err(IterToolEnumParseError {
                attempted: s.to_owned(),
            }),
        }
    }
}

pub fn run(config: Config) -> anyhow::Result<()> {
    #[inline(always)]
    fn make_reader(path: &str) -> anyhow::Result<Box<dyn BufRead>> {
        match path {
            "-" => Ok(Box::new(BufReader::new(io::stdin().lock()))),
            _ => Ok(Box::new(BufReader::new(File::open(path)?))),
        }
    }

    let mut readers: Vec<Box<dyn BufRead>> = Vec::with_capacity(config.files.len());
    for file in config.files {
        match make_reader(&file) {
            Ok(reader) => readers.push(reader),
            // Continue on file read error
            Err(e) => eprintln!("it: '{}': {}", file, e),
        }
    }

    let command = config.command;

    if command.is_by_lines() {
        apply_by_lines(readers, command)?;
    } else {
        apply_to_entire(readers, command)?;
    }

    #[inline(always)]
    fn apply_by_lines(readers: Vec<Box<dyn BufRead>>, command: IterTool) -> anyhow::Result<()> {
        let mut buf_stdout = BufWriter::new(stdout().lock());
        let mut buf = String::with_capacity(255);
        for mut reader in readers {
            loop {
                let bytes = reader.read_line(&mut buf)?;
                if bytes == 0 {
                    buf.clear();
                    break;
                }
                write!(buf_stdout, "{}", command.apply_to(&buf)?)?;
                if command.removes_newline() {
                    writeln!(buf_stdout)?;
                }

                buf.clear();
            }
        }
        buf_stdout.flush()?;

        Ok(())
    }

    #[inline(always)]
    fn apply_to_entire(readers: Vec<Box<dyn BufRead>>, command: IterTool) -> anyhow::Result<()> {
        let mut buf_stdout = BufWriter::new(stdout().lock());
        let mut buf = String::new();
        for mut reader in readers {
            reader.read_to_string(&mut buf)?;
            writeln!(buf_stdout, "{}", command.apply_to(&buf)?)?;
            buf.clear();
        }
        buf_stdout.flush()?;

        Ok(())
    }

    Ok(())
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    command: IterTool,
}

pub fn parse_args() -> anyhow::Result<Config> {
    let matches = Command::new("it")
        .version("0.1.0")
        .author("Lucas Culverhouse")
        .about("Provides command-line access to several useful Rust iterator and string methods")
        .subcommand_required(true)
        .color(if SHOULD_COLORIZE.should_colorize() {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        })
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .help("Input file(s)")
                .default_value("-")
                .num_args(1..)
                .global(true),
        )
        .subcommand(Command::new("trim"))
        .subcommand(Command::new("trim_start").visible_aliases(["trim_left", "triml"]))
        .subcommand(Command::new("trim_end").visible_aliases(["trim_right", "trimr"]))
        .subcommand(Command::new("to_uppercase").visible_aliases(["upper", "uppercase"]))
        .subcommand(Command::new("to_lowercase").visible_aliases(["lower", "lowercase"]))
        .subcommand(Command::new("unique").visible_alias("uniq"))
        .subcommand(Command::new("sum"))
        .get_matches();

    let (subcommand, subargs) = matches
        .subcommand()
        .expect("subcommand should be required by clap");

    let files: Vec<String> = subargs
        .get_many("files")
        .expect("files should at least contain STDIN")
        .cloned()
        .collect();

    let command: IterTool = IterTool::from_str(subcommand)?;

    Ok(Config { files, command })
}
