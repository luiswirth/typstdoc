//! Compiles Typst fragments from Rust doc comments to HTML.
//!
//! The unit is a fragment in one of Typst's [syntactical modes][SyntaxMode],
//! so inline math, a whole doc comment and a figure are one path.
//! [`scan`] finds the fragments a doc comment holds, and a [`Renderer`] keeps
//! the compiler's state while it renders them, each as its own document, to
//! HTML together with the resources the page carrying it needs.

mod doc;
mod error;
mod files;
mod fonts;
mod render;
mod scan;
mod world;

pub use typst::syntax::SyntaxMode;

pub use self::doc::{Failure, PREAMBLE, RenderedDoc, markdown};
pub use self::error::Error;
pub use self::files::{Directories, Files};
pub use self::fonts::Fonts;
pub use self::render::{Assets, Rendered, Renderer};
pub use self::scan::{Fragment, scan};
