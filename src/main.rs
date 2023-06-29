use std::env::args;
use std::error::Error;
#[allow(unused_imports)]
use std::fs::File;
use std::io::{self, stdin, stdout, BufRead, BufReader, BufWriter, Read, Write};
use std::process;

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

fn run(file_handles: Vec<impl Read>, itertool: IterTool) -> io::Result<()> {
    let mut buf_stdout = BufWriter::new(stdout().lock());
    let mut buf = String::with_capacity(255);

    for handle in file_handles {
        let mut buf_file = BufReader::new(handle);
        loop {
            let bytes = buf_file.read_line(&mut buf)?;
            if bytes == 0 {
                break;
            }
            write!(buf_stdout, "{}", itertool.apply_to(&buf))?;
            if itertool.removes_newline() {
                writeln!(buf_stdout)?;
            }

            buf.clear();
        }
    }
    buf_stdout.flush()?;

    Ok(())
}

#[allow(dead_code)]
fn fail(err: impl Error, context: &str) -> ! {
    eprintln!("{}: {}", context, err);
    process::exit(1);
}

fn main() -> io::Result<()> {
    let mut args: Vec<String> = args().skip(1).collect();
    if args.len() == 0 {
        eprintln!("usage: it-rust [itertool]")
    }

    let itertool = match args.pop().unwrap().as_str() {
        "trim" => IterTool::Trim,
        "trim_start" => IterTool::TrimStart,
        "trim_end" => IterTool::TrimEnd,
        "to_uppercase" => IterTool::ToUppercase,
        "to_lowercase" => IterTool::ToLowercase,

        unrecognized => {
            eprintln!("{}: Unknown command", unrecognized);
            process::exit(1);
        }
    };

    if let Some(file) = args.pop() {
        let file_handle = File::open(file)?;
        run(vec![file_handle], itertool)?
    } else {
        run(vec![stdin().lock()], itertool)?;
    }

    /*
    if args.len() > 1 {
        let file_handles: Vec<_> = args
            .into_iter()
            // PANIC: `unwrap` call never panics because we handle the error
            // and exit the program early.
            .map(|f| File::open(&f).map_err(|e| fail(e, &f)).unwrap())
            .collect();
        run(file_handles)?;
    } else {
    */
    //}

    Ok(())
}
