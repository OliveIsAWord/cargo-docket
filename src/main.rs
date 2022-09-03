#![forbid(unsafe_code)]

#[macro_use]
mod cute_error;
mod parse;

use clap::{Parser, Subcommand};
use std::env;
use std::ffi::OsString;
use std::fs::{remove_file, File};
use std::io::{self, BufRead, Write};
use std::iter::once;
use std::process::ExitCode;

macro_rules! debug_println {
    ($($arg:tt)*) => {
        print!("[{} {}:{}] -> ", file!(), line!(), column!());
        println!($($arg)*)
    }
}

const APP_NAME: &str = "docket";
const PRELUDE: &str = include_str!("../prelude.md");
const TEMPLATE: &str = include_str!("../template.md");
const DOCKET_PATH: &str = "docket.md";
const DEFAULT_ACTION: Action = Action::Meow;

#[derive(Debug, Parser)]
#[clap(version, about)]
struct DocketArgs {
    #[clap(subcommand)]
    action: Option<Action>,
}

#[derive(Debug, Subcommand)]
enum Action {
    /// Meow at the user. [default]
    Meow,
    /// Create new docket file.
    New,
    /// Delete current docket file.
    Delete,
}

fn main() -> ExitCode {
    let tryy = "meow meow!!\n this is a coool string :3 \n\r\n\n\runix windows unix mac";
    dbg!(parse::parse(tryy));
    let (is_via_cargo, args) = {
        // Skip the executable path
        let mut args = env::args_os().skip(1).peekable();
        // The second argument will be "docket" if run via `cargo docket`.
        let is_via_cargo = args.next_if_eq(APP_NAME).is_some();
        let args = args.collect::<Vec<_>>();
        (is_via_cargo, args)
    };
    if is_via_cargo {
        debug_println!("ran via cargo");
    } else {
        debug_println!("ran standalone");
    }
    debug_println!("got args: {:?}", args);
    let cli_name = OsString::from(if is_via_cargo {
        format!("cargo {}", APP_NAME)
    } else {
        format!("cargo-{}", APP_NAME)
    });
    let args = match DocketArgs::try_parse_from(once(cli_name).chain(args)) {
        Ok(opt) => opt,
        Err(e) => {
            let _ignore = e.print();
            if e.use_stderr() {
                yeet!()
            }
            return ExitCode::SUCCESS;
        }
    };
    debug_println!("done parsing: {:?}", args);
    match args.action.unwrap_or(DEFAULT_ACTION) {
        Action::Meow => {
            println!("Meow!");
            ExitCode::SUCCESS
        }
        Action::New => action_new(),
        Action::Delete => action_delete(),
    }
}

fn action_new() -> ExitCode {
    debug_println!("ACTION: new");
    match File::options()
        .create_new(true)
        .write(true)
        .open(DOCKET_PATH)
        .map_err(|e| e.kind())
    {
        Ok(mut f) => {
            try_yeet!(write_new_docket(&mut f));
            ExitCode::SUCCESS
        }
        Err(io::ErrorKind::AlreadyExists) => {
            yeet!(DOCKET_PATH, " already exists in this directory")
        }
        Err(e) => yeet!(e),
    }
}

fn write_new_docket<T>(w: &mut T) -> io::Result<()>
where
    T: Write,
{
    writeln!(w, "{}", PRELUDE)?;
    writeln!(w, "{}", TEMPLATE)?;
    w.flush()
}

fn action_delete() -> ExitCode {
    debug_println!("ACTION: delete");
    match yes_no(&format!("Permanently delete {}?", DOCKET_PATH), Some(false)) {
        Ok(true) => (),
        Ok(false) => return ExitCode::SUCCESS,
        Err(e) => yeet!(e),
    };
    match remove_file(DOCKET_PATH).map_err(|e| e.kind()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(io::ErrorKind::NotFound) => {
            yeet!(DOCKET_PATH, " does not exist in this directory")
        }
        Err(e) => yeet!(e),
    }
}

fn yes_no(msg: &str, default_answer: Option<bool>) -> io::Result<bool> {
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    let mut stdin = io::stdin().lock();
    let prompt = match default_answer {
        Some(true) => "Y/n",
        Some(false) => "y/N",
        None => "y/n",
    };
    // I don't know if this is standard, but we only print the question a certain number of times
    // before giving up and throwing an error.
    let max_trys = 3;
    let mut buffer = String::new();
    for _ in 0..max_trys {
        write!(stdout, "{} [{}] ", msg, prompt)?;
        stdout.flush()?;
        buffer.clear();
        stdin.read_line(&mut buffer)?;
        let mut c = buffer.trim_start().chars().next();
        c.as_mut().map(char::make_ascii_lowercase);
        match c {
            Some('y') => return Ok(true),
            Some('n') => return Ok(false),
            Some(_) => writeln!(stderr, "unrecognized character")?,
            None => {
                if let Some(answer) = default_answer {
                    return Ok(answer);
                }
            }
        }
    }
    Err(io::ErrorKind::TimedOut.into())
}
