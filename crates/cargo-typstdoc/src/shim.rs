use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, ExitCode, ExitStatus};

use crate::Result;
use crate::stage;

/// Set on the `cargo doc` this binary runs, so that the same binary knows when
/// it is the one being run as rustdoc.
pub const MARKER: &str = "TYPSTDOC_SHIM";

/// The preamble chosen on the command line, which stands in for the one that
/// would be found beside the crate.
pub const PREAMBLE: &str = "TYPSTDOC_PREAMBLE";

/// The rustdoc to hand the rendered sources to, for a toolchain this binary
/// cannot find on its own.
pub const RUSTDOC: &str = "TYPSTDOC_RUSTDOC";

/// Renders the crate rustdoc was called on and calls rustdoc on the rendering.
///
/// Cargo calls rustdoc for other reasons than documenting a crate, asking it
/// for its version among them, and such a call is passed on as it came.
pub fn run() -> Result<ExitCode> {
    let mut args: Vec<OsString> = std::env::args_os().skip(1).collect();

    if let Some(index) = input(&args) {
        let root = PathBuf::from(&args[index]);
        args[index] = stage::render(&root)?.into_os_string();
    }

    let status = Command::new(rustdoc()?)
        .args(args)
        .env_remove(MARKER)
        .status()?;
    Ok(ExitCode::from(exit_code(status)))
}

/// Where the crate root stands in rustdoc's arguments.
///
/// rustdoc takes one input, and it is the only argument of a documentation
/// build that names a Rust source.
fn input(args: &[OsString]) -> Option<usize> {
    args.iter()
        .position(|arg| arg.as_encoded_bytes().ends_with(b".rs"))
}

/// The real rustdoc, which is the one of the toolchain rustc comes from.
fn rustdoc() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os(RUSTDOC) {
        return Ok(path.into());
    }

    let rustc = std::env::var_os("RUSTC").unwrap_or("rustc".into());
    let sysroot = Command::new(rustc).arg("--print").arg("sysroot").output()?;
    let sysroot = String::from_utf8(sysroot.stdout)?;

    Ok(PathBuf::from(sysroot.trim()).join("bin").join("rustdoc"))
}

/// A process that a signal ended reports no code of its own, and having ended
/// some other way than by finishing is a failure.
pub fn exit_code(status: ExitStatus) -> u8 {
    status.code().unwrap_or(1).try_into().unwrap_or(1)
}
