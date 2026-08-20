//! `cargo typstdoc`, which documents a crate with the Typst in its doc
//! comments rendered, without the crate saying anything about typstdoc.
//!
//! rustdoc reads the doc comments out of the sources of the crate it is given,
//! so the fragments are rendered in a copy of those sources and rustdoc is
//! pointed at the copy. Cargo lets the binary it runs as rustdoc be chosen,
//! which is where this one stands in.

mod cargo;
mod shim;
mod source;
mod stage;
mod theme;

use std::process::ExitCode;

/// Whatever went wrong, which is only ever reported and never handled.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> ExitCode {
    let result = match std::env::var_os(shim::MARKER) {
        Some(_) => shim::run(),
        None => cargo::run(),
    };

    match result {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
