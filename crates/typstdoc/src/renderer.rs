use std::cell::RefCell;
use std::path::PathBuf;

use typstdoc_core::{Directories, Fonts, PREAMBLE, RenderedDoc, Renderer};

thread_local! {
    /// One renderer for the whole build, so that the fonts, the files read and
    /// the compiler's memoization carry from one doc comment to the next.
    static RENDERER: RefCell<Renderer> = RefCell::new(new());
}

pub fn render_doc(doc: &str) -> RenderedDoc {
    RENDERER.with_borrow_mut(|renderer| renderer.render_doc(doc))
}

/// The path of the crate's preamble, if the crate wrote one.
pub fn preamble() -> Option<PathBuf> {
    let path = root()?.join(PREAMBLE);
    path.is_file().then_some(path)
}

/// The renderer a documentation build compiles its fragments through.
///
/// The crate being documented is the project, and the packages are the ones
/// Typst has on the machine, of which a sandboxed build simply has none.
fn new() -> Renderer {
    let files = Directories {
        project: root(),
        ..Directories::installed()
    };
    Renderer::new(files, Fonts::embedded(), read(preamble()))
}

/// A preamble that is there and cannot be read is a failure of the build,
/// where one that is not there is a crate that has none.
fn read(path: Option<PathBuf>) -> String {
    let Some(path) = path else {
        return String::new();
    };
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()))
}

/// The root of the crate being documented.
fn root() -> Option<PathBuf> {
    std::env::var_os("CARGO_MANIFEST_DIR").map(PathBuf::from)
}
