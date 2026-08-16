use std::ffi::OsString;
use std::process::{Command, ExitCode};

use crate::shim;
use crate::Result;

/// Runs `cargo doc` with this binary standing in for rustdoc.
///
/// Everything but the options of this one is handed to `cargo doc`, so the
/// command takes whatever that one takes.
pub fn run() -> Result<ExitCode> {
    let mut args = std::env::args_os().skip(1).peekable();

    // Cargo runs an external subcommand with the name of the subcommand in
    // front, where a direct call has nothing there.
    if args.peek().is_some_and(|arg| arg == "typstdoc") {
        args.next();
    }

    let mut preamble = None;
    let mut forwarded = Vec::new();
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--preamble") => {
                preamble = Some(args.next().ok_or("--preamble takes a path")?);
            }
            _ => forwarded.push(arg),
        }
    }

    let mut command = Command::new(std::env::var_os("CARGO").unwrap_or("cargo".into()));
    command
        .arg("doc")
        .args(forwarded)
        .env(shim::MARKER, "1")
        .env("RUSTDOC", std::env::current_exe()?)
        .env("RUSTDOCFLAGS", rustdocflags());
    if let Some(preamble) = preamble {
        command.env(shim::PREAMBLE, preamble);
    }

    Ok(ExitCode::from(shim::exit_code(command.status()?)))
}

/// Documenting with the fragments rendered is documenting differently, and a
/// flag is what tells cargo that, so that it documents again rather than
/// leaving the pages of an ordinary `cargo doc` in place.
fn rustdocflags() -> OsString {
    let mut flags = std::env::var_os("RUSTDOCFLAGS").unwrap_or_default();
    if !flags.is_empty() {
        flags.push(" ");
    }
    flags.push("--cfg typstdoc");
    flags
}
