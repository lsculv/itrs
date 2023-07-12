# itrs (`it`)
itrs is a simple command line tool that provides command-line access to some
useful Rust iterator and string methods. These may be more memorable subtitute for
some quick `awk` commands or common command-line idioms like `sort | uniq`.
itrs provides a single executable that can access the various subcommands by name. 

## Installation
Installation requires cargo
```bash
cargo install itrs
```

## Usage
After installation you should have a binary called `it` avaliable. This can be
used to access the subcommands. For example, triming the leading and trailing
whitespace from a file called `input.txt` and write the output to the stdout.
```bash
it trim input.txt
```

This could also be acomplished using `awk` with something like:
```bash
awk '{$1=$1;print}' input.txt
```
However using the `it trim` command may be more memorable, and is more performant
in most cases.

A common idiom in shell scripts is `sort | uniq` for getting only the unique
lines of an input. This can be replaced by the `it unique` command, which works
even on unsorted data. Under the hood this uses the Rust `itertools` library's
unique method on iterators. This command can also be accessed with the alias
`it uniq` (many of the subcommands have useful aliases).

More subcommands, their aliases, and their descriptions can be shown with `it help`
