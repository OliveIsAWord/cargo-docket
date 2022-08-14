#![forbid(unsafe_code)]
#![allow(dead_code, unused_imports)]

#[macro_use]
mod cute_error;

use clap::{Parser, Subcommand};
use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Write};
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
}

fn main() -> ExitCode {
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
            yeet!()
        }
    };
    debug_println!("done parsing: {:?}", args);
    match args.action.unwrap_or(DEFAULT_ACTION) {
        Action::New => action_new(),
        Action::Meow => {
            println!("Meow!");
            ExitCode::SUCCESS
        }
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
        Err(io::ErrorKind::AlreadyExists) => yeet!("docket.md already exists in this directory"),
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
