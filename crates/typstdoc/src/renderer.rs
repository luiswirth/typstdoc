use std::cell::RefCell;
use std::path::PathBuf;

use typstdoc_core::{Directories, Error, Fonts, Rendered, Renderer, SyntaxMode};

thread_local! {
    /// One renderer for the whole build, so that the fonts, the files read and
    /// the compiler's memoization carry from one doc comment to the next.
    static RENDERER: RefCell<Renderer> = RefCell::new(new());
}

pub fn render(source: &str, mode: SyntaxMode) -> Result<Rendered, Error> {
    RENDERER.with_borrow_mut(|renderer| renderer.render(source, mode))
}

/// The renderer a documentation build compiles its fragments through.
///
/// The crate being documented is the project, and the packages are the ones
/// Typst has on the machine, of which a sandboxed build simply has none.
fn new() -> Renderer {
    let files = Directories {
        project: std::env::var_os("CARGO_MANIFEST_DIR").map(PathBuf::from),
        ..Directories::installed()
    };
    Renderer::new(files, Fonts::embedded())
}
