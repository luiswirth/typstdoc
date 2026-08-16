use std::ops::Range;
use std::path::Path;

use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use typstdoc_core::{Error, Renderer, markdown};

pub use typstdoc_core::PREAMBLE;

/// One doc comment, and where it was written.
struct Doc {
    /// Whether the comment documents what holds the item, as `//!` writes it.
    inner: bool,
    text: String,
    range: Range<usize>,
}

/// A Rust source with the Typst in its doc comments rendered.
///
/// A source that holds no fragment is copied rather than rewritten, so a
/// source rustdoc reads differs from the one the crate keeps only where a
/// fragment stood.
pub fn render(path: &Path, renderer: &mut Renderer) -> std::io::Result<Vec<u8>> {
    let source = std::fs::read(path)?;
    let Ok(text) = std::str::from_utf8(&source) else {
        return Ok(source);
    };
    if !text.contains('$') {
        return Ok(source);
    }

    Ok(rewrite(text, renderer, &mut |line, fragment, error| {
        eprintln!("warning: {}:{line}: {error}\n  in {fragment}", path.display())
    })
    .into_bytes())
}

/// Rewrites the doc comments of a Rust source, leaving the rest of it byte for
/// byte as it was.
///
/// Every doc comment is replaced by an attribute holding the same
/// documentation, which is the one form a rendered fragment fits in. The
/// attribute is written across as many lines as the comments it replaces, so
/// that everything below it stays on the line it was written on and the source
/// rustdoc shows still matches the one the crate keeps.
pub fn rewrite(
    source: &str,
    renderer: &mut Renderer,
    report: &mut impl FnMut(usize, &str, &Error),
) -> String {
    let Ok(stream) = source.parse::<TokenStream>() else {
        return source.into();
    };

    let mut docs = Vec::new();
    collect(stream, &mut docs);

    let mut out = String::with_capacity(source.len());
    let mut rest = 0;

    for run in runs(source, &docs) {
        let first = &docs[run.start];
        let range = first.range.start..docs[run.end - 1].range.end;
        let doc = markdown(docs[run].iter().map(|doc| doc.text.as_str()));
        let rendered = renderer.render_doc(&doc);

        // A doc comment is one line, so a line of the run is a line of the
        // source, however far into the run the fragment lies.
        let first_line = line(source, range.start);
        for failure in &rendered.failures {
            let line = first_line + doc[..failure.range.start].matches('\n').count();
            report(line, &doc[failure.range.clone()], &failure.error);
        }
        if rendered.markdown == doc {
            continue;
        }

        out.push_str(&source[rest..range.start]);
        out.push_str(&attribute(
            first.inner,
            &rendered.markdown,
            source[range.clone()].matches('\n').count(),
        ));
        rest = range.end;
    }

    out.push_str(&source[rest..]);
    out
}

/// The doc comments a token stream holds, in the order they were written.
fn collect(stream: TokenStream, docs: &mut Vec<Doc>) {
    let mut trees = stream.into_iter().peekable();

    while let Some(tree) = trees.next() {
        match tree {
            TokenTree::Punct(punct) if punct.as_char() == '#' => {
                let inner = matches!(trees.peek(), Some(TokenTree::Punct(punct)) if punct.as_char() == '!');
                if inner {
                    trees.next();
                }
                let Some(TokenTree::Group(group)) = trees.peek() else {
                    continue;
                };
                if let Some(text) = doc_text(group) {
                    docs.push(Doc {
                        inner,
                        text,
                        range: punct.span().byte_range().start..group.span().byte_range().end,
                    });
                    trees.next();
                }
            }
            TokenTree::Group(group) => collect(group.stream(), docs),
            _ => {}
        }
    }
}

/// The runs of doc comments that are one markdown document each.
///
/// Consecutive comments of the same kind are one document, and anything but
/// the whitespace between them ends the run, as does a comment of the other
/// kind: an inner comment documents what holds the item and an outer one the
/// item itself.
fn runs(source: &str, docs: &[Doc]) -> Vec<Range<usize>> {
    let mut runs: Vec<Range<usize>> = Vec::new();

    for (i, doc) in docs.iter().enumerate() {
        let joins = runs.last().is_some_and(|run| {
            let previous = &docs[run.end - 1];
            previous.inner == doc.inner && source[previous.range.end..doc.range.start].trim().is_empty()
        });

        match runs.last_mut() {
            Some(run) if joins => run.end = i + 1,
            _ => runs.push(i..i + 1),
        }
    }

    runs
}

/// The text of a `#[doc = "..."]` attribute.
///
/// Only a string decides anything here, so `#[doc(hidden)]` and a `#[doc]` of
/// something yet to be evaluated are left to rustdoc.
fn doc_text(group: &Group) -> Option<String> {
    if group.delimiter() != Delimiter::Bracket {
        return None;
    }
    let mut trees = group.stream().into_iter();
    let (name, equals, value, end) = (trees.next()?, trees.next()?, trees.next()?, trees.next());

    match (name, equals, value, end) {
        (TokenTree::Ident(name), TokenTree::Punct(equals), TokenTree::Literal(value), None)
            if name == "doc" && equals.as_char() == '=' =>
        {
            Some(
                litrs::StringLit::parse(value.to_string())
                    .ok()?
                    .value()
                    .to_owned(),
            )
        }
        _ => None,
    }
}

/// A doc attribute written across the given number of extra lines.
///
/// Rust reads a newline between two tokens as the space between them, so an
/// attribute stretches over as many lines as it has to without saying anything
/// else.
fn attribute(inner: bool, text: &str, lines: usize) -> String {
    let bang = if inner { "!" } else { "" };
    let breaks = "\n".repeat(lines);
    format!("#{bang}[doc ={breaks} \"{}\"]", escape(text))
}

/// A string as Rust source writes it.
fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '"' | '\\' => {
                escaped.push('\\');
                escaped.push(character);
            }
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ if character.is_control() => {
                escaped.push_str(&format!("\\u{{{:x}}}", character as u32))
            }
            _ => escaped.push(character),
        }
    }
    escaped
}

fn line(source: &str, offset: usize) -> usize {
    source[..offset].matches('\n').count() + 1
}
