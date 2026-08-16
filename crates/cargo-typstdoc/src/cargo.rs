use std::collections::hash_map::DefaultHasher;
use std::ffi::{OsString};
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::process::{Command, ExitCode};

use typstdoc_core::PREAMBLE;

use crate::Result;
use crate::shim;

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
        .env(
            "RUSTDOCFLAGS",
            rustdocflags(stamp(preamble.as_deref().map(Path::new))),
        );
    if let Some(preamble) = preamble {
        command.env(shim::PREAMBLE, preamble);
    }

    Ok(ExitCode::from(shim::exit_code(command.status()?)))
}

/// The flags that tell cargo which documentation build this is.
///
/// Cargo documents again when the flags differ, and what it knows of a build
/// is the sources and these, so documenting with the fragments rendered is
/// another build than an ordinary `cargo doc`, and so is one where the
/// preamble or this binary has changed since.
fn rustdocflags(stamp: u64) -> OsString {
    let mut flags = std::env::var_os("RUSTDOCFLAGS").unwrap_or_default();
    if !flags.is_empty() {
        flags.push(" ");
    }
    flags.push(format!(
        "--cfg typstdoc=\"{stamp:016x}\" --check-cfg cfg(typstdoc,values(any()))"
    ));
    flags
}

/// What a documentation build rests on besides the sources cargo watches.
fn stamp(preamble: Option<&Path>) -> u64 {
    let mut hasher = DefaultHasher::new();

    if let Ok(exe) = std::env::current_exe() {
        version(&exe).hash(&mut hasher);
    }
    match preamble {
        Some(path) => std::fs::read(path).ok().hash(&mut hasher),
        None => preambles(Path::new(".")).hash(&mut hasher),
    }

    hasher.finish()
}

/// Every preamble under a directory, since any of them may be the one a
/// package of the build is documented with.
fn preambles(directory: &Path) -> Vec<Option<Vec<u8>>> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(directory) else {
        return found;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            if name != "target" && name != ".git" {
                found.extend(preambles(&entry.path()));
            }
        } else if name == PREAMBLE {
            found.push(std::fs::read(entry.path()).ok());
        }
    }

    found
}

/// What tells one build of a file from another, without reading it.
fn version(path: &Path) -> Option<(u64, std::time::SystemTime)> {
    let metadata = std::fs::metadata(path).ok()?;
    Some((metadata.len(), metadata.modified().ok()?))
}
