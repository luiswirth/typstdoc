use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use typstdoc_core::markdown;

use crate::renderer;

/// One `#[doc = "..."]` attribute.
struct Doc {
    /// Whether the attribute is an inner one, as `//!` writes it.
    inner: bool,
    text: String,
    span: Span,
}

/// Renders the Typst fragments in the doc comments a token stream holds.
///
/// A fragment that does not compile is reported where it was written and left
/// as it was written, so that the rest of the item still documents.
pub fn rewrite(item: TokenStream) -> TokenStream {
    let mut errors = TokenStream::new();
    let mut out = stream(item, &mut errors);
    out.extend(tracked_preamble());
    out.extend(errors);
    out
}

/// Makes the preamble a source of the crate, so that rustc runs the macro
/// again once it changes.
///
/// A file a macro reads is invisible to rustc otherwise, and the only stable
/// way to name one is to include it. A preamble that is not there yet cannot
/// be named, so a crate that gains one first has to be built for another
/// reason.
fn tracked_preamble() -> TokenStream {
    if renderer::preamble().is_none() {
        return TokenStream::new();
    }
    format!(
        "const _: &str = include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}\"));",
        typstdoc_core::PREAMBLE
    )
    .parse()
    .expect("a valid item")
}

fn stream(stream: TokenStream, errors: &mut TokenStream) -> TokenStream {
    let mut out = TokenStream::new();
    let mut docs = Vec::new();
    let mut trees = stream.into_iter().peekable();

    while let Some(tree) = trees.next() {
        if punct(&tree, '#') {
            let inner = punct_at(trees.peek(), '!');
            let attribute = trees.clone().nth(usize::from(inner));
            if let Some(TokenTree::Group(group)) = attribute
                && let Some(text) = doc_text(&group)
            {
                // An inner attribute documents what holds the item and an outer
                // one the item itself, so the two are never one run.
                if docs.first().is_some_and(|first: &Doc| first.inner != inner) {
                    out.extend(rendered(&mut docs, errors));
                }
                docs.push(Doc {
                    inner,
                    text,
                    span: group.span(),
                });
                trees.nth(usize::from(inner));
                continue;
            }
        }

        out.extend(rendered(&mut docs, errors));
        out.extend([descend(tree, errors)]);
    }

    out.extend(rendered(&mut docs, errors));
    out
}

/// Rewrites the doc comments inside a group, since an item holds its items in
/// one.
fn descend(tree: TokenTree, errors: &mut TokenStream) -> TokenTree {
    let TokenTree::Group(group) = tree else {
        return tree;
    };
    let mut rewritten = Group::new(group.delimiter(), stream(group.stream(), errors));
    rewritten.set_span(group.span());
    TokenTree::Group(rewritten)
}

/// Emits a run of doc comments with its fragments rendered.
///
/// The run is one markdown document, so it is scanned as a whole and emitted
/// as one attribute. A run holding no fragment is emitted as it was written.
fn rendered(docs: &mut Vec<Doc>, errors: &mut TokenStream) -> TokenStream {
    let docs = std::mem::take(docs);
    let Some(first) = docs.first() else {
        return TokenStream::new();
    };

    let doc = markdown(docs.iter().map(|doc| doc.text.as_str()));
    let rendered = renderer::render_doc(&doc);

    for failure in &rendered.failures {
        errors.extend(compile_error(first.span, &failure.error.to_string()));
    }

    attribute(first.inner, &rendered.markdown, first.span)
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
        (TokenTree::Ident(name), equals, TokenTree::Literal(value), None)
            if name.to_string() == "doc" && punct(&equals, '=') =>
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

fn attribute(inner: bool, text: &str, span: Span) -> TokenStream {
    let mut value = Literal::string(text);
    value.set_span(span);

    let mut attribute = TokenStream::from(TokenTree::Punct(Punct::new('#', Spacing::Alone)));
    if inner {
        attribute.extend([TokenTree::Punct(Punct::new('!', Spacing::Alone))]);
    }
    attribute.extend([TokenTree::Group(Group::new(
        Delimiter::Bracket,
        [
            TokenTree::Ident(Ident::new("doc", span)),
            TokenTree::Punct(Punct::new('=', Spacing::Alone)),
            TokenTree::Literal(value),
        ]
        .into_iter()
        .collect(),
    ))]);

    spanned(attribute, span)
}

fn compile_error(span: Span, message: &str) -> TokenStream {
    let error: TokenStream = format!("::core::compile_error!({:?});", message)
        .parse()
        .expect("a message is a string literal");
    spanned(error, span)
}

/// Points every token at the doc comment it came from.
fn spanned(stream: TokenStream, span: Span) -> TokenStream {
    stream
        .into_iter()
        .map(|mut tree| {
            tree.set_span(span);
            tree
        })
        .collect()
}

fn punct(tree: &TokenTree, character: char) -> bool {
    matches!(tree, TokenTree::Punct(punct) if punct.as_char() == character)
}

fn punct_at(tree: Option<&TokenTree>, character: char) -> bool {
    tree.is_some_and(|tree| punct(tree, character))
}
