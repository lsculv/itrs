use std::env::args;
use std::fs::File;
use std::io::{self, stdin, stdout, BufRead, BufReader, BufWriter, Read, Write};
use std::process;
use std::error::Error;

fn run(file_handles: Vec<impl Read>) -> io::Result<()> {
    let mut buf_stdout = BufWriter::new(stdout().lock());
    let mut buf = String::with_capacity(255);

    for handle in file_handles {
        let mut buf_file = BufReader::new(handle);
        loop {
            let bytes = buf_file.read_line(&mut buf)?;
            if bytes == 0 {
                break;
            }
            writeln!(buf_stdout, "{}", buf.trim())?;

            buf.clear();
        }
    }
    buf_stdout.flush()?;

    Ok(())
}

fn fail(err: impl Error, context: &str) -> ! {
    eprintln!("{}: {}", context, err);
    process::exit(1);
}

fn main() -> io::Result<()> {
    let args: Vec<String> = args().collect();
    if args.len() > 1 {
        let file_handles: Vec<_> = args
            .into_iter()
            .skip(1)
            // PANIC: `unwrap` call never panics because we handle the error
            // and exit the program early.
            .map(|f| File::open(&f).map_err(|e| fail(e, &f)).unwrap())
            .collect();
        run(file_handles)?;
    } else {
        run(vec![stdin().lock()])?;
    }

    Ok(())
}
