use std::fmt::Display;
use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
// TODO: does including this function over and over cause code bloat?
pub fn pretty_print_error_start() -> io::Result<()> {
    // TODO: what should be the `ColorChoice` policy??
    let mut stderr = StandardStream::stderr(ColorChoice::Auto);
    stderr.set_color(
        ColorSpec::new()
            .set_fg(Some(Color::Red))
            //.set_bg(Some(Color::Black))
            .set_bold(true),
    )?;
    write!(&mut stderr, "error: ")?;
    stderr.reset()
}

pub fn internal_print_error<T>(err: T) -> io::Result<()>
where
    T: Display,
{
    let mut stderr = StandardStream::stderr(ColorChoice::Never);
    write!(&mut stderr, "{}", err)
}

pub fn internal_print_error_end() -> io::Result<()> {
    let mut stderr = StandardStream::stderr(ColorChoice::Never);
    writeln!(&mut stderr)
}

#[macro_export]
macro_rules! yeet {
    () => {{
        return std::process::ExitCode::FAILURE;
    }};
    ($($err:expr),+) => {{
        let _ignore = $crate::cute_error::pretty_print_error_start();
        $(
            let _ignore = $crate::cute_error::internal_print_error($err);
        )+
        let _ignore = $crate::cute_error::internal_print_error_end();
        return std::process::ExitCode::FAILURE;
    }};
}

#[macro_export]
macro_rules! try_yeet {
    ($val:expr $(,)?) => {
        match $val {
            Ok(x) => x,
            Err(e) => yeet!(e),
        }
    };
    ($val:expr, $($err:expr),+) => {
        match $val {
            Ok(x) => x,
            Err(e) => yeet!(e, $($err),+),
        }
    };
}
