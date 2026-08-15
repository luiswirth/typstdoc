use std::ops::Range;

use pulldown_cmark::{Event, Options, Parser, Tag};
use typst::syntax::SyntaxMode;

/// A Typst fragment written in a doc comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment<'a> {
    /// Where the fragment lies in the doc comment, delimiters included.
    pub range: Range<usize>,
    /// What the delimiters enclose, which is what Typst compiles.
    pub source: &'a str,
    pub mode: SyntaxMode,
}

/// Finds the Typst fragments in a doc comment.
///
/// Markdown says where a fragment may lie, so a `$` in a code span or a code
/// block is text, as it is to rustdoc. Typst says what lies between the
/// delimiters, so the spacing behind them tells inline math from display math,
/// where TeX and the pulldown-cmark math spec read a second `$`.
pub fn scan(doc: &str) -> Vec<Fragment<'_>> {
    let ranges = code_ranges(doc);
    let mut code = ranges.iter().peekable();
    let mut fragments = Vec::new();
    let mut i = 0;

    while i < doc.len() {
        while code.peek().is_some_and(|range| range.end <= i) {
            code.next();
        }
        if let Some(range) = code.peek()
            && range.start <= i
        {
            i = range.end;
            continue;
        }
        let limit = code.peek().map_or(doc.len(), |range| range.start);

        match doc.as_bytes()[i] {
            b'\\' => i += 2,
            b'$' => match closing(doc, i, limit) {
                Some(close) => {
                    fragments.push(Fragment {
                        range: i..close + 1,
                        source: &doc[i + 1..close],
                        mode: SyntaxMode::Math,
                    });
                    i = close + 1;
                }
                None => i += 1,
            },
            _ => i += 1,
        }
    }

    fragments
}

/// Finds the `$` closing the one at `open`, searching up to `limit`.
///
/// A fragment lies within one block of the markdown around it, so a blank line
/// ends the search and leaves the opening delimiter as text.
fn closing(doc: &str, open: usize, limit: usize) -> Option<usize> {
    let mut i = open + 1;
    while i < limit {
        match doc.as_bytes()[i] {
            b'\\' => i += 2,
            b'$' => return Some(i),
            b'\n' if blank(doc, i + 1) => return None,
            _ => i += 1,
        }
    }
    None
}

/// Whether the line beginning at `i` holds nothing.
fn blank(doc: &str, i: usize) -> bool {
    doc[i..].trim_start_matches([' ', '\t']).starts_with('\n')
}

/// The ranges markdown reads as code, in the order they appear.
fn code_ranges(doc: &str) -> Vec<Range<usize>> {
    Parser::new_ext(doc, Options::empty())
        .into_offset_iter()
        .filter(|(event, _)| matches!(event, Event::Code(_) | Event::Start(Tag::CodeBlock(_))))
        .map(|(_, range)| range)
        .collect()
}
