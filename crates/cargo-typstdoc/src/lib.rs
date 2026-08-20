//! `cargo typstdoc`, which documents a crate with the Typst in its doc
//! comments rendered, without the crate saying anything about typstdoc.
//!
//! rustdoc reads the doc comments out of the sources of the crate it is given,
//! so the fragments are rendered in a copy of those sources and rustdoc is
//! pointed at the copy. Cargo lets the binary it runs as rustdoc be chosen,
//! which is where this one stands in.

pub mod cargo;
pub mod shim;
pub mod source;
pub mod stage;
pub mod theme;

/// Whatever went wrong, which is only ever reported and never handled.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
