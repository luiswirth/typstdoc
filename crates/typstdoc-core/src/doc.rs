use std::ops::Range;

use crate::error::Error;
use crate::render::{Assets, Renderer};
use crate::scan::scan;

/// The file a crate writes its preamble in, at its root.
pub const PREAMBLE: &str = "typstdoc.typ";

/// A doc comment whose Typst fragments were rendered.
#[derive(Debug)]
pub struct RenderedDoc {
    /// The markdown rustdoc reads, holding HTML where a fragment stood.
    pub markdown: String,
    /// What a page carrying the doc comment needs beyond that markdown.
    pub assets: Assets,
    /// The fragments that did not compile.
    pub failures: Vec<Failure>,
}

/// A fragment that did not compile, left as it was written.
#[derive(Debug)]
pub struct Failure {
    /// Where the fragment lies in the doc comment it was written in.
    pub range: Range<usize>,
    pub error: Error,
}

impl Renderer {
    /// Renders every Typst fragment a doc comment holds.
    ///
    /// What is not a fragment is left as it was written, and so is a fragment
    /// that does not compile, so that the rest of the doc comment still
    /// documents.
    pub fn render_doc(&mut self, doc: &str) -> RenderedDoc {
        let mut rendered = RenderedDoc {
            markdown: String::new(),
            assets: Assets::default(),
            failures: Vec::new(),
        };
        let mut rest = 0;

        for fragment in scan(doc) {
            rendered.markdown.push_str(&doc[rest..fragment.range.start]);
            match self.render(fragment.source, fragment.mode) {
                Ok(one) => {
                    rendered.markdown.push_str(&one.html);
                    rendered.assets.absorb(one.assets);
                }
                Err(error) => {
                    rendered.markdown.push_str(&doc[fragment.range.clone()]);
                    rendered.failures.push(Failure {
                        range: fragment.range.clone(),
                        error,
                    });
                }
            }
            rest = fragment.range.end;
        }

        rendered.markdown.push_str(&doc[rest..]);
        rendered
    }
}

/// Joins a run of doc comments into the markdown rustdoc reads.
///
/// A doc comment is one line, and the indentation its lines share follows the
/// comment marker rather than the markdown.
pub fn markdown<'a>(lines: impl IntoIterator<Item = &'a str>) -> String {
    let text = lines.into_iter().collect::<Vec<_>>().join("\n");

    let indent = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    text.lines()
        .map(|line| line.get(indent..).unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}
