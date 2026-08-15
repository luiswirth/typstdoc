//! Compiles Typst fragments from Rust doc comments to HTML.
//!
//! The unit is a fragment in one of Typst's [syntactical modes][SyntaxMode],
//! so inline math, a whole doc comment and a figure are one path.
//! Each fragment compiles as its own document through a [`Renderer`] that keeps
//! the compiler's state, and yields the HTML together with the resources the
//! page carrying it needs.

mod error;
mod files;
mod fonts;
mod fragment;
mod world;

pub use typst::syntax::SyntaxMode;

pub use self::error::Error;
pub use self::files::{Directories, Files};
pub use self::fonts::Fonts;
pub use self::fragment::{Assets, Fragment};

use typst_html::HtmlDocument;

use self::world::FragmentWorld;

/// Renders the fragments of one crate.
pub struct Renderer {
    world: FragmentWorld,
}

impl Renderer {
    pub fn new(files: impl Files + 'static, fonts: Fonts) -> Self {
        Self {
            world: FragmentWorld::new(files, fonts),
        }
    }

    pub fn render(&mut self, source: &str, mode: SyntaxMode) -> Result<Fragment, Error> {
        self.world.set_main(fragment::wrap(source, mode));
        let compiled = typst::compile::<HtmlDocument>(&self.world);
        let document = compiled.output.map_err(Error::Compile)?;
        fragment::extract(&document)
    }
}
